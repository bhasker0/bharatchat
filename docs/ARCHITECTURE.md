# BharatChat / SovereignComm — Technical Architecture & Threat Model

**Version**: 1.0.0-PROD  
**Classification**: Cryptographic Engineering & Regulatory Compliance Whitepaper  
**Jurisdiction**: Republic of India (IT Rules 2021, DPDP Act 2023, RBI Directives, CERT-In Directions)

---

## 1. System Overview

BharatChat (SovereignComm) is a decentralized, high-concurrency messaging and conversational commerce platform providing complete feature parity with WhatsApp while operating on zero-trust cryptographic foundations. It establishes:
1. **Mathematical Guarantees**: Plaintext messages, voice calls, and media chunks are permanently inaccessible to servers, cloud providers, and intermediate relays.
2. **Statutory First-Originator Traceability (Rule 4(2))**: Resolves originator identity under lawful court warrants using multi-sig threshold HPKE without breaking E2EE content secrecy.
3. **Irreversible Right to Erasure (DPDP Act Sec 12)**: Implements mathematical crypto-shredding where StorageMasterKey zeroization renders off-chain ciphertext permanently unrecoverable.
4. **Minor Protection (DPDP Act Sec 9)**: Zero-Knowledge Age Verification gates access without storing or transmitting Aadhaar numbers or birthdates.
5. **Economic Anti-Spam (Consortium Ledger)**: Micro-staking locks collateral on broadcast messages, slashing offenders upon community spam reporting.

---

## 2. High-Level System Architecture

```mermaid
graph TD
    subgraph Client Enclave ["Client Secure Enclave (Pixel / iPhone / PC)"]
        DID[W3C DID: did:bharat:pubkey]
        SMK[Storage Master Key: SMK]
        DR[Double Ratchet / MLS TreeKEM]
        CSAM[Pre-Encrypt CSAM Bloom Scanner]
        SIM[Hardware SIM Tokenizer]
    end

    subgraph Ephemeral Relay ["Zero-Retention Ephemeral Relay Mesh"]
        Router[In-Memory Async Router]
        Buffer[Strict TTL Staging Queue]
    end

    subgraph Consortium Ledger ["Permissioned Consortium Blockchain"]
        DIDReg[On-Chain DID Registry]
        ConsentLedger[DPDP Consent Ledger]
        StakingPool[Micro-Staking & Slashing Pool]
    end

    subgraph Offchain Storage ["Indian Localized Encrypted Storage"]
        Chunker[256KB AES-256-GCM Chunker]
        CIDs[BLAKE3 Content Addressing]
        Purge[Cascade Unpinning Engine]
    end

    subgraph Legal Escrow ["Rule 4(2) Threshold Legal Escrow"]
        RGO[Custodian 1: Grievance Officer]
        CCO[Custodian 2: Compliance Officer]
        Court[Custodian 3: High Court Escrow]
    end

    Client Enclave -->|Zero-Disk Encrypted Packets| Router
    Client Enclave -->|256KB Encrypted Chunks| Chunker
    Client Enclave -->|DID Registration & Stake| Consortium Ledger
    Client Enclave -->|Hardware SIM Binding| SIM
    Legal Escrow -.->|2-of-3 Warrant Decryption| DID
```

---

## 3. Cryptographic Specifications

### 3.1 Identity Layer (W3C DID)
- **DID Format**: `did:bharat:<hex-encoded-ed25519-verifying-key>`.
- **Key Generation**: Rooted in hardware secure enclaves (Android StrongBox / Apple Secure Enclave).
- **Sub-Device Multi-Device Provisioning**: Companion devices (Desktop, Tablet, Web) pair via QR-code ephemeral X25519 ECDH challenges (`crates/sovereign-multidevice`). Primary private keys are never exported across devices.

### 3.2 Rule 4(2) First-Originator Traceability without Breaking E2EE
When a message $M$ is composed:
1. Canonical Content Identifier is computed: $CID = \text{BLAKE3}(M)$.
2. An `OriginatorMetadata` token containing `(originator_did, CID, timestamp, nonce, signature)` is constructed.
3. The token is sealed into a `TraceabilityEnvelope` via Hybrid Public Key Encryption (HPKE) addressed to the legal multi-sig public key $PK_{\text{Legal}}$.
4. Intermediate forwarders attach a signed `ForwardHopProof` with the unchanged parent CID.
5. Inquiries require an order from a competent judicial authority. Two of three designated custodians ($RGO, CCO, \text{Court}$) combine their secret shares via Lagrange interpolation over Curve25519:
   $$SK_{\text{Legal}} = \sum_{j \in S} y_j \prod_{m \ne j} \frac{-x_m}{x_j - x_m} \pmod p$$
6. $SK_{\text{Legal}}$ decrypts *only* the flagged message's originator token, revealing the first creator without revealing message plaintext or accessing any other user's conversations.

### 3.3 Large Groups & Umbrella Communities (TreeKEM / RFC 9420 MLS)
- Groups with up to 2,048 members operate using Message Layer Security (`crates/sovereign-mls`).
- TreeKEM left-balanced trees enable $O(\log N)$ key updates on member joins or departures, completely eliminating the $O(N)$ battery and bandwidth tax of legacy pairwise double ratchets.
- Umbrella Communities partition membership into Admin-only Announcements and decentralized Topic Discussion channels.

### 3.4 WebRTC Group Calling (SFrame RFC 9605)
- Voice and video calls supporting up to 32 simultaneous participants use Secure Frame (SFrame) encryption (`crates/sovereign-calls`).
- Media RTP frames are encrypted directly in the client worker before transmission to Selective Forwarding Units (SFUs), preventing cloud eavesdropping.
- MLS epochs rotate SFrame room keys automatically whenever participants join or leave.

### 3.5 DPDP Act Section 12 Crypto-Shredding
- Off-chain files (images, audio notes, documents) are chunked into 256KB blocks and encrypted under a client-side `StorageMasterKey` (SMK).
- When a user requests account erasure:
  1. The client zeroizes the SMK in memory and secure hardware storage.
  2. The storage cluster cascades unpinning of all chunk CIDs.
  3. The on-chain DID is revoked on the consortium blockchain.
  4. Without the SMK, all off-chain ciphertext blocks exhibit pure uniform random noise (Shannon entropy $H(X) \ge 7.989$ bits/byte), rendering forensic recovery mathematically impossible.

---

## 4. Threat Model & Security Analysis (STRIDE)

| Threat Category | Attack Vector | Sovereign Mitigation |
|---|---|---|
| **Spoofing** | Impersonating senders or forging originators | Every message packet is Ed25519-signed by the sender DID; Originator envelope is signed over CID and timestamp. |
| **Tampering** | Modifying forwarded messages in transit | Ephemeral relay and recipients verify BLAKE3 canonical CID matches the sealed envelope. Altered text breaks the hash check. |
| **Repudiation** | Denying broadcast spam or harassment | Consortium ledger locks micro-stake (5 paise/recipient). Slashing occurs automatically on on-chain verified complaint thresholds. |
| **Information Disclosure** | Server compromise or rogue relay operators | Zero server retention; Ephemeral relay holds packets in volatile RAM only during transit TTL. All payloads are E2EE Double Ratchet or MLS encrypted. |
| **Denial of Service** | Sybil broadcast flooding | Mandatory micro-staking makes mass spam economically non-viable. Spam complaint rates $> 3\%$ slash 100% of collateral and trigger a 14-day broadcast suspension. |
| **Elevation of Privilege** | Rogue subpoena or compromised single official | 2-of-3 threshold legal escrow ensures no single entity (neither Grievance Officer, Compliance Officer, nor Court clerk alone) can decrypt originator tokens. |

---

## 5. Indian Regulatory Operations Manual

### 5.1 Resident Grievance Officer (RGO)
- **Mandate**: Rule 3(2) of IT Rules, 2021.
- **Workflow**:
  1. Receive grievance ticket via in-app portal or signed email.
  2. Acknowledge within 24 hours with unique complaint tracking ID.
  3. Resolve within 15 days (or 72 hours if involving non-consensual sexual content / Section 67A).
  4. Coordinate with CCO if judicial threshold key presentation is formally summoned.

### 5.2 Chief Compliance Officer (CCO)
- **Mandate**: Rule 4(1)(a) of IT Rules, 2021.
- **Qualifications**: Key managerial personnel or senior employee resident in India.
- **Responsibilities**:
  1. Custodian of Key Share #2 of the Rule 4(2) Legal Threshold Escrow.
  2. Validates validity of judicial orders issued under Section 69A of the IT Act before authorizing multi-sig signing.
  3. Oversees compliance with CERT-In 6-hour cybersecurity incident reporting window.

### 5.3 24x7 Nodal Contact Person
- **Mandate**: Rule 4(1)(b) of IT Rules, 2021.
- **Operational Availability**: Continuous 24/7 hotline and secure endpoint for law enforcement coordination.
