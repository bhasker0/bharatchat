use sovereign_upi::{SimBindingToken, UpiPaymentOrder, PaymentStatus};

#[test]
fn test_sim_binding_and_upi_payment_lifecycle() {
    let now = 1729186000;
    let device_hardware_id = "ANDROID_TITAN_M2_ENCLAVE_DEV987";
    let sim_iccid = "89918765432109876543";

    // 1. Hardware SIM Binding
    let token = SimBindingToken::create(device_hardware_id, sim_iccid, now);
    assert!(token.verify(device_hardware_id, sim_iccid));
    // SIM swap / device change simulation fails verification
    assert!(!token.verify("DIFFERENT_PHONE_ID", sim_iccid));
    assert!(!token.verify(device_hardware_id, "SWAPPED_SIM_ICCID"));

    // 2. UPI 2.0 Payment Intent Generation
    let mut order = UpiPaymentOrder::new(
        "merchant.store@hdfcbank",
        "FabIndia Organics",
        1499.50,
        "Order #87654 Festive Kurta",
        now,
    ).expect("Order creation should succeed");

    assert_eq!(order.status, PaymentStatus::Initiated);
    assert_eq!(order.data_jurisdiction, "INDIA_LOCALIZED_MUMBAI_BLR");

    let upi_uri = order.generate_upi_uri();
    assert!(upi_uri.starts_with("upi://pay?pa=merchant.store@hdfcbank"));
    assert!(upi_uri.contains("am=1499.50"));
    assert!(upi_uri.contains("cu=INR"));

    // 3. Dispatch to NPCI Bank Switch
    order.dispatch_to_bank_switch().expect("Dispatch should transition state");
    assert_eq!(order.status, PaymentStatus::PendingBankSwitch);

    // 4. Finalize payment outcome
    order.finalize(true, now + 15).expect("Finalization should succeed");
    assert_eq!(order.status, PaymentStatus::Success);
    assert_eq!(order.finalized_at, Some(now + 15));
}
