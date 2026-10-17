use sovereign_commerce::{
    bot::{BusinessProfile, ConversationalBotEngine},
    catalog::{ItemAvailability, ProductCatalog, ProductItem},
    interactive::{InteractiveAction, InteractiveMessage, ListSection, ListRow},
    order::{OrderDraft, OrderStatus},
};

#[test]
fn test_jsonld_product_catalog_and_category_filtering() {
    let mut catalog = ProductCatalog::new("cat_surat_textiles_01", "did:bharat:merchant123", "Surat Silk & Sarees");

    let mut item1 = ProductItem::new(
        "prod_001",
        "SKU-SILK-001",
        "Pure Kanjivaram Silk Saree",
        "Handloom woven with authentic zari border",
        4500.00,
        "Textiles",
        "KANJI-001",
    ).expect("Product creation should succeed");
    item1.add_image_cid("bafkreicid_blake3_saree_front");

    let item2 = ProductItem::new(
        "prod_002",
        "SKU-COTTON-002",
        "Chanderi Cotton Kurti",
        "Breathable handblock printed summer wear",
        1200.00,
        "Textiles",
        "CHAND-002",
    ).expect("Product creation should succeed");

    catalog.add_item(item1);
    catalog.add_item(item2);

    assert_eq!(catalog.items.len(), 2);
    let textile_items = catalog.filter_by_category("textiles");
    assert_eq!(textile_items.len(), 2);

    let retrieved = catalog.find_item("prod_001").expect("Must find item");
    assert_eq!(retrieved.name, "Pure Kanjivaram Silk Saree");
    assert_eq!(retrieved.availability, ItemAvailability::InStock);

    // Verify JSON-LD Schema.org interoperability
    let jsonld = retrieved.to_jsonld();
    assert_eq!(jsonld["@context"], "https://schema.org/");
    assert_eq!(jsonld["@type"], "Product");
    assert_eq!(jsonld["offers"]["price"], "4500.00");
    assert_eq!(jsonld["offers"]["priceCurrency"], "INR");
    assert_eq!(jsonld["image"][0], "bafkreicid_blake3_saree_front");
}

#[test]
fn test_interactive_messages_and_button_construction() {
    let button_msg = InteractiveMessage::new_button_message(
        "Welcome to Surat Textiles! Please choose an option:",
        vec![
            ("btn_cat", "Browse Catalog"),
            ("btn_cart", "View Cart"),
            ("btn_help", "Talk to Support"),
        ],
        Some("Surat Textiles"),
        Some("Press a button to reply"),
    );

    match button_msg.action {
        InteractiveAction::Buttons { buttons } => {
            assert_eq!(buttons.len(), 3);
            assert_eq!(buttons[0].id, "btn_cat");
            assert_eq!(buttons[0].title, "Browse Catalog");
        }
        _ => panic!("Expected Buttons action"),
    }

    let list_msg = InteractiveMessage::new_list_message(
        "Select your fabric collection",
        "View Fabrics",
        vec![ListSection {
            title: "Traditional Weaves".to_string(),
            rows: vec![
                ListRow {
                    id: "row_kanjivaram".to_string(),
                    title: "Kanjivaram Silk".to_string(),
                    description: Some("From Tamil Nadu master weavers".to_string()),
                },
                ListRow {
                    id: "row_banarasi".to_string(),
                    title: "Banarasi Brocade".to_string(),
                    description: Some("Varanasi golden thread work".to_string()),
                },
            ],
        }],
        Some("Collections"),
        None,
    );

    match list_msg.action {
        InteractiveAction::List { button_text, sections } => {
            assert_eq!(button_text, "View Fabrics");
            assert_eq!(sections.len(), 1);
            assert_eq!(sections[0].rows.len(), 2);
        }
        _ => panic!("Expected List action"),
    }
}

#[test]
fn test_cart_order_tax_and_upi_intent_generation() {
    let mut order = OrderDraft::new("ORD-2026-9901", "did:bharat:buyer_rahul", "did:bharat:merchant_surat", 1776000000);
    order.add_item("prod_001", "Pure Kanjivaram Silk Saree", 4500.00, 2); // 9000.00
    order.add_item("prod_002", "Chanderi Cotton Kurti", 1200.00, 1);    // 1200.00
    // Total subtotal = 10,200.00

    // Indian GST calculation (e.g. 5% textile GST: 2.5% CGST + 2.5% SGST)
    order.calculate_totals(5.0);
    assert_eq!(order.subtotal_inr, 10200.00);
    assert_eq!(order.tax.total_tax_inr, 510.00);
    assert_eq!(order.tax.cgst_inr, 255.00);
    assert_eq!(order.tax.sgst_inr, 255.00);
    assert_eq!(order.total_amount_inr, 10710.00);

    // Generate NPCI UPI 2.0 intent URI
    let upi_uri = order.generate_upi_intent("merchant@icici", "Surat Textiles Pvt Ltd", 1776000010)
        .expect("Should generate UPI payment intent");

    assert!(upi_uri.starts_with("upi://pay?pa=merchant@icici"));
    assert!(upi_uri.contains("am=10710.00"));
    assert!(upi_uri.contains("cu=INR"));
    assert_eq!(order.status, OrderStatus::AwaitingPayment);

    // Confirm settlement
    order.confirm_payment(1776000050).expect("Confirmation should succeed");
    assert_eq!(order.status, OrderStatus::PaidConfirmed);
}

#[test]
fn test_automated_bot_greeting_away_and_keyword_routing() {
    let profile = BusinessProfile::default_retailer("did:bharat:merchant_surat", "Surat Handlooms");
    let mut bot = ConversationalBotEngine::new(profile);

    // Register a custom keyword route for "MENU"
    bot.register_keyword_route("MENU", InteractiveMessage::new_button_message(
        "Here is our service menu:",
        vec![("catalog", "Catalog"), ("offers", "Festive Offers")],
        None,
        None,
    ));

    // Case 1: First inbound greeting during business hours (14:00 IST)
    let greeting_resp = bot.process_inbound("did:bharat:customer_01", "Hello there", 1000, 14);
    assert!(greeting_resp.is_some());
    let greeting = greeting_resp.unwrap();
    assert!(greeting.body_text.contains("Namaste! Welcome to Surat Handlooms"));

    // Case 2: Immediate follow-up within 24h should NOT trigger greeting again
    let follow_up = bot.process_inbound("did:bharat:customer_01", "Can you help me?", 1050, 14);
    assert!(follow_up.is_none());

    // Case 3: Keyword routing matches "menu" regardless of case
    let menu_resp = bot.process_inbound("did:bharat:customer_01", "  menu  ", 1100, 14);
    assert!(menu_resp.is_some());
    assert_eq!(menu_resp.unwrap().body_text, "Here is our service menu:");

    // Case 4: Message sent during off-hours (22:00 IST) triggers Away message
    let away_resp = bot.process_inbound("did:bharat:customer_02", "Hi are you open?", 1200, 22);
    assert!(away_resp.is_some());
    assert!(away_resp.unwrap().body_text.contains("outside our operating hours"));
}
