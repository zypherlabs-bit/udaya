import 'package:firebase_core/firebase_core.dart' as firebase_core;

class FirebaseOptions {
  static const String apiKey = 'YOUR_API_KEY';
  static const String appId = 'YOUR_APP_ID';
  static const String messagingSenderId = 'YOUR_SENDER_ID';
  static const String projectId = 'YOUR_PROJECT_ID';
  static const String storageBucket = 'YOUR_STORAGE_BUCKET';

  static firebase_core.FirebaseOptions get currentPlatform {
    return firebase_core.FirebaseOptions(
      apiKey: apiKey,
      appId: appId,
      messagingSenderId: messagingSenderId,
      projectId: projectId,
      storageBucket: storageBucket,
    );
  }
}
