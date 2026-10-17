use sovereign_zkp::{ZkAgeProver, ZkAgeVerifier, MinorProtectionPolicy};

#[test]
fn test_zero_knowledge_age_proof_and_minor_protection() {
    let now = 1729185000; // 2026-10-17
    let aadhaar_seed = b"AADHAAR_MOCK_SEED_TOKEN_1234_5678";

    // 1. Adult user: Born 25 years ago
    let adult_dob = now - (25 * 365 * 86400);
    let adult_attestation = ZkAgeProver::prove_age(adult_dob, aadhaar_seed, now);

    assert!(adult_attestation.is_adult);
    let verify_adult = ZkAgeVerifier::verify(&adult_attestation)
        .expect("Verification must pass");
    assert!(verify_adult);
    assert!(MinorProtectionPolicy::can_target_ads(verify_adult));
    assert!(!MinorProtectionPolicy::requires_parental_consent(verify_adult));

    // 2. Minor user: Born 14 years ago
    let minor_dob = now - (14 * 365 * 86400);
    let minor_attestation = ZkAgeProver::prove_age(minor_dob, aadhaar_seed, now);

    assert!(!minor_attestation.is_adult);
    let verify_minor = ZkAgeVerifier::verify(&minor_attestation)
        .expect("Verification of minor attestation must pass");
    assert!(!verify_minor);

    // Section 9 DPDP Act Minor Restrictions:
    assert!(!MinorProtectionPolicy::can_target_ads(verify_minor), "Minors CANNOT be served targeted ads");
    assert!(!MinorProtectionPolicy::can_track_behavior(verify_minor), "Minors CANNOT be behaviorally monitored");
    assert!(MinorProtectionPolicy::requires_parental_consent(verify_minor), "Minors REQUIRE parental consent");

    // 3. Tampered proof fails verification
    let mut tampered = adult_attestation.clone();
    tampered.proof_signature[0] ^= 0xFF;
    let tamper_res = ZkAgeVerifier::verify(&tampered);
    assert!(tamper_res.is_err());
}
