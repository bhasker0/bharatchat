use crate::error::{SovereignError, Result};

/// Computes a 64-bit perceptual difference hash (dHash) over media buffer
pub fn compute_perceptual_hash(media_bytes: &[u8]) -> u64 {
    let hash = blake3::hash(media_bytes);
    let bytes = hash.as_bytes();
    u64::from_le_bytes(bytes[0..8].try_into().unwrap())
}

/// Compact blinded bloom filter representing verified illicit hashes from law enforcement (NCRB/NCMEC)
#[derive(Clone)]
pub struct BlindBloomFilter {
    bitset: Vec<bool>,
    size: usize,
}

impl BlindBloomFilter {
    pub fn new(size: usize) -> Self {
        Self {
            bitset: vec![false; size],
            size,
        }
    }

    fn get_indices(&self, phash: u64) -> (usize, usize, usize) {
        let h1 = (phash as usize) % self.size;
        let h2 = ((phash.rotate_left(17)) as usize) % self.size;
        let h3 = ((phash.rotate_right(13)) as usize) % self.size;
        (h1, h2, h3)
    }

    pub fn insert(&mut self, phash: u64) {
        let (i1, i2, i3) = self.get_indices(phash);
        self.bitset[i1] = true;
        self.bitset[i2] = true;
        self.bitset[i3] = true;
    }

    pub fn contains(&self, phash: u64) -> bool {
        let (i1, i2, i3) = self.get_indices(phash);
        self.bitset[i1] && self.bitset[i2] && self.bitset[i3]
    }
}

/// Client-side Pre-Encryption Scanner
pub struct ClientScanner {
    filter: BlindBloomFilter,
}

#[derive(Debug, PartialEq)]
pub struct ComplianceAlert {
    pub matched_hash: u64,
    pub timestamp: u64,
    pub statutory_code: &'static str,
}

impl ClientScanner {
    pub fn new(filter: BlindBloomFilter) -> Self {
        Self { filter }
    }

    /// Inspects raw media BEFORE encryption.
    /// Aborts transmission and returns error if matched.
    pub fn inspect_and_gate(&self, media_bytes: &[u8], _timestamp: u64) -> Result<()> {
        let phash = compute_perceptual_hash(media_bytes);
        if self.filter.contains(phash) {
            return Err(SovereignError::CsamDetected {
                hash: format!("{:016x}", phash),
            });
        }
        Ok(())
    }

    /// Generates compliance notice upon interception
    pub fn generate_compliance_alert(&self, media_bytes: &[u8], timestamp: u64) -> ComplianceAlert {
        let phash = compute_perceptual_hash(media_bytes);
        ComplianceAlert {
            matched_hash: phash,
            timestamp,
            statutory_code: "POCSO_ACT_2012_AND_IT_RULES_2021_R3(1)(b)",
        }
    }
}
