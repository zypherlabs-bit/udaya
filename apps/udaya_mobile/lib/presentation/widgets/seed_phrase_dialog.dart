import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';

class SeedPhraseDialog extends StatelessWidget {
  final List<String> seedWords;
  final bool showCopyWarning;

  const SeedPhraseDialog({
    super.key,
    required this.seedWords,
    this.showCopyWarning = true,
  });

  static Future<void> show(BuildContext context, List<String> seedWords) {
    return showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => SeedPhraseDialog(seedWords: seedWords),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      backgroundColor: UdayaTheme.bgCard,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      title: const Row(
        children: [
          Icon(Icons.warning_amber_rounded, color: UdayaTheme.warning),
          SizedBox(width: 12),
          Text(
            'Seed Phrase',
            style: TextStyle(color: UdayaTheme.textPrimary),
          ),
        ],
      ),
      content: SizedBox(
        width: double.maxFinite,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: UdayaTheme.warning.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: UdayaTheme.warning.withValues(alpha: 0.3)),
              ),
              child: const Text(
                'Never share your seed phrase. Anyone with access to it can steal your funds. Store it securely offline.',
                style: TextStyle(color: UdayaTheme.warning, fontSize: 13),
                textAlign: TextAlign.center,
              ),
            ),
            const SizedBox(height: 16),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: UdayaTheme.bgSurface,
                borderRadius: BorderRadius.circular(12),
              ),
              child: GridView.builder(
                shrinkWrap: true,
                gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                  crossAxisCount: 3,
                  childAspectRatio: 2.5,
                  crossAxisSpacing: 8,
                  mainAxisSpacing: 8,
                ),
                itemCount: seedWords.length,
                itemBuilder: (context, index) {
                  return Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: UdayaTheme.bgCard,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: UdayaTheme.textTertiary.withValues(alpha: 0.3)),
                    ),
                    child: Row(
                      children: [
                        Text(
                          '${index + 1}.',
                          style: const TextStyle(
                            color: UdayaTheme.textTertiary,
                            fontSize: 12,
                          ),
                        ),
                        const SizedBox(width: 4),
                        Expanded(
                          child: Text(
                            seedWords[index],
                            style: const TextStyle(
                              color: UdayaTheme.textPrimary,
                              fontSize: 13,
                              fontWeight: FontWeight.w500,
                            ),
                          ),
                        ),
                      ],
                    ),
                  );
                },
              ),
            ),
          ],
        ),
      ),
      actions: [
        OutlinedButton.icon(
          onPressed: () {
            final seed = seedWords.join(' ');
            Clipboard.setData(ClipboardData(text: seed));
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(
                content: Text('Seed phrase copied to clipboard'),
                behavior: SnackBarBehavior.floating,
              ),
            );
          },
          icon: const Icon(Icons.copy, size: 18),
          label: const Text('Copy'),
        ),
        ElevatedButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Done'),
        ),
      ],
    );
  }
}
