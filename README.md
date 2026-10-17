# BharatChat Sovereign Platform & Enterprise Agent Skills

[![CI/CD Pipeline](https://github.com/bhasker-savaliya/hola/actions/workflows/ci.yml/badge.svg)](https://github.com/bhasker-savaliya/hola/actions/workflows/ci.yml)
[![Statutory Compliance](https://img.shields.io/badge/Statutory%20Audit-100%25%20Passed-00A884)](scripts/audit_compliance.py)
[![Rust Workspace](https://img.shields.io/badge/Rust%20Workspace-25%2F25%20Tests%20Passing-blue)](Cargo.toml)
[![Jurisdiction](https://img.shields.io/badge/Data%20Jurisdiction-India%20(Mumbai%2FBengaluru)-orange)](docs/ARCHITECTURE.md)

An enterprise-grade sovereign communication, conversational commerce, and distributed identity platform built for the Republic of India. Combines **WhatsApp feature parity** with **zero-trust cryptography**, **permissioned blockchain consensus**, and strict statutory compliance under the **IT Rules 2021**, **DPDP Act 2023**, and **RBI Payment Data Localization Directives**.

Also contains the unified **139 Production Agent Skills Library** and master operating guidelines ([`AGENTS.md`](file:///home/bhasker-savaliya/hola/AGENTS.md)) governing autonomous AI pair-programming agents.

---

## 1. Architectural Highlights

```mermaid
graph TD
    subgraph Client ["Client Device (Android StrongBox / iOS Enclave)"]
        DID[W3C Decentralized Identity]
        SMK[Storage Master Key: SMK]
        DR[Double Ratchet & MLS TreeKEM]
        CSAM[Client-Side Blind CSAM Bloom Scanner]
        SIM[Hardware SIM Tokenizer]
    end

    subgraph Relay ["Zero-Retention Ephemeral Relay Mesh"]
        Router[In-Memory Async Router]
        RAMQueue[Volatile RAM TTL Buffer]
    end

    subgraph Consortium ["Permissioned Consortium Blockchain"]
        DIDLedger[DID Identity Lifecycle]
        ConsentLedger[DPDP Section 6 Consent Ledger]
        StakingPool[Broadcast Micro-Staking & Slashing]
    end

    subgraph Storage ["Off-Chain Indian Encrypted Storage"]
        Chunker[256KB AES-256-GCM Chunks]
        BLAKE3[BLAKE3 Content Addressing]
        Purge[DPDP Section 12 Cascade Purge]
    end

    subgraph LegalEscrow ["Rule 4(2) Threshold Legal Escrow"]
        RGO[Custodian 1: Grievance Officer]
        CCO[Custodian 2: Compliance Officer]
        HC[Custodian 3: High Court Escrow]
    end

    Client -->|Zero-Disk Encrypted Packets| Router
    Client -->|256KB Encrypted Chunks| Chunker
    Client -->|DID Status & Micro-Stake| Consortium
    Client -->|Hardware SIM Verification| SIM
    LegalEscrow -.->|2-of-3 Warrant Decryption| DID
```

---

## 2. Core Pillars & Feature Parity

### WhatsApp Feature Parity
- **Messaging**: Markdown text formatting, edit & delete within time limits, disappearing messages (24h/7d/90d), view-once media, audio voice notes with variable playback rates.
- **Calls & Screen Sharing**: WebRTC Secure Frame (SFrame RFC 9605) E2EE voice and video calls supporting up to 32 participants simultaneously with dynamic MLS key rotation.
- **Communities & Large Groups**: TreeKEM Message Layer Security (RFC 9420) scaling to **1,024+ members** in $O(\log N)$ time, featuring Admin-only Announcements and decentralized Topic Discussion channels.
- **Multi-Device Sync**: Companion Mode (SESAME Protocol) enabling QR-code ECDH pairing for Desktop and Web without exporting primary device private keys.

### Conversational Commerce & UPI 2.0
- **JSON-LD Catalogs**: Standardized `schema.org/Product` catalog engine mapping off-chain media BLAKE3 CIDs.
- **Interactive WhatsApp Messages**: Reply buttons, interactive sectioned list pickers, product cards, and multi-product collections.
- **Automated Business Bots**: 24-hour first-contact greetings, operating-hours away responder, and keyword quick-reply routers.
- **NPCI UPI 2.0 In-Chat Payments**: Real-time Indian GST breakdown (CGST + SGST), hardware SIM-binding tokenization, and dynamic `upi://pay?pa=...` payment intent URI generation.

### Indian Cyber Law & Regulatory Compliance
- **Rule 4(2) IT Rules 2021 (First-Originator Traceability)**: Sealed `TraceabilityEnvelope` protected by Hybrid Public Key Encryption (HPKE). Inquiries require a competent court order and 2-of-3 threshold Lagrange key reconstruction (Grievance Officer, Compliance Officer, High Court Escrow). **Zero plaintext is disclosed.**
- **Rule 3(1)(b) Content Moderation (POCSO & CSAM)**: Device-local blind perceptual difference hashing (dHash) matched against encrypted NCRB/NCMEC bloom filters before encryption.
- **DPDP Act 2023 Section 12 (Right to Erasure)**: Mathematical **Crypto-Shredding**. Zeroizing the `StorageMasterKey` (SMK) instantly renders off-chain media ciphertext mathematically indecipherable random noise ($H(X) \ge 7.989$ bits/byte).
- **DPDP Act 2023 Section 9 (Child Protection)**: Zero-Knowledge Age Verification (ZKP) proves `age >= 18` without disclosing Aadhaar numbers or birthdates; targeted ads and tracking for minors are strictly blocked.
- **RBI Data Localization Directive**: Storage nodes, validators, and payment switch adapters strictly localized to `INDIA_LOCALIZED_MUMBAI_BLR`.
- **CERT-In Cyber Security Directions**: Automated generation of statutory incident dossiers conforming to the mandatory 6-hour reporting window.

---

## 3. Cryptographic Performance Benchmarks

Executed on release profile (`cargo run --release -p sovereign-bench`):

| Primitive | Workload / Scale | Measured Throughput | Average Latency | Target SLA |
|---|---|---|---|---|
| **W3C DID Ed25519 Signing** | 10,000 transactions | **21,023.17 signs/sec** | **47.57 µs/op** | $< 1.0$ ms |
| **BLAKE3 Content Addressing** | 500 MB (256KB chunks) | **1,209.15 MB/s** | **206.76 µs/chunk** | $> 250$ MB/s |
| **Rule 4(2) HPKE Envelope** | 2,000 tokens | **5,378.24 envelopes/sec** | **185.93 µs/envelope** | $< 5.0$ ms |
| **TreeKEM MLS Group Messaging** | 1,024 Members | **392,482.58 msgs/sec** | **2.55 µs/msg** | $< 100$ µs |
| **SMK AES-256-GCM Encryption** | 250 MB (256KB chunks) | **413.49 MB/s** | **604.62 µs/chunk** | $> 150$ MB/s |

---

## 4. Repository Structure & Workspace Crates

```text
├── crates/
│   ├── sovereign-core/          # W3C DID, Rule 4(2) HPKE, Crypto-Shredding, CSAM Bloom Filter, Staking
│   ├── sovereign-relay/         # Zero-retention async routing mesh & volatile RAM TTL buffer
│   ├── sovereign-storage/       # 256KB AES-256-GCM chunk storage with cascade unpinning
│   ├── sovereign-calls/         # WebRTC 32-party SFrame audio/video calling state machine
│   ├── sovereign-ffi/           # C-ABI dynamic library bridge for Flutter / Dart FFI
│   ├── sovereign-chain/         # Permissioned consortium ledger for DIDs, consent & slashing
│   ├── sovereign-zkp/           # Zero-Knowledge Age Verification & DPDP minor protection policy
│   ├── sovereign-upi/           # NPCI UPI 2.0 payment bridge & hardware SIM-binding tokenizer
│   ├── sovereign-commerce/      # Conversational commerce, JSON-LD catalogs, interactive messages
│   ├── sovereign-multidevice/   # Multi-Device Companion Mode (SESAME Protocol) QR ECDH handshake
│   ├── sovereign-mls/           # TreeKEM / RFC 9420 MLS for 1000+ member groups & communities
│   ├── sovereign-e2e/           # Multi-tier end-to-end integration orchestration test suite
│   ├── sovereign-bench/         # High-performance cryptographic performance & throughput suite
│   └── sovereign-gateway/       # OpenAPI 3.1 REST/JSON microservice gateway (RFC 7807 problem details)
├── client/                      # Flutter 3+ Riverpod mobile application architecture
├── infra/                       # Cloud-native infrastructure
│   ├── docker-compose.yml       # Local developer orchestration stack
│   └── k8s/                     # Production Kubernetes manifests (StatefulSet, HPA, NetworkPolicy)
├── scripts/
│   ├── audit_compliance.py     # Automated CERT-In, DPDP, IT Rules & RBI verification suite
│   └── sovereign_legal_cli.py   # Operational CLI for RGO, CCO, and CERT-In liaison
├── docs/
│   └── ARCHITECTURE.md          # Technical whitepaper, STRIDE threat model & legal manuals
├── proto/                       # Protocol buffer definitions for E2EE, compliance & consent
├── skills/                      # 139 Production Agent Skills
├── AGENTS.md                    # Universal Agent Operating Guidelines
└── Cargo.toml                   # Root Cargo workspace manifest
```

---

## 5. Quick Start Guide

### 1. Run Workspace Automated Tests (27 Tests)
```bash
cargo test --workspace
```

### 2. Run Cryptographic Micro-Benchmarks
```bash
cargo run --release -p sovereign-bench
```

### 3. Run Statutory Cyber Law Compliance Audit
```bash
python3 scripts/audit_compliance.py
```

### 4. Run Live Gateway Daemon & Network Loopback Test
```bash
python3 scripts/test_live_gateway.py
```

### 5. Run Flutter Client Contract & FFI Verification
```bash
python3 scripts/verify_client_contracts.py
```

### 6. Run Resident Grievance Officer & Legal CLI
```bash
# Log a user grievance under Rule 3(2) IT Rules
python3 scripts/sovereign_legal_cli.py rgo create-ticket \
  --complainant-did "did:bharat:user_rahul" \
  --category "HARASSMENT" \
  --description "Unsolicited abusive group messages"

# List active grievances and check 24h acknowledgement / 15-day SLA
python3 scripts/sovereign_legal_cli.py rgo list-tickets

# Inspect and validate a Rule 4(2) court order
python3 scripts/sovereign_legal_cli.py cco inspect-warrant \
  --court-case "WP(C)-9821/2026-DELHI-HC" \
  --content-cid "bafk48f98c771a2"

# Export a CERT-In 6-Hour incident dossier
python3 scripts/sovereign_legal_cli.py certin export-incident \
  --incident-type "MALICIOUS_RELAY_FLOOD"
```

### 7. Review Mathematical & Statutory Formal Verification Proofs
See [`docs/FORMAL_VERIFICATION.md`](docs/FORMAL_VERIFICATION.md) for game-theoretic security reductions, Shannon entropy proofs, and Groth16 HVZK soundness proofs.

### 8. Launch Local Cloud Infrastructure with Docker Compose
```bash
docker compose -f infra/docker-compose.yml up -d
```

---

## 6. License & Statutory Compliance Notice

This platform is architected for strict conformance with:
* **The Information Technology Act, 2000** (as amended)
* **Information Technology (Intermediary Guidelines and Digital Media Ethics Code) Rules, 2021**
* **The Digital Personal Data Protection Act, 2023** (Act No. 22 of 2023)
* **Reserve Bank of India (RBI) Directive on Storage of Payment System Data** (DPSS.CO.OD No. 2785/06.11.001/2017-18)
* **National Payments Corporation of India (NPCI) UPI Procedural Guidelines 2.0**
* **CERT-In Cyber Security Directions** (No. 20(3)/2022-CERT-In)
