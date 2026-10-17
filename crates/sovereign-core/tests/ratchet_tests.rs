use sovereign_core::DoubleRatchetSession;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use rand::rngs::OsRng;

#[test]
fn test_double_ratchet_round_trip_and_ratchet_turns() {
    let shared_master = [42u8; 32];

    // Bob creates initial pre-key
    let bob_dh_secret = StaticSecret::random_from_rng(OsRng);
    let bob_dh_pubkey = X25519PublicKey::from(&bob_dh_secret);

    // Alice initializes session
    let mut alice = DoubleRatchetSession::init_alice(shared_master, *bob_dh_pubkey.as_bytes());

    // Bob initializes session
    let mut bob = DoubleRatchetSession::init_bob(shared_master, (bob_dh_secret, bob_dh_pubkey));

    // 1. Alice -> Bob
    let msg1_plain = b"Namaste Bharat! E2EE Double Ratchet Initialized.";
    let encrypted1 = alice.encrypt(msg1_plain).expect("Alice encryption failed");
    let decrypted1 = bob.decrypt(&encrypted1).expect("Bob decryption failed");
    assert_eq!(decrypted1, msg1_plain);

    // 2. Bob -> Alice (Triggers DH Ratchet Turn)
    let msg2_plain = b"Dhanyavaad Alice! Received securely.";
    let encrypted2 = bob.encrypt(msg2_plain).expect("Bob encryption failed");
    let decrypted2 = alice.decrypt(&encrypted2).expect("Alice decryption failed");
    assert_eq!(decrypted2, msg2_plain);

    // 3. Alice -> Bob (Another DH Ratchet Turn)
    let msg3_plain = b"Post-Compromise Security (PCS) active across ratchet epochs.";
    let encrypted3 = alice.encrypt(msg3_plain).expect("Alice encryption failed");
    let decrypted3 = bob.decrypt(&encrypted3).expect("Bob decryption failed");
    assert_eq!(decrypted3, msg3_plain);
}

#[test]
fn test_double_ratchet_out_of_order_delivery() {
    let shared_master = [99u8; 32];
    let bob_dh_secret = StaticSecret::random_from_rng(OsRng);
    let bob_dh_pubkey = X25519PublicKey::from(&bob_dh_secret);

    let mut alice = DoubleRatchetSession::init_alice(shared_master, *bob_dh_pubkey.as_bytes());
    let mut bob = DoubleRatchetSession::init_bob(shared_master, (bob_dh_secret, bob_dh_pubkey));

    // Alice sends 3 messages in sequence
    let msg0 = alice.encrypt(b"Packet #0 (First)").unwrap();
    let msg1 = alice.encrypt(b"Packet #1 (Second)").unwrap();
    let msg2 = alice.encrypt(b"Packet #2 (Third)").unwrap();

    // Bob receives Packet #2 first (out-of-order over network)
    let dec2 = bob.decrypt(&msg2).expect("Decrypting msg2 out of order");
    assert_eq!(dec2, b"Packet #2 (Third)");

    // Bob receives Packet #0 next
    let dec0 = bob.decrypt(&msg0).expect("Decrypting msg0 from skipped keys");
    assert_eq!(dec0, b"Packet #0 (First)");

    // Bob receives Packet #1 last
    let dec1 = bob.decrypt(&msg1).expect("Decrypting msg1 from skipped keys");
    assert_eq!(dec1, b"Packet #1 (Second)");
}

#[test]
fn test_double_ratchet_tamper_rejection() {
    let shared_master = [77u8; 32];
    let bob_dh_secret = StaticSecret::random_from_rng(OsRng);
    let bob_dh_pubkey = X25519PublicKey::from(&bob_dh_secret);

    let mut alice = DoubleRatchetSession::init_alice(shared_master, *bob_dh_pubkey.as_bytes());
    let mut bob = DoubleRatchetSession::init_bob(shared_master, (bob_dh_secret, bob_dh_pubkey));

    let mut encrypted = alice.encrypt(b"Secret Sovereign Payload").unwrap();

    // Tamper with ciphertext
    if let Some(byte) = encrypted.ciphertext.first_mut() {
        *byte ^= 0xFF;
    }

    let result = bob.decrypt(&encrypted);
    assert!(result.is_err(), "Tampered ciphertext must fail authentication");
}
