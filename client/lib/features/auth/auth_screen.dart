import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/theme/app_theme.dart';
import 'auth_state.dart';

class AuthScreen extends ConsumerStatefulWidget {
  const AuthScreen({super.key});

  @override
  ConsumerState<AuthScreen> createState() => _AuthScreenState();
}

class _AuthScreenState extends ConsumerState<AuthScreen> with SingleTickerProviderStateMixin {
  late TabController _tabController;

  // Controllers for Sign In
  final _signInPhoneController = TextEditingController();
  final _signInPinController = TextEditingController();

  // Controllers for Registration
  final _regNameController = TextEditingController();
  final _regPhoneController = TextEditingController();
  String _selectedState = 'Gujarat';
  bool _isSimBound = true;
  bool _consentAccepted = true;

  final List<String> _indianStates = const [
    'Gujarat',
    'Maharashtra',
    'Karnataka',
    'Tamil Nadu',
    'Delhi NCR',
    'Telangana',
    'Uttar Pradesh',
    'West Bengal',
    'Rajasthan',
    'Other (India)',
  ];

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    _signInPhoneController.dispose();
    _signInPinController.dispose();
    _regNameController.dispose();
    _regPhoneController.dispose();
    super.dispose();
  }

  void _handleSignIn() {
    final phone = _signInPhoneController.text.trim();
    if (phone.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Please enter your mobile number or DID.'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }
    ref.read(authProvider.notifier).signIn(
          phoneOrDid: phone,
          pin: _signInPinController.text.trim(),
        );
  }

  void _handleRegister() {
    final name = _regNameController.text.trim();
    final phone = _regPhoneController.text.trim();

    if (name.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Please enter your display name.'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    if (phone.length < 10) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Please enter a valid 10-digit mobile number.'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    if (!_consentAccepted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Please accept the DPDP Act 2023 Statutory Notice to proceed.'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    ref.read(authProvider.notifier).register(
          name: name,
          phone: phone,
          jurisdiction: '$_selectedState, India (ap-south-1)',
          isSimBound: _isSimBound,
        );
  }

  @override
  Widget build(BuildContext context) {
    final authState = ref.watch(authProvider);

    return Scaffold(
      backgroundColor: AppTheme.darkBackground,
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 480),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  // App Emblem & Header
                  Container(
                    width: 72,
                    height: 72,
                    decoration: BoxDecoration(
                      color: AppTheme.primaryGreen.withOpacity(0.15),
                      shape: BoxShape.circle,
                      border: Border.all(color: AppTheme.primaryGreen, width: 2),
                    ),
                    child: const Icon(
                      Icons.shield,
                      size: 40,
                      color: AppTheme.primaryGreen,
                    ),
                  ),
                  const SizedBox(height: 16),
                  const Text(
                    "BharatChat Sovereign",
                    style: TextStyle(
                      fontSize: 24,
                      fontWeight: FontWeight.bold,
                      color: AppTheme.textPrimary,
                      letterSpacing: 0.5,
                    ),
                  ),
                  const SizedBox(height: 6),
                  const Text(
                    "India's End-to-End Encrypted Sovereign Platform",
                    style: TextStyle(
                      fontSize: 13,
                      color: AppTheme.textSecondary,
                    ),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 12),
                  // Statutory Compliance Badge
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                    decoration: BoxDecoration(
                      color: AppTheme.surfaceDark,
                      borderRadius: BorderRadius.circular(20),
                      border: Border.all(color: AppTheme.primaryGreen.withOpacity(0.3)),
                    ),
                    child: const Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(Icons.lock_clock, size: 14, color: AppTheme.primaryGreen),
                        SizedBox(width: 6),
                        Flexible(
                          child: Text(
                            "DPDP Act 2023 • IT Rules 4(2) • Zero Cloud Retention",
                            textAlign: TextAlign.center,
                            style: TextStyle(fontSize: 11, color: AppTheme.primaryGreen, fontWeight: FontWeight.w500),
                          ),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 24),

                  // Error Banner
                  if (authState.errorMessage != null)
                    Container(
                      margin: const EdgeInsets.only(bottom: 16),
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: Colors.red.withOpacity(0.12),
                        borderRadius: BorderRadius.circular(8),
                        border: Border.all(color: Colors.red.withOpacity(0.5)),
                      ),
                      child: Row(
                        children: [
                          const Icon(Icons.error_outline, color: Colors.red, size: 20),
                          const SizedBox(width: 10),
                          Expanded(
                            child: Text(
                              authState.errorMessage!,
                              style: const TextStyle(color: Colors.red, fontSize: 13),
                            ),
                          ),
                        ],
                      ),
                    ),

                  // Tab Bar Card
                  Card(
                    color: AppTheme.surfaceDark,
                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
                    child: Padding(
                      padding: const EdgeInsets.all(20),
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          // Tab selector
                          Container(
                            height: 44,
                            decoration: BoxDecoration(
                              color: AppTheme.darkBackground,
                              borderRadius: BorderRadius.circular(10),
                            ),
                            child: TabBar(
                              controller: _tabController,
                              indicator: BoxDecoration(
                                borderRadius: BorderRadius.circular(10),
                                color: AppTheme.primaryGreen,
                              ),
                              labelColor: Colors.white,
                              unselectedLabelColor: AppTheme.textSecondary,
                              labelStyle: const TextStyle(fontWeight: FontWeight.bold, fontSize: 13),
                              tabs: const [
                                Tab(text: "Sign In"),
                                Tab(text: "Create Account"),
                              ],
                            ),
                          ),
                          const SizedBox(height: 20),

                          // Tab Content
                          AnimatedBuilder(
                            animation: _tabController,
                            builder: (context, _) {
                              return _tabController.index == 0
                                  ? _buildSignInForm(authState)
                                  : _buildRegisterForm(authState);
                            },
                          ),
                        ],
                      ),
                    ),
                  ),

                  const SizedBox(height: 16),
                  // Quick Demo Shortcut for 1-tap review
                  TextButton.icon(
                    icon: const Icon(Icons.bolt, color: AppTheme.primaryGreen, size: 18),
                    label: const Text(
                      "Instant Demo Access (Ramesh Patel • Surat)",
                      style: TextStyle(color: AppTheme.primaryGreen, fontSize: 13),
                    ),
                    onPressed: () {
                      ref.read(authProvider.notifier).setDemoUser('Ramesh Patel', '9876543210');
                    },
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildSignInForm(AuthState authState) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Text(
          "Registered Mobile Number or DID",
          style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
        ),
        const SizedBox(height: 6),
        TextField(
          controller: _signInPhoneController,
          keyboardType: TextInputType.phone,
          style: const TextStyle(color: AppTheme.textPrimary),
          decoration: InputDecoration(
            hintText: "9876543210 or did:bharat:...",
            hintStyle: TextStyle(color: AppTheme.textSecondary.withOpacity(0.5)),
            prefixIcon: const Icon(Icons.phone_iphone, color: AppTheme.primaryGreen, size: 20),
            filled: true,
            fillColor: AppTheme.darkBackground,
            border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: BorderSide.none),
          ),
        ),
        const SizedBox(height: 14),
        const Text(
          "Hardware Security PIN (Optional)",
          style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
        ),
        const SizedBox(height: 6),
        TextField(
          controller: _signInPinController,
          obscureText: true,
          keyboardType: TextInputType.number,
          style: const TextStyle(color: AppTheme.textPrimary),
          decoration: InputDecoration(
            hintText: "•••••• (6-digit enclave PIN)",
            hintStyle: TextStyle(color: AppTheme.textSecondary.withOpacity(0.5)),
            prefixIcon: const Icon(Icons.pin, color: AppTheme.primaryGreen, size: 20),
            filled: true,
            fillColor: AppTheme.darkBackground,
            border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: BorderSide.none),
          ),
        ),
        const SizedBox(height: 20),
        ElevatedButton(
          style: ElevatedButton.styleFrom(
            backgroundColor: AppTheme.primaryGreen,
            foregroundColor: Colors.white,
            padding: const EdgeInsets.symmetric(vertical: 14),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
          ),
          onPressed: authState.isLoading ? null : _handleSignIn,
          child: authState.isLoading
              ? const SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                )
              : const Text("Verify & Sign In", style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold)),
        ),
      ],
    );
  }

  Widget _buildRegisterForm(AuthState authState) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Text(
          "Full Name / Business Display Name",
          style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
        ),
        const SizedBox(height: 6),
        TextField(
          controller: _regNameController,
          style: const TextStyle(color: AppTheme.textPrimary),
          decoration: InputDecoration(
            hintText: "e.g. Surat Heritage Silk or Ramesh Patel",
            hintStyle: TextStyle(color: AppTheme.textSecondary.withOpacity(0.5)),
            prefixIcon: const Icon(Icons.person_outline, color: AppTheme.primaryGreen, size: 20),
            filled: true,
            fillColor: AppTheme.darkBackground,
            border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: BorderSide.none),
          ),
        ),
        const SizedBox(height: 14),
        const Text(
          "Mobile Number (+91 India)",
          style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
        ),
        const SizedBox(height: 6),
        TextField(
          controller: _regPhoneController,
          keyboardType: TextInputType.phone,
          style: const TextStyle(color: AppTheme.textPrimary),
          decoration: InputDecoration(
            prefixText: "+91 ",
            prefixStyle: const TextStyle(color: AppTheme.primaryGreen, fontWeight: FontWeight.bold),
            hintText: "9876543210",
            hintStyle: TextStyle(color: AppTheme.textSecondary.withOpacity(0.5)),
            filled: true,
            fillColor: AppTheme.darkBackground,
            border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: BorderSide.none),
          ),
        ),
        const SizedBox(height: 14),
        const Text(
          "Statutory Data Residency / State",
          style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
        ),
        const SizedBox(height: 6),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12),
          decoration: BoxDecoration(
            color: AppTheme.darkBackground,
            borderRadius: BorderRadius.circular(8),
          ),
          child: DropdownButton<String>(
            value: _selectedState,
            isExpanded: true,
            underline: const SizedBox(),
            dropdownColor: AppTheme.surfaceDark,
            style: const TextStyle(color: AppTheme.textPrimary, fontSize: 14),
            items: _indianStates.map((st) {
              return DropdownMenuItem(value: st, child: Text(st));
            }).toList(),
            onChanged: (val) {
              if (val != null) setState(() => _selectedState = val);
            },
          ),
        ),
        const SizedBox(height: 14),
        // SIM Binding Toggle
        Container(
          padding: const EdgeInsets.all(10),
          decoration: BoxDecoration(
            color: AppTheme.darkBackground,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Row(
            children: [
              const Icon(Icons.sim_card, color: AppTheme.primaryGreen, size: 20),
              const SizedBox(width: 10),
              const Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text("DoT SIM Binding", style: TextStyle(color: AppTheme.textPrimary, fontSize: 13, fontWeight: FontWeight.bold)),
                    Text("Bind DID to device secure enclave ICCID", style: TextStyle(color: AppTheme.textSecondary, fontSize: 11)),
                  ],
                ),
              ),
              Switch(
                value: _isSimBound,
                activeColor: AppTheme.primaryGreen,
                onChanged: (v) => setState(() => _isSimBound = v),
              ),
            ],
          ),
        ),
        const SizedBox(height: 10),
        // Consent Checkbox
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Checkbox(
              value: _consentAccepted,
              activeColor: AppTheme.primaryGreen,
              onChanged: (v) => setState(() => _consentAccepted = v ?? false),
            ),
            const Expanded(
              child: Text(
                "I agree to the DPDP Act 2023 Section 6 Notice. My data remains localized in India with right to erasure.",
                style: TextStyle(fontSize: 11, color: AppTheme.textSecondary),
              ),
            ),
          ],
        ),
        const SizedBox(height: 16),
        ElevatedButton(
          style: ElevatedButton.styleFrom(
            backgroundColor: AppTheme.primaryGreen,
            foregroundColor: Colors.white,
            padding: const EdgeInsets.symmetric(vertical: 14),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
          ),
          onPressed: authState.isLoading ? null : _handleRegister,
          child: authState.isLoading
              ? const SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                )
              : const Text("Register Sovereign DID", style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold)),
        ),
      ],
    );
  }
}
