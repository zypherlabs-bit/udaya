import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import 'package:udaya_mobile/core/theme/app_theme.dart';

class HashrateChart extends StatelessWidget {
  final List<DateTime> timestamps;
  final List<double> hashrates;

  const HashrateChart({
    super.key,
    required this.timestamps,
    required this.hashrates,
  });

  @override
  Widget build(BuildContext context) {
    if (hashrates.isEmpty) {
      return const SizedBox(
        height: 200,
        child: Center(child: Text('No hashrate data available')),
      );
    }

    return SizedBox(
      height: 200,
      child: LineChart(
        LineChartData(
          lineBarsData: [
            LineChartBarData(
              spots: hashrates
                  .asMap()
                  .entries
                  .map((entry) => FlSpot(entry.key.toDouble(), entry.value))
                  .toList(),
              isCurved: true,
              color: UdayaTheme.secondaryColor,
              barWidth: 3,
              dotData: const FlDotData(show: false),
              belowBarData: BarAreaData(
                show: true,
                color: UdayaTheme.secondaryColor.withValues(alpha: 0.1),
              ),
            ),
          ],
          titlesData: FlTitlesData(
            bottomTitles: AxisTitles(
              sideTitles: SideTitles(
                showTitles: true,
                interval: timestamps.length / 6,
                getTitlesWidget: (value, meta) {
                  final index = value.toInt();
                  if (index >= 0 && index < timestamps.length) {
                    final time = timestamps[index];
                    return Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Text(
                        '${time.hour}:${time.minute.toString().padLeft(2, '0')}',
                        style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 10),
                      ),
                    );
                  }
                  return const SizedBox.shrink();
                },
              ),
            ),
            leftTitles: AxisTitles(
              sideTitles: SideTitles(
                showTitles: true,
                interval: hashrates.isEmpty ? 0 : (hashrates.reduce((a, b) => a > b ? a : b) / 3),
                getTitlesWidget: (value, meta) => Text(
                  value.toInt().toString(),
                  style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 10),
                ),
              ),
            ),
            topTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
            rightTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
          ),
          borderData: FlBorderData(show: false),
          gridData: FlGridData(
            show: true,
            drawVerticalLine: false,
            horizontalInterval: hashrates.isEmpty ? 0 : (hashrates.reduce((a, b) => a > b ? a : b) / 3),
            getDrawingHorizontalLine: (value) => FlLine(
              color: UdayaTheme.bgSurface,
              strokeWidth: 1,
            ),
          ),
        ),
      ),
    );
  }
}

class PeerDistributionChart extends StatelessWidget {
  final List<MapEntry<String, int>> topCountries;
  final List<MapEntry<String, int>> topAsns;

  const PeerDistributionChart({
    super.key,
    required this.topCountries,
    required this.topAsns,
  });

  @override
  Widget build(BuildContext context) {
    final data = topCountries.isEmpty ? topAsns : topCountries;
    if (data.isEmpty) {
      return const SizedBox(
        height: 200,
        child: Center(child: Text('No peer data available')),
      );
    }

    final maxValue = data.map((e) => e.value).reduce((a, b) => a > b ? a : b);

    return SizedBox(
      height: 200,
      child: BarChart(
        BarChartData(
          barGroups: data
              .asMap()
              .entries
              .map((entry) {
                final index = entry.key;
                final item = entry.value;
                return BarChartGroupData(
                  x: index,
                  barRods: [
                    BarChartRodData(
                      toY: item.value.toDouble(),
                      color: UdayaTheme.info,
                      width: 20,
                      borderRadius: BorderRadius.circular(4),
                    ),
                  ],
                );
              })
              .toList(),
          titlesData: FlTitlesData(
            bottomTitles: AxisTitles(
              sideTitles: SideTitles(
                showTitles: true,
                interval: 1,
                getTitlesWidget: (value, meta) {
                  final index = value.toInt();
                  if (index >= 0 && index < data.length) {
                    return Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Text(
                        data[index].key,
                        style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 10),
                      ),
                    );
                  }
                  return const SizedBox.shrink();
                },
              ),
            ),
            leftTitles: AxisTitles(
              sideTitles: SideTitles(
                showTitles: true,
                interval: maxValue / 3,
                getTitlesWidget: (value, meta) => Text(
                  value.toInt().toString(),
                  style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 10),
                ),
              ),
            ),
            topTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
            rightTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
          ),
          borderData: FlBorderData(show: false),
          gridData: FlGridData(
            show: true,
            drawVerticalLine: false,
            horizontalInterval: maxValue / 3,
            getDrawingHorizontalLine: (value) => FlLine(
              color: UdayaTheme.bgSurface,
              strokeWidth: 1,
            ),
          ),
        ),
      ),
    );
  }
}

class BlockTimeChart extends StatelessWidget {
  final List<DateTime> timestamps;
  final List<int> blockTimes;

  const BlockTimeChart({
    super.key,
    required this.timestamps,
    required this.blockTimes,
  });

  @override
  Widget build(BuildContext context) {
    if (blockTimes.isEmpty) {
      return const SizedBox(
        height: 200,
        child: Center(child: Text('No block time data available')),
      );
    }

    final maxBlockTime = blockTimes.reduce((a, b) => a > b ? a : b).toDouble();

    return SizedBox(
      height: 200,
      child: LineChart(
        LineChartData(
          lineBarsData: [
            LineChartBarData(
              spots: blockTimes
                  .asMap()
                  .entries
                  .map((entry) => FlSpot(entry.key.toDouble(), entry.value.toDouble()))
                  .toList(),
              isCurved: true,
              color: UdayaTheme.warning,
              barWidth: 3,
              dotData: const FlDotData(show: false),
              belowBarData: BarAreaData(
                show: true,
                color: UdayaTheme.warning.withValues(alpha: 0.1),
              ),
            ),
          ],
          titlesData: FlTitlesData(
            bottomTitles: AxisTitles(
              sideTitles: SideTitles(
                showTitles: true,
                interval: timestamps.length / 6,
                getTitlesWidget: (value, meta) {
                  final index = value.toInt();
                  if (index >= 0 && index < timestamps.length) {
                    final time = timestamps[index];
                    return Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Text(
                        '${time.hour}:${time.minute.toString().padLeft(2, '0')}',
                        style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 10),
                      ),
                    );
                  }
                  return const SizedBox.shrink();
                },
              ),
            ),
            leftTitles: AxisTitles(
              sideTitles: SideTitles(
                showTitles: true,
                interval: maxBlockTime / 3,
                getTitlesWidget: (value, meta) => Text(
                  '${value.toInt()}s',
                  style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 10),
                ),
              ),
            ),
            topTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
            rightTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
          ),
          borderData: FlBorderData(show: false),
          gridData: FlGridData(
            show: true,
            drawVerticalLine: false,
            horizontalInterval: maxBlockTime / 3,
            getDrawingHorizontalLine: (value) => FlLine(
              color: UdayaTheme.bgSurface,
              strokeWidth: 1,
            ),
          ),
        ),
      ),
    );
  }
}

class MempoolChart extends StatelessWidget {
  final List<MapEntry<int, int>> feeDistribution;

  const MempoolChart({
    super.key,
    required this.feeDistribution,
  });

  @override
  Widget build(BuildContext context) {
    if (feeDistribution.isEmpty) {
      return const SizedBox(
        height: 200,
        child: Center(child: Text('No mempool data available')),
      );
    }

    final maxCount = feeDistribution.map((e) => e.value).reduce((a, b) => a > b ? a : b);

    return SizedBox(
      height: 200,
      child: BarChart(
        BarChartData(
          barGroups: feeDistribution
              .asMap()
              .entries
              .map((entry) {
                final index = entry.key;
                final item = entry.value;
                return BarChartGroupData(
                  x: index,
                  barRods: [
                    BarChartRodData(
                      toY: item.value.toDouble(),
                      color: UdayaTheme.success,
                      width: 16,
                      borderRadius: BorderRadius.circular(4),
                    ),
                  ],
                );
              })
              .toList(),
          titlesData: FlTitlesData(
            bottomTitles: AxisTitles(
              sideTitles: SideTitles(
                showTitles: true,
                interval: 1,
                getTitlesWidget: (value, meta) {
                  final index = value.toInt();
                  if (index >= 0 && index < feeDistribution.length) {
                    return Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Text(
                        '${feeDistribution[index].key} sat',
                        style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 10),
                      ),
                    );
                  }
                  return const SizedBox.shrink();
                },
              ),
            ),
            leftTitles: AxisTitles(
              sideTitles: SideTitles(
                showTitles: true,
                interval: maxCount / 3,
                getTitlesWidget: (value, meta) => Text(
                  value.toInt().toString(),
                  style: TextStyle(color: UdayaTheme.textTertiary, fontSize: 10),
                ),
              ),
            ),
            topTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
            rightTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
          ),
          borderData: FlBorderData(show: false),
          gridData: FlGridData(
            show: true,
            drawVerticalLine: false,
            horizontalInterval: maxCount / 3,
            getDrawingHorizontalLine: (value) => FlLine(
              color: UdayaTheme.bgSurface,
              strokeWidth: 1,
            ),
          ),
        ),
      ),
    );
  }
}
