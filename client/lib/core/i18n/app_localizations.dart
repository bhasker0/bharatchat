import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Supported Languages under Eighth Schedule of the Constitution of India (DPDP Act 2023 Sec 5 Mandate)
enum AppLanguage {
  english,
  hindi,
  gujarati,
  tamil,
  marathi,
  bengali,
  telugu,
}

extension AppLanguageExtension on AppLanguage {
  String get code {
    switch (this) {
      case AppLanguage.english:
        return 'en';
      case AppLanguage.hindi:
        return 'hi';
      case AppLanguage.gujarati:
        return 'gu';
      case AppLanguage.tamil:
        return 'ta';
      case AppLanguage.marathi:
        return 'mr';
      case AppLanguage.bengali:
        return 'bn';
      case AppLanguage.telugu:
        return 'te';
    }
  }

  String get nativeName {
    switch (this) {
      case AppLanguage.english:
        return 'English';
      case AppLanguage.hindi:
        return 'हिन्दी (Hindi)';
      case AppLanguage.gujarati:
        return 'ગુજરાતી (Gujarati)';
      case AppLanguage.tamil:
        return 'தமிழ் (Tamil)';
      case AppLanguage.marathi:
        return 'मराठी (Marathi)';
      case AppLanguage.bengali:
        return 'বাংলা (Bengali)';
      case AppLanguage.telugu:
        return 'తెలుగు (Telugu)';
    }
  }
}

class AppTranslations {
  static const Map<AppLanguage, Map<String, String>> _strings = {
    AppLanguage.english: {
      'app_title': 'BharatChat Sovereign',
      'tab_chats': 'Chats',
      'tab_updates': 'Updates',
      'tab_calls': 'Calls (32)',
      'tab_catalog': 'Catalog',
      'tab_dpdp': 'DPDP / Law',
      'e2ee_notice': 'Messages are end-to-end encrypted. Rule 4(2) Originator Envelope is sealed to multi-sig judicial key.',
      'status_title': 'Status',
      'status_24h_shred': '24h Auto-Shred',
      'channels_title': 'Channels',
      'verified_merchant': 'Verified Merchant • E2EE Encrypted',
      'upi_pay_button': 'Pay via UPI 2.0',
      'crypto_shred_button': 'Execute DPDP Section 12 Crypto-Shredding',
      'crypto_shred_dialog_title': 'DPDP Act 2023 Section 12 Right to Erasure',
      'crypto_shred_dialog_body': 'This will immediately crypto-shred your local StorageMasterKey (SMK), cascade unpin all off-chain encrypted blocks from Indian storage nodes, and revoke your on-chain DID on the Consortium Ledger.',
      'backup_title': '64-Digit E2EE Encrypted Cloud Backup',
      'backup_subtitle': 'Argon2id + AES-256-GCM zero-knowledge archive strictly stored in Mumbai/Bengaluru.',
      'voice_note': 'Voice Note',
      'recording': 'Recording audio...',
    },
    AppLanguage.hindi: {
      'app_title': 'भारतचैट संप्रभु',
      'tab_chats': 'चैट',
      'tab_updates': 'अपडेट',
      'tab_calls': 'कॉल (32)',
      'tab_catalog': 'कैटलॉग',
      'tab_dpdp': 'डीपीडीपी / कानून',
      'e2ee_notice': 'संदेश एंड-टू-एंड एन्क्रिप्टेड हैं। नियम 4(2) प्रर्वतक लिफाफा न्यायिक बहु-हस्ताक्षर कुंजी से सीलबंद है।',
      'status_title': 'स्थिति',
      'status_24h_shred': '24 घंटे स्वतः नष्ट',
      'channels_title': 'चैनल',
      'verified_merchant': 'सत्यापित व्यापारी • E2EE एन्क्रिप्टेड',
      'upi_pay_button': 'UPI 2.0 द्वारा भुगतान करें',
      'crypto_shred_button': 'डीपीडीपी धारा 12 क्रिप्टो-श्रेड निष्पादित करें',
      'crypto_shred_dialog_title': 'डीपीडीपी अधिनियम 2023 धारा 12 मिटाने का अधिकार',
      'crypto_shred_dialog_body': 'यह तुरंत आपकी स्थानीय स्टोरेज मास्टर कुंजी (SMK) को नष्ट कर देगा और भारतीय भंडारण नोड्स से सभी ब्लॉक हटा देगा।',
      'backup_title': '64-अंकीय E2EE एन्क्रिप्टेड बैकअप',
      'backup_subtitle': 'Argon2id + AES-256-GCM शून्य-ज्ञान संग्रह केवल मुंबई/बेंगलुरु में संग्रहीत।',
      'voice_note': 'वॉयस नोट',
      'recording': 'ऑडियो रिकॉर्ड हो रहा है...',
    },
    AppLanguage.gujarati: {
      'app_title': 'ભારતચેટ સાર્વભૌમ',
      'tab_chats': 'ચેટ્સ',
      'tab_updates': 'અપડેટ્સ',
      'tab_calls': 'કોલ્સ (32)',
      'tab_catalog': 'કેટલોગ',
      'tab_dpdp': 'DPDP / કાયદો',
      'e2ee_notice': 'સંદેશાઓ એન્ડ-ટુ-એન્ડ એન્ક્રિપ્ટેડ છે. નિયમ 4(2) મૂળ લિફાફો ન્યાયિક કી સાથે સીલબંધ છે.',
      'status_title': 'સ્થિતિ',
      'status_24h_shred': '24 કલાકમાં સ્વતઃ નાશ',
      'channels_title': 'ચેનલો',
      'verified_merchant': 'ચકાસાયેલ વેપારી • E2EE એન્ક્રિપ્ટેડ',
      'upi_pay_button': 'UPI 2.0 દ્વારા ચૂકવણી કરો',
      'crypto_shred_button': 'DPDP કલમ 12 ક્રિપ્ટો-શ્રેડ ચલાવો',
      'crypto_shred_dialog_title': 'DPDP એક્ટ 2023 કલમ 12 ભૂંસી નાખવાનો અધિકાર',
      'crypto_shred_dialog_body': 'આ તમારી સ્થાનિક સ્ટોરેજ માસ્ટર કી (SMK) નો નાશ કરશે અને ભારતીય નોડ્સમાંથી તમામ ડેટા ભૂંસી નાખશે.',
      'backup_title': '64-અંકનું E2EE એન્ક્રિપ્ટેડ બેકઅપ',
      'backup_subtitle': 'Argon2id + AES-256-GCM મુંબઈ/બેંગલુરુમાં સુરક્ષિત ડેટા.',
      'voice_note': 'વોઈસ નોટ',
      'recording': 'ઓડિયો રેકોર્ડિંગ ચાલુ છે...',
    },
    AppLanguage.tamil: {
      'app_title': 'பாரத்சாட் இறையாண்மை',
      'tab_chats': 'அரட்டைகள்',
      'tab_updates': 'புதுப்பிப்புகள்',
      'tab_calls': 'அழைப்புகள் (32)',
      'tab_catalog': 'பட்டியல்',
      'tab_dpdp': 'DPDP / சட்டம்',
      'e2ee_notice': 'செய்திகள் ஆரம்பம் முதல் இறுதி வரை குறியாக்கம் செய்யப்பட்டுள்ளன.',
      'status_title': 'நிலை',
      'status_24h_shred': '24 மணிநேர தானியங்கி அழிப்பு',
      'channels_title': 'சேனல்கள்',
      'verified_merchant': 'சரிபார்க்கப்பட்ட வணிகர் • E2EE குறியாக்கம்',
      'upi_pay_button': 'UPI 2.0 மூலம் செலுத்தவும்',
      'crypto_shred_button': 'DPDP பிரிவு 12 தரவு அழிப்பைச் செயல்படுத்தவும்',
      'crypto_shred_dialog_title': 'DPDP சட்டம் 2023 பிரிவு 12 அழிக்கும் உரிமை',
      'crypto_shred_dialog_body': 'இது உங்களின் சேமிப்பக முதன்மை விசையை (SMK) உடனடியாக அழிக்கும்.',
      'backup_title': '64-இலக்க E2EE மறைகுறியாக்கப்பட்ட காப்புப்பிரதி',
      'backup_subtitle': 'மும்பை/பெங்களூருவில் மட்டுமே பாதுகாக்கப்பட்ட காப்பகம்.',
      'voice_note': 'குரல் குறிப்பு',
      'recording': 'ஆடியோ பதிவு செய்யப்படுகிறது...',
    },
    AppLanguage.marathi: {
      'app_title': 'भारतचॅट सार्वभौम',
      'tab_chats': 'गप्पा (चॅट्स)',
      'tab_updates': 'अपडेट्स',
      'tab_calls': 'कॉल्स (32)',
      'tab_catalog': 'कॅटलॉग',
      'tab_dpdp': 'DPDP / कायदा',
      'e2ee_notice': 'संदेश एंड-टू-एंड एनक्रिप्टेड आहेत. नियम 4(2) ओरिजनेटर लिफाफा सुरक्षित आहे.',
      'status_title': 'स्टेटस',
      'status_24h_shred': '24 तास स्वयं-नष्ट',
      'channels_title': 'चॅनेल्स',
      'verified_merchant': 'सत्यापित व्यापारी • E2EE एनक्रिप्टेड',
      'upi_pay_button': 'UPI 2.0 द्वारे पैसे द्या',
      'crypto_shred_button': 'DPDP कलम 12 क्रिप्टो-श्रेड करा',
      'crypto_shred_dialog_title': 'DPDP कायदा 2023 कलम 12 हटवण्याचा अधिकार',
      'crypto_shred_dialog_body': 'हे तुमची स्थानिक स्टोरेज मास्टर की नष्ट करेल आणि डेटा कायमचा हटवेल.',
      'backup_title': '64-अंकी E2EE एनक्रिप्टेड बॅकअप',
      'backup_subtitle': 'Argon2id + AES-256-GCM मुंबई/बंगळुरूमध्ये सुरक्षित संग्रह.',
      'voice_note': 'व्हॉइस नोट',
      'recording': 'ऑडिओ रेकॉर्डिंग सुरू आहे...',
    },
    AppLanguage.bengali: {
      'app_title': 'ভারতচ্যাট সার্বভৌম',
      'tab_chats': 'চ্যাট',
      'tab_updates': 'আপডেট',
      'tab_calls': 'কল (৩২)',
      'tab_catalog': 'ক্যাটালগ',
      'tab_dpdp': 'DPDP / আইন',
      'e2ee_notice': 'বার্তাগুলি এন্ড-টু-এন্ড এনক্রিপ্ট করা হয়েছে।',
      'status_title': 'স্ট্যাটাস',
      'status_24h_shred': '২৪ ঘণ্টায় স্বয়ংক্রিয় ধ্বংস',
      'channels_title': 'চ্যানেল',
      'verified_merchant': 'যাচাইকৃত ব্যবসায়ী • E2EE এনক্রিপ্ট করা',
      'upi_pay_button': 'UPI 2.0 দিয়ে পেমেন্ট করুন',
      'crypto_shred_button': 'DPDP ধারা ১২ ক্রিপ্টো-শ্রেড কার্যকর করুন',
      'crypto_shred_dialog_title': 'DPDP আইন ২০২৩ ধারা ১২ মুছে ফেলার অধিকার',
      'crypto_shred_dialog_body': 'এটি অবিলম্বে আপনার স্থানীয় স্টোরেজ মাস্টার কি ধ্বংস করবে।',
      'backup_title': '৬৪-সংখ্যার E2EE এনক্রিপ্ট করা ব্যাকআপ',
      'backup_subtitle': 'মুম্বাই/বেঙ্গালুরু ডেটা সেন্টারে সুরক্ষিত।',
      'voice_note': 'ভয়েস নোট',
      'recording': 'অডিও রেকর্ড হচ্ছে...',
    },
    AppLanguage.telugu: {
      'app_title': 'భారత్‌చాట్ సార్వభౌమ',
      'tab_chats': 'చాట్స్',
      'tab_updates': 'అప్‌డేట్స్',
      'tab_calls': 'కాల్స్ (32)',
      'tab_catalog': 'కాటలాగ్',
      'tab_dpdp': 'DPDP / చట్టం',
      'e2ee_notice': 'సందేశాలు ఎండ్-టు-ఎండ్ ఎన్‌క్రిప్ట్ చేయబడ్డాయి.',
      'status_title': 'స్టేటస్',
      'status_24h_shred': '24 గంటల్లో స్వయం తొలగింపు',
      'channels_title': 'ఛానెల్స్',
      'verified_merchant': 'ధృవీకరించబడిన వ్యాపారి • E2EE ఎన్‌క్రిప్టెడ్',
      'upi_pay_button': 'UPI 2.0 ద్వారా చెల్లించండి',
      'crypto_shred_button': 'DPDP సెక్షన్ 12 క్రిప్టో-ష్రెడ్ చేయండి',
      'crypto_shred_dialog_title': 'DPDP చట్టం 2023 సెక్షన్ 12 తొలగింపు హక్కు',
      'crypto_shred_dialog_body': 'ఇది మీ స్థానిక స్టోరేజ్ మాస్టర్ కీని వెంటనే నాశనం చేస్తుంది.',
      'backup_title': '64-అంకెల E2EE ఎన్‌క్రిప్టెడ్ బ్యాకప్',
      'backup_subtitle': 'ముంబై/బెంగళూరు డేటా సెంటర్‌లలో భద్రపరచబడింది.',
      'voice_note': 'వాయిస్ నోట్',
      'recording': 'ఆడియో రికార్డ్ అవుతోంది...',
    },
  };

  static String get(AppLanguage lang, String key) {
    return _strings[lang]?[key] ?? _strings[AppLanguage.english]?[key] ?? key;
  }
}

class LanguageNotifier extends Notifier<AppLanguage> {
  @override
  AppLanguage build() => AppLanguage.english;

  void setLanguage(AppLanguage lang) => state = lang;
}

final languageProvider = NotifierProvider<LanguageNotifier, AppLanguage>(LanguageNotifier.new);
