use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use sovereign_core::StorageMasterKey;

pub const DEFAULT_CHUNK_SIZE: usize = 256 * 1024; // 256 KB chunks

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Chunk not found on storage nodes: {0}")]
    ChunkNotFound(String),

    #[error("Crypto engine error: {0}")]
    CryptoError(String),

    #[error("Storage access revoked or key shredded")]
    AccessRevoked,
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChunkDescriptor {
    pub chunk_cid: String,
    pub nonce: [u8; 12],
    pub encrypted_size: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EncryptedMediaDescriptor {
    pub root_cid: String,
    pub original_size: usize,
    pub mime_type: String,
    pub chunks: Vec<ChunkDescriptor>,
}

/// Simulated sovereign off-chain IPFS / Ceph storage cluster in Mumbai / Bengaluru
#[derive(Clone)]
pub struct SovereignStorageCluster {
    /// CID -> Raw Encrypted Bytes
    storage_nodes: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    /// User DID -> Set of CIDs pinned by this user
    user_pins: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl SovereignStorageCluster {
    pub fn new() -> Self {
        Self {
            storage_nodes: Arc::new(RwLock::new(HashMap::new())),
            user_pins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Chunks, encrypts, and stores media off-chain under the user's StorageMasterKey
    pub async fn upload_media(
        &self,
        owner_did: &str,
        media_bytes: &[u8],
        mime_type: &str,
        smk: &StorageMasterKey,
    ) -> Result<EncryptedMediaDescriptor> {
        let mut chunks = Vec::new();
        let mut all_chunk_cids = Vec::new();

        let mut nodes = self.storage_nodes.write().await;
        let mut pins = self.user_pins.write().await;
        let user_pin_list = pins.entry(owner_did.to_string()).or_insert_with(Vec::new);

        for chunk_slice in media_bytes.chunks(DEFAULT_CHUNK_SIZE) {
            let (ciphertext, nonce) = smk.encrypt_payload(chunk_slice)
                .map_err(|e| StorageError::CryptoError(e.to_string()))?;

            let chunk_cid = format!("bafk{}", hex::encode(blake3::hash(&ciphertext).as_bytes()));
            let encrypted_size = ciphertext.len();

            nodes.insert(chunk_cid.clone(), ciphertext);
            user_pin_list.push(chunk_cid.clone());
            all_chunk_cids.push(chunk_cid.clone());

            chunks.push(ChunkDescriptor {
                chunk_cid,
                nonce,
                encrypted_size,
            });
        }

        let root_cid = format!("bafy{}", hex::encode(blake3::hash(all_chunk_cids.join("").as_bytes()).as_bytes()));

        Ok(EncryptedMediaDescriptor {
            root_cid,
            original_size: media_bytes.len(),
            mime_type: mime_type.to_string(),
            chunks,
        })
    }

    /// Fetches and reassembles media chunks, decrypting under user's StorageMasterKey
    pub async fn download_media(
        &self,
        descriptor: &EncryptedMediaDescriptor,
        smk: &StorageMasterKey,
    ) -> Result<Vec<u8>> {
        let nodes = self.storage_nodes.read().await;
        let mut assembled_media = Vec::with_capacity(descriptor.original_size);

        for chunk_desc in &descriptor.chunks {
            let ciphertext = nodes.get(&chunk_desc.chunk_cid)
                .ok_or_else(|| StorageError::ChunkNotFound(chunk_desc.chunk_cid.clone()))?;

            let decrypted_chunk = smk.decrypt_payload(ciphertext, &chunk_desc.nonce)
                .map_err(|e| StorageError::CryptoError(e.to_string()))?;

            assembled_media.extend_from_slice(&decrypted_chunk);
        }

        Ok(assembled_media)
    }

    /// DPDP Act 2023 Section 12 Cascade Erasure: Unpins and physically deletes all blocks owned by user
    pub async fn unpin_and_purge_user(&self, owner_did: &str) -> usize {
        let mut pins = self.user_pins.write().await;
        let mut nodes = self.storage_nodes.write().await;

        if let Some(user_cids) = pins.remove(owner_did) {
            let count = user_cids.len();
            for cid in user_cids {
                nodes.remove(&cid);
            }
            count
        } else {
            0
        }
    }

    /// Returns the number of raw blocks stored on nodes
    pub async fn total_stored_blocks(&self) -> usize {
        let nodes = self.storage_nodes.read().await;
        nodes.len()
    }
}
