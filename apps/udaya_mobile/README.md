# Udaya Mobile

Native mobile client for the Udaya Layer-1 blockchain. Built with Flutter, connecting to Udaya nodes via FFI and JSON-RPC for wallet, peer-to-peer networking, mining, and explorer functionality.

## Features

- **Futuristic Glassmorphism UI**: Premium frosted-glass interface with layered translucent cards, subtle blur effects, and elegant gradients
- **Full Wallet Support**: Create, restore, send, receive, and track UDAYA transactions
- **Block Explorer**: Browse blocks, transactions, and network statistics
- **Node Management**: Start/stop local Udaya node, monitor sync status, view logs
- **Mining Dashboard**: CPU mining with thread control, hashrate monitoring, and block reward tracking
- **Peer Management**: View connected peers, latency, and connection details
- **Testnet Support**: Switch between Mainnet and Testnet with data separation
- **Security Center**: Biometric authentication, app lock, encryption status
- **Android Biometric App Lock**: Fingerprint and face authentication with secure storage
- **Screenshot Protection**: Secure window flags on sensitive screens
- **Clipboard Protection**: Secure handling of sensitive copied data

## Design System

The app uses a custom futuristic design system built on glassmorphism principles:

- **GlassCard**: Translucent cards with backdrop blur and subtle borders
- **GlassPanel**: Larger frosted panels for complex content areas
- **GlassButton**: Minimal buttons with glassmorphic styling
- **GlassAppBar**: Transparent app bars with backdrop blur
- **GlassNavigationBar**: Bottom navigation with glass effects
- **MetricCard**: Animated metric displays with iconography
- **TransactionCard**: Elegant transaction display with visual status states
- **NetworkStatusCard**: Real-time network and sync status
- **NodeStatusCard**: Comprehensive node health monitoring
- **PeerCard**: Individual peer connection status
- **MiningCard**: Mining dashboard with hashrate and statistics
- **SecurityCard**: Security status indicators
- **StatusIndicator**: Pulsing status dots for active states
- **ProgressRing**: Animated circular progress indicators
- **AnimatedNumber**: Smooth number transitions

## Theme

- **Dark Theme**: Deep navy/black backgrounds with subtle gradients (default)
- **Light Theme**: Clean light theme with glassmorphic adaptations
- **Typography**: Inter font family with careful weight and spacing hierarchy
- **Colors**: Premium indigo/cyan/violet palette with status colors

## Requirements

- Flutter 3.24+
- Android SDK 29+ (Android 10)
- iOS 14+
- Udaya node or public RPC endpoint

## Setup

```bash
flutter pub get
flutter run
```

## Testing

```bash
flutter test
flutter analyze
```

## Build

```bash
flutter build apk --release
flutter build appbundle --release
```

## Architecture

```
Flutter UI
  ↓
Dart API Layer
  ↓
FFI/JNI Bridge
  ↓
Rust Udaya Core
```

The Flutter layer handles UI/UX and user interactions. All blockchain logic (consensus, wallet cryptography, P2P, UTXO, mining, security) remains in the Rust core, accessed through FFI/JNI.

## Security

- Android Keystore-backed secure storage for sensitive data
- Biometric authentication via Android Biometric API
- Never stores raw fingerprint or biometric data
- Secure storage for encryption keys and app preferences
- Screenshot protection via FLAG_SECURE
- Clipboard protection for sensitive information

## License

Proprietary. See LICENSE.
