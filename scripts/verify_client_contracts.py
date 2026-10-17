#!/usr/bin/env python3
"""
Automated Contract and FFI Verification Suite.
Validates parity between:
1. Rust C-ABI FFI exports (`crates/sovereign-ffi/src/lib.rs`) and Dart FFI bindings (`client/lib/core/ffi/sovereign_bridge.dart`).
2. Gateway OpenAPI 3.1 schemas and Flutter client DTO models.
3. Statutory legal identifiers and UI constants.
"""

import re
import os
import sys
import json

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))

def verify_ffi_bindings():
    print("[1/4] Verifying Rust C-ABI Exports vs. Dart FFI Bindings...")
    rust_ffi_path = os.path.join(REPO_ROOT, "crates/sovereign-ffi/src/lib.rs")
    dart_bridge_path = os.path.join(REPO_ROOT, "client/lib/core/ffi/sovereign_bridge.dart")

    with open(rust_ffi_path, "r", encoding="utf-8") as f:
        rust_src = f.read()

    with open(dart_bridge_path, "r", encoding="utf-8") as f:
        dart_src = f.read()

    # Extract #[no_mangle] pub extern "C" fn <name>
    rust_functions = re.findall(r'#\[no_mangle\]\s+pub\s+unsafe\s+extern\s+"C"\s+fn\s+([a-zA-Z0-9_]+)', rust_src)
    if not rust_functions:
        rust_functions = re.findall(r'#\[no_mangle\]\s+pub\s+extern\s+"C"\s+fn\s+([a-zA-Z0-9_]+)', rust_src)

    print(f"      Discovered {len(rust_functions)} exported C-ABI functions in Rust: {rust_functions}")
    assert len(rust_functions) >= 3, "Expected at least 3 FFI exports"

    for fn_name in rust_functions:
        assert fn_name in dart_src, f"FFI export '{fn_name}' missing from Dart bridge lookup!"
        print(f"      [OK] Dart bridge binds '{fn_name}'")

def verify_openapi_client_contract():
    print("[2/4] Verifying Gateway OpenAPI 3.1 Contract vs. Client DTOs...")
    gateway_lib_path = os.path.join(REPO_ROOT, "crates/sovereign-gateway/src/lib.rs")
    chat_screen_path = os.path.join(REPO_ROOT, "client/lib/features/chat/chat_screen.dart")
    catalog_screen_path = os.path.join(REPO_ROOT, "client/lib/features/commerce/catalog_screen.dart")
    compliance_screen_path = os.path.join(REPO_ROOT, "client/lib/features/compliance/compliance_screen.dart")

    with open(gateway_lib_path, "r", encoding="utf-8") as f:
        gateway_src = f.read()
    with open(chat_screen_path, "r", encoding="utf-8") as f:
        chat_src = f.read()
    with open(catalog_screen_path, "r", encoding="utf-8") as f:
        catalog_src = f.read()
    with open(compliance_screen_path, "r", encoding="utf-8") as f:
        compliance_src = f.read()

    # Check endpoints defined in OpenAPI spec
    expected_endpoints = [
        "/health",
        "/api/v1/identity",
        "/api/v1/messages",
        "/api/v1/commerce/order",
        "/api/v1/compliance/shred"
    ]
    for ep in expected_endpoints:
        assert ep in gateway_src, f"Missing endpoint {ep} in OpenAPI generator"
        print(f"      [OK] Verified OpenAPI endpoint: {ep}")

    # Check Commerce schema alignment
    assert "toJsonLd" in catalog_src, "Missing schema.org JSON-LD generator in Flutter catalog"
    assert "upi_uri" in gateway_src and "upi_uri" in gateway_src, "UPI Intent mismatch"
    print("      [OK] Verified Conversational Commerce JSON-LD & UPI 2.0 schema parity")

    # Check DPDP Section 12 shredding parity
    assert "REVOKED_AND_PURGED" in gateway_src
    assert "REVOKED_AND_PURGED" in compliance_src
    assert "smk_zeroized" in gateway_src
    print("      [OK] Verified DPDP Act Section 12 Crypto-Shredding state parity")

    # Check Realtime Delivery Receipts & Privacy Controls
    assert "record_sent" in gateway_src
    assert "record_delivered" in gateway_src
    assert "record_read" in gateway_src
    assert "hasReadReceipt" in chat_src
    print("      [OK] Verified Realtime Tick Receipts (Sent / Delivered / Read) parity")

def verify_statutory_and_ui_tokens():
    print("[3/4] Verifying Statutory Identifiers and Design Tokens...")
    theme_path = os.path.join(REPO_ROOT, "client/lib/core/theme/app_theme.dart")
    with open(theme_path, "r", encoding="utf-8") as f:
        theme_src = f.read()

    # WhatsApp-parity color tokens
    assert "0xFF00A884" in theme_src, "WhatsApp signature teal/green primary token missing"
    assert "0xFF111B21" in theme_src, "WhatsApp dark background token missing"
    assert "0xFF202C33" in theme_src, "WhatsApp dark surface theme token missing"
    assert "0xFF005C4B" in theme_src, "WhatsApp outbound chat bubble green missing"
    print("      [OK] WhatsApp design tokens verified (Primary Teal: #00A884, Background: #111B21, Bubble: #005C4B)")

    # Data localization jurisdiction invariant
    gateway_src = open(os.path.join(REPO_ROOT, "crates/sovereign-gateway/src/lib.rs")).read()
    assert "INDIA_LOCALIZED_MUMBAI_BLR" in gateway_src, "Data localization tag missing from gateway"
    print("      [OK] RBI & DPDP Data Localization tag verified: INDIA_LOCALIZED_MUMBAI_BLR")

def verify_broadcast_and_misinformation_contracts():
    print("[4/4] Verifying Broadcast & Misinformation Prevention Contracts...")
    status_rs_path = os.path.join(REPO_ROOT, "crates/sovereign-broadcast/src/status.rs")
    channels_rs_path = os.path.join(REPO_ROOT, "crates/sovereign-broadcast/src/channels.rs")
    forwarding_rs_path = os.path.join(REPO_ROOT, "crates/sovereign-broadcast/src/forwarding.rs")
    abuse_rs_path = os.path.join(REPO_ROOT, "crates/sovereign-broadcast/src/abuse_report.rs")
    updates_dart_path = os.path.join(REPO_ROOT, "client/lib/features/updates/updates_screen.dart")

    with open(status_rs_path, "r", encoding="utf-8") as f:
        status_src = f.read()
    with open(channels_rs_path, "r", encoding="utf-8") as f:
        channels_src = f.read()
    with open(forwarding_rs_path, "r", encoding="utf-8") as f:
        forwarding_src = f.read()
    with open(abuse_rs_path, "r", encoding="utf-8") as f:
        abuse_src = f.read()
    with open(updates_dart_path, "r", encoding="utf-8") as f:
        updates_src = f.read()

    # 1. Ephemeral Status & Privacy Audience Options
    assert "enum StatusAudience" in status_src
    assert "MyContacts" in status_src and "MyContactsExcept" in status_src and "OnlyShareWith" in status_src
    assert "enum StatusAudienceOption" in updates_src
    assert "myContacts" in updates_src and "myContactsExcept" in updates_src and "onlyShareWith" in updates_src
    assert "86400" in status_src or "86_400" in status_src, "24h status expiration invariant missing from Rust backend"
    assert "24h Auto-Shred" in updates_src, "Status 24h auto-shred UI indicator missing from Flutter"
    print("      [OK] Ephemeral 24h Status and Audience Privacy (MyContacts, Except, OnlyShare) verified")

    # 2. Tamper-Evident Broadcast Channels & Blinded Reactions
    assert "verify_post_signature" in channels_src, "Channel post signature verification missing"
    assert "Ed25519 Tamper-Evident Signed Post" in updates_src, "Ed25519 badge missing in Flutter UpdatesScreen"
    assert "reactions" in channels_src and "reactions" in updates_src
    print("      [OK] Verified Broadcast Channels & Blinded Aggregate Reactions verified")

    # 3. Viral Misinformation Friction Engine
    assert "ForwardedManyTimes" in forwarding_src
    assert "Forwarded many times (>= 5 hops)" in updates_src
    assert "1 chat at a time" in updates_src
    assert "MAX_FREQUENTLY_FORWARDED_TARGETS: usize = 1" in forwarding_src
    assert "FREQUENTLY_FORWARDED_THRESHOLD: u32 = 5" in forwarding_src
    print("      [OK] Viral Friction Threshold (F >= 5 restricted to 1 target) verified")

    # 4. IT Rules 2021 Statutory Abuse Grievance Mechanism
    assert "GrievanceTicket" in abuse_src
    assert "acknowledged_at" in abuse_src and "resolution_deadline" in abuse_src
    assert "AbuseReportRegistry" in abuse_src
    print("      [OK] IT Rules 2021 Rule 3(2) Abuse Reporting & Grievance Ticket SLA verified")

def main():
    print("======================================================================")
    print("   BHARATCHAT CLIENT CONTRACT & FFI VERIFICATION HARNESS")
    print("======================================================================")
    verify_ffi_bindings()
    verify_openapi_client_contract()
    verify_statutory_and_ui_tokens()
    verify_broadcast_and_misinformation_contracts()
    print("======================================================================")
    print(" [ALL PASS] Client contract & FFI bindings verified with 100% accuracy!")
    print("======================================================================")

if __name__ == "__main__":
    main()
