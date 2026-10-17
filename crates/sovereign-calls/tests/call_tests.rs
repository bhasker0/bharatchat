use sovereign_calls::{CallSession, CallType, CallStatus, CallError, MAXIMUM_CALL_PARTICIPANTS};

#[test]
fn test_webrtc_sframe_calling_and_capacity_limits() {
    let initiator = "did:bharat:host_doctor_patel";
    let now = 1729184000;

    let mut session = CallSession::new(initiator, CallType::AudioVideo, now);
    assert_eq!(session.status, CallStatus::Initiating);
    assert_eq!(session.participant_count(), 1);

    // 1. Up to 32 participants join
    let mut initial_key = None;
    for i in 1..MAXIMUM_CALL_PARTICIPANTS {
        let participant = format!("did:bharat:participant_{:02}", i);
        let key_dist = session.join(&participant, now + i as u64)
            .expect("Participant join within capacity must succeed");
        assert_eq!(key_dist.epoch, 1);
        if initial_key.is_none() {
            initial_key = Some(key_dist.sframe_key_bytes);
        } else {
            // All active participants share identical SFrame room key in epoch 1
            assert_eq!(Some(key_dist.sframe_key_bytes), initial_key);
        }
    }

    assert_eq!(session.participant_count(), 32);
    assert_eq!(session.status, CallStatus::Active);

    // 2. 33rd participant attempts to join -> Rejected with RoomFull
    let excess_participant = "did:bharat:participant_33";
    let excess_result = session.join(excess_participant, now + 100);
    assert!(matches!(excess_result, Err(CallError::RoomFull)));

    // 3. Signaling message creation (Offer/Answer/ICE)
    let signal = session.create_signaling(
        initiator,
        Some("did:bharat:participant_01"),
        "OFFER",
        "v=0\r\no=- 42 2 IN IP4 127.0.0.1...",
    ).expect("Signaling envelope generation must succeed");

    assert_eq!(signal.payload_type, "OFFER");
    assert_eq!(signal.epoch, 1);

    // 4. Participant leaves -> SFrame room key rotates (Forward/Backward Secrecy)
    let leaving_participant = "did:bharat:participant_01";
    let rotate_opt = session.leave(leaving_participant)
        .expect("Participant departure must succeed");

    let new_key_dist = rotate_opt.expect("New epoch key must be issued");
    assert_eq!(new_key_dist.epoch, 2);
    assert_ne!(Some(new_key_dist.sframe_key_bytes), initial_key);
    assert_eq!(session.participant_count(), 31);
}
