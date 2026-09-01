import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';

class FeeSelector extends StatelessWidget {
  final int selectedFeeRate;
  final ValueChanged<int> onFeeRateChanged;

  const FeeSelector({
    super.key,
    required this.selectedFeeRate,
    required this.onFeeRateChanged,
  });

  static const List<FeePreset> presets = [
    FeePreset(name: 'Economy', rate: 1, description: '~30 min'),
    FeePreset(name: 'Normal', rate: 5, description: '~10 min'),
    FeePreset(name: 'Priority', rate: 15, description: '~5 min'),
    FeePreset(name: 'Custom', rate: -1, description: 'Set manually'),
  ];

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Transaction Fee',
          style: TextStyle(
            color: UdayaTheme.textPrimary,
            fontSize: 16,
            fontWeight: FontWeight.w600,
          ),
        ),
        const SizedBox(height: 12),
        Row(
          children: presets.map((preset) {
            final isSelected = preset.name == 'Custom'
                ? !presets.any((p) => p.rate == selectedFeeRate && p.name != 'Custom')
                : preset.rate == selectedFeeRate;
            return Expanded(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 4),
                child: InkWell(
                  onTap: () {
                    if (preset.name != 'Custom') {
                      onFeeRateChanged(preset.rate);
                    }
                  },
                  borderRadius: BorderRadius.circular(8),
                  child: Container(
                    padding: const EdgeInsets.symmetric(vertical: 10, horizontal: 4),
                    decoration: BoxDecoration(
                      color: isSelected
                          ? UdayaTheme.secondaryColor.withValues(alpha: 0.2)
                           : UdayaTheme.bgSurface,
                      borderRadius: BorderRadius.circular(8),
                      border: Border.all(
                        color: isSelected
                            ? UdayaTheme.secondaryColor
                            : UdayaTheme.textTertiary.withValues(alpha: 0.3),
                      ),
                    ),
                    child: Column(
                      children: [
                        Text(
                          preset.name,
                          style: TextStyle(
                            color: isSelected
                                ? UdayaTheme.secondaryColor
                                : UdayaTheme.textSecondary,
                            fontSize: 12,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                        const SizedBox(height: 2),
                        Text(
                          preset.description,
                          style: const TextStyle(
                            color: UdayaTheme.textTertiary,
                            fontSize: 10,
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            );
          }).toList(),
        ),
        const SizedBox(height: 12),
        Row(
          children: [
            const Text(
              'sat/vB:',
              style: TextStyle(color: UdayaTheme.textSecondary, fontSize: 14),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Slider(
                value: selectedFeeRate.toDouble().clamp(1, 100),
                min: 1,
                max: 100,
                divisions: 99,
                activeColor: UdayaTheme.secondaryColor,
                label: '$selectedFeeRate sat/vB',
                onChanged: (value) => onFeeRateChanged(value.round()),
              ),
            ),
            SizedBox(
              width: 50,
              child: Text(
                '$selectedFeeRate',
                style: const TextStyle(
                  color: UdayaTheme.textPrimary,
                  fontWeight: FontWeight.w600,
                ),
                textAlign: TextAlign.end,
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class FeePreset {
  final String name;
  final int rate;
  final String description;

  const FeePreset({
    required this.name,
    required this.rate,
    required this.description,
  });
}
