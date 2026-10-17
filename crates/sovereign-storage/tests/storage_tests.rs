use sovereign_core::StorageMasterKey;
use sovereign_storage::SovereignStorageCluster;

#[tokio::test]
async fn test_offchain_media_upload_download_and_cascade_purge() {
    let cluster = SovereignStorageCluster::new();
    let alice_did = "did:bharat:alice_storage_test_user";
    let smk = StorageMasterKey::generate(alice_did);

    // 1MB test media buffer (spans multiple 256KB chunks)
    let original_media: Vec<u8> = (0..(1024 * 1024)).map(|i| (i % 256) as u8).collect();
    let mime_type = "video/mp4";

    // 1. Upload & encrypt media
    let descriptor = cluster.upload_media(alice_did, &original_media, mime_type, &smk).await
        .expect("Upload and chunking must succeed");

    assert_eq!(descriptor.chunks.len(), 4); // 1MB / 256KB = 4 chunks
    assert_eq!(descriptor.original_size, 1024 * 1024);
    assert_eq!(descriptor.mime_type, mime_type);
    assert!(cluster.total_stored_blocks().await >= 4);

    // 2. Download and decrypt media (byte-for-byte exact verification)
    let downloaded = cluster.download_media(&descriptor, &smk).await
        .expect("Download and reassembly must succeed");

    assert_eq!(downloaded.len(), original_media.len());
    assert_eq!(downloaded, original_media);

    // 3. DPDP Act Section 12 Cascade Purge
    let purged_count = cluster.unpin_and_purge_user(alice_did).await;
    assert_eq!(purged_count, 4);
    assert_eq!(cluster.total_stored_blocks().await, 0);

    // 4. Subsequent download attempt must fail because chunks were unpinned and purged
    let fail_download = cluster.download_media(&descriptor, &smk).await;
    assert!(fail_download.is_err());
}
