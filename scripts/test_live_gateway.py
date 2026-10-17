#!/usr/bin/env python3
"""
Integration test for the live Sovereign Gateway network daemon.
Verifies real TCP sockets, HTTP/1.1 parsing, JSON payloads, and RFC 7807 responses.
"""

import subprocess
import time
import urllib.request
import urllib.error
import json
import os
import signal
import sys

def main():
    port = "8088"
    env = os.environ.copy()
    env["SOVEREIGN_HOST"] = "127.0.0.1"
    env["SOVEREIGN_PORT"] = port

    print("[1/5] Compiling and launching Sovereign Gateway daemon on 127.0.0.1:{}...".format(port))
    proc = subprocess.Popen(
        ["cargo", "run", "-p", "sovereign-gateway"],
        cwd="/home/bhasker-savaliya/hola",
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    base_url = f"http://127.0.0.1:{port}"
    time.sleep(2)

    # Wait for gateway to respond to /health
    ready = False
    for attempt in range(20):
        try:
            with urllib.request.urlopen(f"{base_url}/health", timeout=2) as response:
                if response.status == 200:
                    ready = True
                    break
        except Exception:
            time.sleep(0.5)

    if not ready:
        print("[FAIL] Gateway failed to bind and serve on port {}".format(port))
        proc.terminate()
        out, err = proc.communicate(timeout=5)
        print("STDOUT:", out)
        print("STDERR:", err)
        sys.exit(1)

    print("[2/5] Gateway daemon is LIVE. Testing /health and /api/v1/openapi.json...")
    with urllib.request.urlopen(f"{base_url}/health") as resp:
        health_data = json.loads(resp.read().decode())
        assert health_data["status"] == "UP"
        assert health_data["jurisdiction"] == "INDIA_LOCALIZED_MUMBAI_BLR"
        print(f"      Status: {health_data['status']}, Jurisdiction: {health_data['jurisdiction']}")

    with urllib.request.urlopen(f"{base_url}/api/v1/openapi.json") as resp:
        spec = json.loads(resp.read().decode())
        assert spec["openapi"] == "3.1.0"
        print(f"      OpenAPI: {spec['openapi']}, Title: {spec['info']['title']}")

    print("[3/5] Testing Identity lifecycle over HTTP...")
    test_did = "did:bharat:daemon_test_user_42"
    id_req_data = json.dumps({
        "did": test_did,
        "vk_hex": "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
        "timestamp": 1776000000
    }).encode("utf-8")

    req = urllib.request.Request(
        f"{base_url}/api/v1/identity",
        data=id_req_data,
        headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req) as resp:
        reg_resp = json.loads(resp.read().decode())
        assert resp.status == 201
        assert reg_resp["did"] == test_did
        print(f"      Registered DID: {reg_resp['did']}")

    with urllib.request.urlopen(f"{base_url}/api/v1/identity/{test_did}") as resp:
        get_resp = json.loads(resp.read().decode())
        assert resp.status == 200
        assert get_resp["status"] == "ACTIVE"
        print(f"      Query DID: ACTIVE")

    print("[4/5] Testing Conversational Commerce and UPI 2.0 Intent generation...")
    order_req_data = json.dumps({
        "order_id": "ORD-LIVE-DAEMON-999",
        "customer_did": test_did,
        "merchant_did": "did:bharat:merchant_varanasi",
        "items": [
            {"product_id": "item_banarasi", "name": "Pure Zari Silk Saree", "unit_price": 6000.0, "quantity": 1}
        ],
        "gst_rate": 5.0,
        "payee_vpa": "varanasisilk@okhdfcbank",
        "payee_name": "Varanasi Weavers Heritage",
        "timestamp": 1776000000
    }).encode("utf-8")

    req = urllib.request.Request(
        f"{base_url}/api/v1/commerce/order",
        data=order_req_data,
        headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req) as resp:
        order_resp = json.loads(resp.read().decode())
        assert resp.status == 201
        assert order_resp["total_amount_inr"] == 6300.0
        assert "upi://pay?pa=varanasisilk@okhdfcbank" in order_resp["upi_uri"]
        print(f"      Order created: {order_resp['order_id']}, Total: INR {order_resp['total_amount_inr']}")
        print(f"      UPI Intent: {order_resp['upi_uri']}")

    print("[5/5] Testing DPDP Act Section 12 Right-to-Erasure Crypto-Shredding...")
    shred_req_data = json.dumps({
        "did": test_did,
        "timestamp": 1776000500
    }).encode("utf-8")

    req = urllib.request.Request(
        f"{base_url}/api/v1/compliance/shred",
        data=shred_req_data,
        headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req) as resp:
        shred_resp = json.loads(resp.read().decode())
        assert resp.status == 200
        assert shred_resp["status"] == "REVOKED_AND_PURGED"
        assert shred_resp["smk_zeroized"] is True
        print(f"      Crypto-shredded DID: {shred_resp['did']}, SMK Zeroized: {shred_resp['smk_zeroized']}")

    # Verify subsequent lookup gives 404 Problem Details
    try:
        urllib.request.urlopen(f"{base_url}/api/v1/identity/{test_did}")
        print("[FAIL] Expected 404 for shredded DID")
        sys.exit(1)
    except urllib.error.HTTPError as e:
        assert e.code == 404
        prob = json.loads(e.read().decode())
        assert prob["title"] == "Identity Not Found"
        assert prob["jurisdiction"] == "INDIA_MUMBAI_BLR"
        print(f"      Post-shred RFC 7807 response confirmed: 404 {prob['title']}")

    print("\n[SUCCESS] Live Gateway Daemon network test passed with 100% fidelity!")
    proc.terminate()
    try:
        proc.wait(timeout=3)
    except Exception:
        proc.kill()

if __name__ == "__main__":
    main()
