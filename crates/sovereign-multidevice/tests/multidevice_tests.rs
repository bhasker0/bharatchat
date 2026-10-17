use sovereign_core::IdentityKeyPair;
use sovereign_multidevice::{
    CompanionDeviceType, CompanionProver, MultiDeviceError, MultiDeviceManager,
};

#[test]
fn test_qr_pairing_flow_and_secure_companion_handshake() {
    let now = 1776000000u64;
    let primary_user = IdentityKeyPair::generate();
    let mut manager = MultiDeviceManager::new(&primary_user.did);

    // 1. Desktop companion generates QR challenge
    let (prover, qr_challenge) = CompanionProver::new_pairing_challenge("session_qr_991", now);
    assert_eq!(qr_challenge.session_id, "session_qr_991");

    // 2. Primary mobile scans QR and approves companion
    let (bundle, record) = manager
        .approve_pairing(
            &primary_user,
            &qr_challenge,
            CompanionDeviceType::DesktopMac,
            "Rahul's MacBook Pro M3",
            now + 5,
        )
        .expect("Primary pairing approval should succeed");

    assert!(manager.is_device_authorized(&record.device_id));
    assert_eq!(manager.list_active_devices().len(), 1);

    // 3. Companion decrypts bundle and validates primary signature & nonce
    let finalized = prover
        .finalize_pairing(&bundle)
        .expect("Companion bundle decryption must succeed");

    assert_eq!(finalized.primary_did, primary_user.did);
    assert_eq!(finalized.device_id, record.device_id);
    assert_eq!(finalized.pairing_nonce, qr_challenge.pairing_nonce);
}

#[test]
fn test_multi_device_revocation_and_unlinking() {
    let now = 1776000000u64;
    let primary_user = IdentityKeyPair::generate();
    let mut manager = MultiDeviceManager::new(&primary_user.did);

    let (_, challenge1) = CompanionProver::new_pairing_challenge("session_1", now);
    let (_, challenge2) = CompanionProver::new_pairing_challenge("session_2", now);

    let (_, dev1) = manager
        .approve_pairing(&primary_user, &challenge1, CompanionDeviceType::Tablet, "Rahul iPad", now)
        .unwrap();
    let (_, dev2) = manager
        .approve_pairing(&primary_user, &challenge2, CompanionDeviceType::WebBrowser, "Chrome Linux", now)
        .unwrap();

    assert_eq!(manager.list_active_devices().len(), 2);

    // Revoke iPad
    manager.revoke_device(&dev1.device_id).unwrap();
    assert!(!manager.is_device_authorized(&dev1.device_id));
    assert!(manager.is_device_authorized(&dev2.device_id));
    assert_eq!(manager.list_active_devices().len(), 1);

    // Duplicate revocation should fail
    let err = manager.revoke_device(&dev1.device_id);
    assert!(matches!(err, Err(MultiDeviceError::DeviceRevoked)));
}

#[test]
fn test_invalid_pairing_nonce_rejection() {
    let now = 1776000000u64;
    let primary_user = IdentityKeyPair::generate();
    let mut manager = MultiDeviceManager::new(&primary_user.did);

    let (prover_a, _challenge_a) = CompanionProver::new_pairing_challenge("session_a", now);
    let (_, challenge_b) = CompanionProver::new_pairing_challenge("session_b", now);

    // Primary approves challenge_b
    let (bundle_b, _) = manager
        .approve_pairing(&primary_user, &challenge_b, CompanionDeviceType::DesktopWindows, "Office PC", now)
        .unwrap();

    // Attacker tries to inject bundle_b into prover_a (mismatched nonce / pubkey)
    let outcome = prover_a.finalize_pairing(&bundle_b);
    assert!(outcome.is_err());
}
