# Udaya ASIC Optimization Profiles

## Certified ASIC Models
| Model | Hashrate | Power | Eff (J/TH) | Firmware |
|-------|----------|-------|------------|----------|
| BF-ASIC S1 | 100 TH/s | 3250W | 32.5 | v2.1.0+ |
| BF-ASIC Pro | 200 TH/s | 5000W | 25.0 | v2.1.0+ |
| BF-ASIC Ultra | 500 TH/s | 10000W | 20.0 | v2.2.0+ |
| Antminer S19 Pro | 110 TH/s | 3250W | 29.5 | Stock/Braiins |
| Antminer S19 XP | 140 TH/s | 3010W | 21.5 | Stock/Braiins |
| Whatsminer M50S | 126 TH/s | 3276W | 26.0 | Stock |

## Optimization Settings
### Braiins OS Settings
```json
{
  "frequency": 450,
  "voltage": 1300,
  "fan_speed": 80,
  "temperature_target": 65
}
```

### Performance Tuning
- **Eco Mode**: 85% hashrate, 70% power consumption
- **Turbo Mode**: 110% hashrate, 130% power consumption (requires enhanced cooling)
- **Custom**: Adjust frequency in 25MHz steps

## Pool Optimization
- Use geographically closest pool endpoint
- Set static difficulty for stable shares
- Enable fan speed auto-regulation
- Monitor reject rate (<2% target)