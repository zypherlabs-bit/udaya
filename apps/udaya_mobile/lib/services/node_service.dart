import 'dart:async';
import 'package:udaya_mobile/domain/entities/node_entity.dart';

class NodeService {
  bool _running = false;
  StreamController<NodeStatus>? _statusController;
  Timer? _pollingTimer;

  Future<void> startNode({
    String? configPath,
    bool? enableMining,
    int? minerThreads,
  }) async {
    if (_running) return;
    _running = true;
    _startStatusPolling();
  }

  Future<void> stopNode() async {
    if (!_running) return;
    _stopStatusPolling();
    _running = false;
  }

  Future<NodeStatus> getStatus() async {
    return NodeStatus(
      status: _running ? 'running' : 'stopped',
      blockHeight: 0,
      bestBlockHash: '',
      chainwork: '',
      difficulty: 0.0,
      peerCount: 0,
      syncPercentage: 0,
      lastBlockTime: null,
      network: 'mainnet',
      cpuUsage: 0.0,
      memoryUsage: 0.0,
      diskUsageBytes: 0,
    );
  }

  Future<int> getBlockHeight() async {
    return 0;
  }

  Stream<NodeStatus> get statusStream {
    _statusController ??= StreamController<NodeStatus>.broadcast();
    return _statusController!.stream;
  }

  void _startStatusPolling() {
    _pollingTimer = Timer.periodic(const Duration(seconds: 2), (_) async {
      try {
        final status = await getStatus();
        _statusController?.add(status);
      } catch (e) {
        // ignore
      }
    });
  }

  void _stopStatusPolling() {
    _pollingTimer?.cancel();
    _pollingTimer = null;
  }

  bool get isNodeRunning => _running;

  Future<void> dispose() async {
    await stopNode();
    await _statusController?.close();
    _statusController = null;
  }
}
