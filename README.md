# ternary-weather: Environmental conditions and their effects on agents

Models load storms, network weather, resource climate, and other environmental phenomena that affect how agents behave in fleet rooms.

## Why This Exists

Fleet rooms don't operate in isolation — they have conditions. Load spikes are storms. Network latency is weather. Long-term averages are climate. Agents need to sense these conditions, predict them, and take shelter when things get bad. This crate provides the vocabulary for that: current conditions, forecasts, storms, climate patterns, change detection, and shelter strategies.

## Core Concepts

- **Weather**: The current environmental state of a room — a collection of named conditions with severity levels.
- **Severity**: Five levels from Calm to Critical. Conditions at Severe or above are "dangerous."
- **Forecast**: A sequence of predicted future weather states, keyed by time offset.
- **Storm**: An adverse event with a start time, duration, affected locations, and severity.
- **Climate**: Long-term averages and storm frequency for a region. Detects anomalies (values far from average).
- **WeatherVane**: Tracks condition changes over time and flags significant shifts (above a threshold).
- **StormShelter**: Named refuges with capacity limits and lists of conditions they protect against.

## Quick Start

```toml
[dependencies]
ternary-weather = "0.1"
```

```rust
use ternary_weather::*;

let mut weather = Weather::new("engine-room", 100);
weather.set_condition(Condition::new("latency", 250.0, Severity::Severe));
weather.set_condition(Condition::new("packet_loss", 0.05, Severity::Mild));

assert!(!weather.is_safe()); // Severe latency makes it unsafe

let mut storm = Storm::new("load-spike", Severity::Critical, 100, 50);
storm.affect("engine-room");
assert!(storm.is_active_at(130));
assert!(!storm.is_active_at(200));
```

## API Overview

| Type | Description |
|------|-------------|
| `Weather` | Current conditions for a room, with overall severity computation |
| `Condition` | A single measurable condition (name, value, severity) |
| `Severity` | Five-level scale: Calm, Mild, Moderate, Severe, Critical |
| `Forecast` | Ordered predictions of future weather states |
| `Storm` | An adverse event spanning locations with a time window |
| `Climate` | Long-term averages and anomaly detection for a region |
| `WeatherVane` | Detects significant changes between readings |
| `StormShelter` | Named refuges with capacity and condition-specific protection |

## How It Works

`Weather` is a simple `HashMap<String, Condition>`. Overall severity is the maximum of all individual condition severities. An empty weather report is Calm (no conditions = no danger).

`Forecast` stores predictions as a `Vec` of (offset, Weather) pairs. Lookups are linear scan — fine for short forecasts. The `nearest_to` method finds the closest prediction to any requested time.

`Storm` uses a simple time window: active from `started_at` to `started_at + duration`. Progress is a linear 0.0→1.0 ramp. No bell curves or gradual onset/offset modeling.

`Climate` tracks averages and flags anomalies as values more than 2x or less than 0.5x the average. This is intentionally crude — for proper statistical anomaly detection, use `ternary-entropy` or similar.

`StormShelter` is a capacity-managed registry. Shelters have named lists of conditions they protect against — a shelter that blocks "load" doesn't help against "latency" storms.

## Known Limitations

- **No interpolation**: Forecasts are discrete time points. Between predictions, there's no gradual transition.
- **Linear storm model**: Storms are full-on during their window and gone after. No gradual ramp-up or cool-down.
- **Crude anomaly detection**: 2x threshold is arbitrary. Real anomaly detection needs distribution awareness.
- **No geographic propagation**: Storms affect explicitly listed locations. There's no spatial spread model.
- **Single-dimension shelter**: A shelter either blocks a condition or doesn't. No partial protection.

## Use Cases

- **Load shedding**: When weather reports Severe load, agents throttle non-critical work.
- **Predictive caching**: Forecasts let agents pre-cache data before a predicted storm.
- **Capacity planning**: Climate data shows which rooms historically have the most storms.
- **Alert routing**: WeatherVane detects sudden changes and triggers operator notifications.
- **Evacuation**: StormShelter tracks where agents can retreat during critical events.

## Ecosystem Context

Part of the SuperInstance ternary fleet. Related crates:
- `ternary-room`: Rooms that weather applies to.
- `ternary-sensor`: Raw sensor data that feeds into weather conditions.
- `ternary-observatory`: Broader fleet-wide monitoring that includes weather data.

## License

MIT

## See Also
- **ternary-observatory** — related
- **ternary-predict** — related
- **ternary-sensor** — related
- **ternary-dynamics** — related
- **ternary-beacon** — related

