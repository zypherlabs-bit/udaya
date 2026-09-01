class NotificationType {
  final String id;
  final String title;
  final String description;

  const NotificationType({
    required this.id,
    required this.title,
    required this.description,
  });

  static const transactionConfirmed = NotificationType(
    id: 'transaction_confirmed',
    title: 'Transaction Confirmed',
    description: 'Your transaction has been confirmed on the network',
  );

  static const nodeStatusChange = NotificationType(
    id: 'node_status_change',
    title: 'Node Status Change',
    description: 'Your node status has changed',
  );

  static const peerConnected = NotificationType(
    id: 'peer_connected',
    title: 'Peer Connected',
    description: 'A new peer has connected to your node',
  );

  static const peerDisconnected = NotificationType(
    id: 'peer_disconnected',
    title: 'Peer Disconnected',
    description: 'A peer has disconnected from your node',
  );

  static const blockFound = NotificationType(
    id: 'block_found',
    title: 'Block Found',
    description: 'Your miner found a new block',
  );

  static const difficultyChange = NotificationType(
    id: 'difficulty_change',
    title: 'Difficulty Change',
    description: 'Network difficulty has changed',
  );

  static const syncComplete = NotificationType(
    id: 'sync_complete',
    title: 'Sync Complete',
    description: 'Blockchain sync has completed',
  );

  static const syncStalled = NotificationType(
    id: 'sync_stalled',
    title: 'Sync Stalled',
    description: 'Blockchain sync has stalled',
  );

  static const securityAlert = NotificationType(
    id: 'security_alert',
    title: 'Security Alert',
    description: 'A security-related event was detected',
  );

  static const priceAlert = NotificationType(
    id: 'price_alert',
    title: 'Price Alert',
    description: 'UDYA price reached your target',
  );

  static const governanceProposal = NotificationType(
    id: 'governance_proposal',
    title: 'Governance Proposal',
    description: 'A new governance proposal is available for voting',
  );

  static const all = <NotificationType>[
    transactionConfirmed,
    nodeStatusChange,
    peerConnected,
    peerDisconnected,
    blockFound,
    difficultyChange,
    syncComplete,
    syncStalled,
    securityAlert,
    priceAlert,
    governanceProposal,
  ];
}

class AppNotification {
  final String id;
  final NotificationType type;
  final String title;
  final String body;
  final Map<String, dynamic>? payload;
  final DateTime timestamp;
  final bool isRead;

  AppNotification({
    required this.id,
    required this.type,
    required this.title,
    required this.body,
    this.payload,
    required this.timestamp,
    this.isRead = false,
  });

  AppNotification copyWith({
    String? id,
    NotificationType? type,
    String? title,
    String? body,
    Map<String, dynamic>? payload,
    DateTime? timestamp,
    bool? isRead,
  }) {
    return AppNotification(
      id: id ?? this.id,
      type: type ?? this.type,
      title: title ?? this.title,
      body: body ?? this.body,
      payload: payload ?? this.payload,
      timestamp: timestamp ?? this.timestamp,
      isRead: isRead ?? this.isRead,
    );
  }
}

class AppNotificationSettings {
  final bool enabled;
  final Map<String, bool> typeSettings;
  final bool soundEnabled;
  final bool vibrationEnabled;
  final bool showPreview;

  const AppNotificationSettings({
    this.enabled = true,
    this.typeSettings = const {},
    this.soundEnabled = true,
    this.vibrationEnabled = true,
    this.showPreview = true,
  });

  bool isTypeEnabled(String typeId) {
    return typeSettings[typeId] ?? true;
  }

  AppNotificationSettings copyWith({
    bool? enabled,
    Map<String, bool>? typeSettings,
    bool? soundEnabled,
    bool? vibrationEnabled,
    bool? showPreview,
  }) {
    return AppNotificationSettings(
      enabled: enabled ?? this.enabled,
      typeSettings: typeSettings ?? this.typeSettings,
      soundEnabled: soundEnabled ?? this.soundEnabled,
      vibrationEnabled: vibrationEnabled ?? this.vibrationEnabled,
      showPreview: showPreview ?? this.showPreview,
    );
  }
}
