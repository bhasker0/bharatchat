use sovereign_mls::{CommunityRole, MlsError, MlsGroupSession, UmbrellaCommunity};

#[test]
fn test_treekem_1000_member_group_scaling_and_epoch_transitions() {
    let mut group = MlsGroupSession::new("grp_surat_textile_artisans", "did:bharat:admin");

    // Add 1,023 members to reach 1,024 members (WhatsApp 1000+ member parity)
    for i in 1..=1023 {
        let did = format!("did:bharat:karigar_{:04}", i);
        group.add_member(&did, CommunityRole::Member).unwrap();
    }

    assert_eq!(group.member_count(), 1024);
    assert_eq!(group.epoch, 1024);

    // Encrypt group broadcast from member 500
    let sender_did = "did:bharat:karigar_0500";
    let message = b"Diwali bonus distribution hisab posted on secure storage";
    let envelope = group
        .encrypt_group_message(sender_did, message)
        .expect("Group encryption should succeed");

    assert_eq!(envelope.epoch, 1024);

    // Decrypt group message
    let decrypted = group
        .decrypt_group_message(&envelope)
        .expect("Group decryption should succeed");
    assert_eq!(decrypted, message);

    // Remove a compromised or rogue member
    let removed_did = "did:bharat:karigar_0500";
    group.remove_member(removed_did).unwrap();
    assert_eq!(group.member_count(), 1023);
    assert_eq!(group.epoch, 1025);

    // Removed member cannot send messages to the group anymore
    let send_result = group.encrypt_group_message(removed_did, b"Rogue attempt");
    assert!(matches!(send_result, Err(MlsError::MemberNotFound(_))));

    // Old envelope from epoch 1024 cannot be decrypted under new epoch 1025
    let old_decrypt_attempt = group.decrypt_group_message(&envelope);
    assert!(old_decrypt_attempt.is_err());
}

#[test]
fn test_umbrella_community_announcement_permissions() {
    let mut community = UmbrellaCommunity::new(
        "comm_surat_textile_chamber",
        "Surat Chamber of Textile Commerce",
        "did:bharat:president_patel",
    );

    // Add regular member to announcements
    community
        .announcements_channel
        .add_member("did:bharat:trader_rohit", CommunityRole::Member)
        .unwrap();

    // 1. President (Admin) posts announcement -> Success
    let announcement_text = b"Annual Surat Textile Expo registration is now open.";
    let post_result = community.post_announcement(
        "did:bharat:president_patel",
        announcement_text,
    );
    assert!(post_result.is_ok());

    // 2. Regular trader attempts to post announcement -> Denied with Unauthorized
    let trader_attempt = community.post_announcement(
        "did:bharat:trader_rohit",
        b"Spam promotion",
    );
    assert!(matches!(trader_attempt, Err(MlsError::Unauthorized(CommunityRole::Admin))));

    // 3. Create child topic group
    community.create_topic_group("topic_raw_cotton_pricing", "did:bharat:president_patel");
    assert!(community.topic_groups.contains_key("topic_raw_cotton_pricing"));
}
