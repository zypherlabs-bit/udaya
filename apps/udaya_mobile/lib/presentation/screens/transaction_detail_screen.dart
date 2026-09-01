import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';
import 'package:udaya_mobile/domain/entities/blockchain_entity.dart';
import 'package:udaya_mobile/core/constants/app_constants.dart';

class TransactionDetailScreen extends ConsumerWidget {
  final TransactionInfo tx;

  const TransactionDetailScreen({super.key, required this.tx});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      backgroundColor: UdayaTheme.bgDeep,
      appBar: AppBar(
        title: const Text('Transaction Detail'),
        backgroundColor: UdayaTheme.bgCard,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _buildSectionTitle('Transaction ID'),
          _buildInfoRow('TXID', tx.txid, monospace: true),
          const SizedBox(height: 16),
          _buildSectionTitle('Status'),
          _buildInfoRow('Confirmations', '${tx.confirmations}'),
          _buildInfoRow('Block Height', tx.blockHeight?.toString() ?? 'Unconfirmed'),
          _buildInfoRow('Timestamp', _formatTimestamp(tx.timestamp)),
          const SizedBox(height: 16),
          _buildSectionTitle('Amounts'),
          _buildInfoRow('Fee', '${(tx.fee / AppConstants.satoshisPerCoin).toStringAsFixed(8)} UDYA'),
          _buildInfoRow('Size', '${tx.sizeBytes} bytes'),
          _buildInfoRow('VSize', '${tx.vsize} vbytes'),
          const SizedBox(height: 16),
          _buildSectionTitle('Inputs (${tx.inputs.length})'),
          ...tx.inputs.map((input) => _buildInputTile(input)),
          const SizedBox(height: 16),
          _buildSectionTitle('Outputs (${tx.outputs.length})'),
          ...tx.outputs.map((output) => _buildOutputTile(output)),
        ],
      ),
    );
  }

  Widget _buildSectionTitle(String title) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Text(
        title,
        style: const TextStyle(
          color: UdayaTheme.accentColor,
          fontSize: 14,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }

  Widget _buildInfoRow(String label, String value, {bool monospace = false}) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(
              label,
              style: TextStyle(color: Colors.grey.shade400, fontSize: 13),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: TextStyle(
                color: Colors.white,
                fontSize: 13,
                fontFamily: monospace ? 'monospace' : null,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInputTile(TxInputInfo input) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: UdayaTheme.bgCard,
          borderRadius: BorderRadius.circular(12),
        ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('TXID: ${input.txid}', style: const TextStyle(color: Colors.white70, fontSize: 12, fontFamily: 'monospace')),
          const SizedBox(height: 4),
          Text('Vout: ${input.vout}', style: const TextStyle(color: Colors.white70, fontSize: 12)),
          if (input.address != null)
            Text('Address: ${input.address}', style: const TextStyle(color: Colors.white70, fontSize: 12)),
          if (input.value != null)
            Text('Value: ${(input.value! / AppConstants.satoshisPerCoin).toStringAsFixed(8)} UDYA', style: const TextStyle(color: Colors.white70, fontSize: 12)),
        ],
      ),
    );
  }

  Widget _buildOutputTile(TxOutputInfo output) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: UdayaTheme.bgCard,
          borderRadius: BorderRadius.circular(12),
        ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Address: ${output.address}', style: const TextStyle(color: Colors.white70, fontSize: 12)),
          const SizedBox(height: 4),
          Text('Value: ${output.valueUdya.toStringAsFixed(8)} UDYA', style: const TextStyle(color: Colors.white70, fontSize: 12)),
          Text('Script: ${output.scriptPubkeyHex}', style: const TextStyle(color: Colors.grey, fontSize: 11, fontFamily: 'monospace')),
        ],
      ),
    );
  }

  String _formatTimestamp(int timestamp) {
    final date = DateTime.fromMillisecondsSinceEpoch(timestamp * 1000);
    return '${date.year}-${date.month.toString().padLeft(2, '0')}-${date.day.toString().padLeft(2, '0')} ${date.hour.toString().padLeft(2, '0')}:${date.minute.toString().padLeft(2, '0')}';
  }
}
