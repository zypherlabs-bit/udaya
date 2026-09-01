import 'dart:async';
import 'dart:convert';
import 'package:udaya_mobile/core/errors/app_exceptions.dart';

enum NodeEventType {
  blockConnected,
  blockDisconnected,
  peerConnected,
  peerDisconnected,
  syncProgress,
  walletBalanceChanged,
  transactionConfirmed,
  transactionReceived,
  miningBlockFound,
  nodeError,
}

class NodeEvent {
  final NodeEventType type;
  final Map<String, dynamic> data;
  final DateTime timestamp;

  NodeEvent({
    required this.type,
    required this.data,
    DateTime? timestamp,
  }) : timestamp = timestamp ?? DateTime.now();

  @override
  String toString() {
    return 'NodeEvent(${type.name}, data: $data)';
  }
}

class EventService {
  final StreamController<NodeEvent> _eventController =
      StreamController<NodeEvent>.broadcast();
  final List<void Function(NodeEvent)> _listeners = [];
  bool _isListening = false;

  Stream<NodeEvent> get eventStream => _eventController.stream;

  void registerEventCallback(dynamic nodeHandle) {
    if (_isListening) return;
    _isListening = true;
  }

  void addListener(void Function(NodeEvent) listener) {
    _listeners.add(listener);
  }

  void removeListener(void Function(NodeEvent) listener) {
    _listeners.remove(listener);
  }

  void handleNativeEvent(String eventTypeJson, String eventDataJson) {
    try {
      final eventType = NodeEventType.values.firstWhere(
        (e) => e.name == eventTypeJson,
        orElse: () => NodeEventType.nodeError,
      );

      final data = jsonDecode(eventDataJson) as Map<String, dynamic>;

      final event = NodeEvent(
        type: eventType,
        data: data,
      );

      _eventController.add(event);

      for (final listener in _listeners) {
        try {
          listener(event);
        } catch (e) {
          // Ignore listener errors
        }
      }
    } catch (e) {
      _eventController.addError(AppException(
        message: 'Failed to parse node event: $e',
        code: 'EVENT_PARSE_ERROR',
      ));
    }
  }

  Stream<NodeEvent> filterByType(NodeEventType type) {
    return eventStream.where((event) => event.type == type);
  }

  Stream<int> get syncProgressStream {
    return filterByType(NodeEventType.syncProgress).map((event) {
      final progress = event.data['progress'];
      if (progress is int) return progress;
      if (progress is double) return progress.toInt();
      return 0;
    });
  }

  Stream<Map<String, dynamic>> get peerEventsStream {
    final connected = filterByType(NodeEventType.peerConnected);
    final disconnected = filterByType(NodeEventType.peerDisconnected);
    return connected.asyncExpand((event) async* {
      yield event.data;
      await for (final e in disconnected) {
        yield e.data;
      }
    });
  }

  Stream<Map<String, dynamic>> get blockEventsStream {
    final connected = filterByType(NodeEventType.blockConnected);
    final disconnected = filterByType(NodeEventType.blockDisconnected);
    return connected.asyncExpand((event) async* {
      yield event.data;
      await for (final e in disconnected) {
        yield e.data;
      }
    });
  }

  Stream<Map<String, dynamic>> get transactionEventsStream {
    final confirmed = filterByType(NodeEventType.transactionConfirmed);
    final received = filterByType(NodeEventType.transactionReceived);
    return confirmed.asyncExpand((event) async* {
      yield event.data;
      await for (final e in received) {
        yield e.data;
      }
    });
  }

  Future<void> dispose() async {
    await _eventController.close();
    _listeners.clear();
    _isListening = false;
  }
}
