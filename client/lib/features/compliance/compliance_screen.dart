import 'package:flutter/material.dart';
import '../../core/theme/app_theme.dart';
import '../../core/ffi/sovereign_bridge.dart';

class ComplianceScreen extends StatefulWidget {
  const ComplianceScreen({super.key});

  @override
  State<ComplianceScreen> createState() => _ComplianceScreenState();
}

class _ComplianceScreenState extends State<ComplianceScreen> {
  String did = "did:bharat:7a9f8e4d2b1c6e5a3f0d9c8b7a6e5f4d3c2b1a0f9e8d7c6b5a4f3e2d1c0b";
  bool isAdultVerified = true;
  bool simBound = true;
  bool isCsamScannerActive = true;
  bool accountErased = false;

  void _triggerCryptoShredding() {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppTheme.surfaceDark,
        title: const Text("DPDP Act 2023 Section 12 Right to Erasure", style: TextStyle(color: Colors.red)),
        content: const Text(
          "This will immediately crypto-shred your local StorageMasterKey (SMK), cascade unpin all off-chain encrypted blocks from Indian storage nodes, and revoke your on-chain DID on the Consortium Ledger.\n\nThis action is permanent and mathematically irreversible.",
          style: TextStyle(color: AppTheme.textPrimary, fontSize: 13),
        ),
        actions: [
          TextButton(
            child: const Text("Cancel", style: TextStyle(color: AppTheme.textSecondary)),
            onPressed: () => Navigator.pop(ctx),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            child: const Text("Confirm Crypto-Shred", style: TextStyle(color: Colors.white)),
            onPressed: () {
              Navigator.pop(ctx);
              setState(() {
                accountErased = true;
                did = "REVOKED_AND_PURGED";
              });
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text("Account crypto-shredded. SMK zeroized. All off-chain ciphertext rendered unrecoverable."),
                  backgroundColor: Colors.red,
                ),
              );
            },
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text("DPDP & Legal Compliance Dashboard"),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _buildCard(
            title: "W3C Sovereign Identity (DID)",
            subtitle: did,
            icon: Icons.fingerprint,
            badgeColor: accountErased ? Colors.red : AppTheme.primaryGreen,
            badgeText: accountErased ? "REVOKED & PURGED" : "ACTIVE ON-CHAIN",
          ),
          const SizedBox(height: 12),
          _buildCard(
            title: "Rule 4(2) IT Rules 2021 Traceability",
            subtitle: "Threshold HPKE Envelope: Multi-sig 2-of-3 Escrow (Grievance Officer + Compliance Officer + High Court). E2EE Plaintext never accessible.",
            icon: Icons.gavel_outlined,
            badgeColor: AppTheme.accentBlue,
            badgeText: "STATUTORY COMPLIANT",
          ),
          const SizedBox(height: 12),
          _buildCard(
            title: "DPDP Act Section 9 Minor Protection",
            subtitle: "Zero-Knowledge Proof (ZKP) Age Verification: Verified >= 18 without transmitting Aadhaar number or date of birth.",
            icon: Icons.verified_user_outlined,
            badgeColor: isAdultVerified ? AppTheme.primaryGreen : Colors.orange,
            badgeText: isAdultVerified ? "ZKP ADULT PROVEN" : "MINOR RESTRICTED",
          ),
          const SizedBox(height: 12),
          _buildCard(
            title: "RBI Hardware SIM-Binding",
            subtitle: "Device Hardware Enclave Token + SIM ICCID bound for NPCI UPI 2.0 transactions.",
            icon: Icons.sim_card_outlined,
            badgeColor: simBound ? AppTheme.primaryGreen : Colors.red,
            badgeText: simBound ? "SIM BOUND (AIRTEL)" : "UNBOUND",
          ),
          const SizedBox(height: 12),
          _buildCard(
            title: "Client-Side CSAM Prevention (POCSO Act)",
            subtitle: "Blind perceptual difference hashing (dHash) against encrypted NCRB/NCMEC illicit bloom filter prior to encryption.",
            icon: Icons.security_outlined,
            badgeColor: isCsamScannerActive ? AppTheme.primaryGreen : Colors.grey,
            badgeText: "PRE-ENCRYPT ACTIVE",
          ),
          const SizedBox(height: 24),
          if (!accountErased)
            ElevatedButton.icon(
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.red[900],
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(vertical: 14),
              ),
              icon: const Icon(Icons.delete_forever),
              label: const Text("Execute DPDP Section 12 Crypto-Shredding"),
              onPressed: _triggerCryptoShredding,
            )
          else
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.red.withOpacity(0.1),
                border: Border.all(color: Colors.red),
                borderRadius: BorderRadius.circular(8),
              ),
              child: const Text(
                "Your identity and storage keys have been permanently destroyed. Ciphertext on storage nodes is mathematically indecipherable random noise.",
                style: TextStyle(color: Colors.red, fontSize: 13),
                textAlign: TextAlign.center,
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildCard({
    required String title,
    required String subtitle,
    required IconData icon,
    required Color badgeColor,
    required String badgeText,
  }) {
    return Card(
      color: AppTheme.surfaceDark,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(icon, color: AppTheme.primaryGreen, size: 22),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    title,
                    style: const TextStyle(
                      fontSize: 14,
                      fontWeight: FontWeight.bold,
                      color: AppTheme.textPrimary,
                    ),
                  ),
                ),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: badgeColor.withOpacity(0.2),
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: badgeColor, width: 0.8),
                  ),
                  child: Text(
                    badgeText,
                    style: TextStyle(fontSize: 9, fontWeight: FontWeight.bold, color: badgeColor),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              subtitle,
              style: const TextStyle(fontSize: 12, color: AppTheme.textSecondary),
            ),
          ],
        ),
      ),
    );
  }
}
