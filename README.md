# ternary-weather

**Ternary weather simulation: weather phenomena modeled with ternary states {-1, 0, +1}.**

[![Tests](https://img.shields.io/badge/tests-27%20passing-brightgreen)]()

## Why?

Weather is something *everyone* understands — hot, cold, calm, stormy. By mapping
these intuitions to ternary mathematics ({-1, 0, +1}), we make abstract ternary
operations tangible and intuitive.

When a student sees cold air meeting hot air, they're watching ternary diffusion.
When pressure gradients create wind, they're seeing ternary convergence. When
conditions align and a storm forms, they're witnessing ternary pattern emergence.

This crate is part of **Loom** — making agent coordination concepts accessible
through weather, the most universally understood complex system.

## Core Model

### WeatherCell

Each cell in the simulation has three ternary properties:

| Property     | -1        | 0         | +1        |
|-------------|-----------|-----------|-----------|
| Temperature | Cold ❄    | Neutral ━ | Hot ☀     |
| Pressure    | Low ▽     | Normal ─  | High △    |
| Wind        | Left ←    | None ·    | Right →   |

```rust
use ternary_weather::{WeatherCell, Temperature, Pressure, Wind};

let cell = WeatherCell::new(Temperature::Hot, Pressure::Low, Wind::Right);
assert!(cell.is_stormy()); // Hot + Low pressure + Wind = Storm!
```

### WeatherGrid

A 2D grid of weather cells with spatial simulation:

```rust
use ternary_weather::WeatherGrid;

let mut grid = WeatherGrid::new(10, 10);
grid.set(3, 3, WeatherCell::new(Temperature::Hot, Pressure::High, Wind::Right));

// Run simulation
let next = grid.step();
let forecast = grid.forecast(24); // 24-step forecast
```

## Simulation Dynamics

### Heat Diffusion (Ternary Trit Propagation)

Temperature spreads through ternary neighborhood interaction:
- A cold cell surrounded by 2+ hot neighbors warms to neutral
- A hot cell surrounded by 2+ cold neighbors cools to neutral
- A neutral cell with 3+ extreme neighbors shifts toward the majority

This mirrors real heat diffusion — thermal equilibrium through ternary averaging.

### Pressure Systems (Convergence & Divergence)

Pressure evolves based on neighborhood dynamics:
- Normal pressure with 2+ low neighbors → drops to Low (convergence zone)
- Normal pressure with 2+ high neighbors → rises to High (divergence zone)
- Low pressure surrounded by low neighbors → deepens (stays Low)
- High pressure surrounded by high neighbors → strengthens (stays High)

### Wind Generation

Wind is determined by **pressure gradients**:
- Higher pressure to the right → wind blows Left (toward low)
- Higher pressure to the left → wind blows Right (toward low)
- Equal pressure → no wind

This is a ternary version of the real atmospheric rule: wind flows from
high to low pressure.

### Storm Formation

A storm forms when all three conditions align:
1. **Extreme temperature** (Hot or Cold — not Neutral)
2. **Low pressure**
3. **Wind present** (Left or Right — not None)

```rust
let storms = grid.find_storms();
for (x, y) in storms {
    println!("Storm at ({}, {})!", x, y);
}
```

## Quick Start

```rust
use ternary_weather::*;

// Create a weather system
let mut grid = WeatherGrid::new(8, 8);

// Set up a cold front on the left
for y in 0..8 {
    grid.set(0, y, WeatherCell::new(Temperature::Cold, Pressure::High, Wind::Right));
    grid.set(1, y, WeatherCell::new(Temperature::Cold, Pressure::Normal, Wind::Right));
}

// Set up warm air on the right
for y in 0..8 {
    grid.set(6, y, WeatherCell::new(Temperature::Hot, Pressure::Low, Wind::Left));
    grid.set(7, y, WeatherCell::new(Temperature::Hot, Pressure::Low, Wind::Left));
}

// Simulate the collision
let forecast = grid.forecast(10);
for (step, state) in forecast.iter().enumerate() {
    println!("=== Hour {} ===", step + 1);
    println!("{}", state.render_temperature());
    let stats = state.statistics();
    println!("Storms: {}, Imbalance: {:.2}", 
        stats.storms, stats.temperature_imbalance());
}
```

## Features

### Forecasting
Run N steps ahead and examine the full trajectory:
```rust
let forecast = grid.forecast(48); // 48-step forecast
```

### Storm Detection
Find all cells where storm conditions have aligned:
```rust
let storms = grid.find_storms();
```

### Statistics
Comprehensive population statistics:
```rust
let stats = grid.statistics();
println!("Cold: {}  Neutral: {}  Hot: {}", stats.cold, stats.neutral, stats.hot);
println!("Storm ratio: {:.1}%", stats.storm_ratio() * 100.0);
println!("Temperature imbalance: {:.3}", stats.temperature_imbalance());
```

### Rendering
Multiple visualization modes:
```rust
println!("{}", grid.render_temperature()); // ❄━☀ view
println!("{}", grid.render_wind());        // ←·→ view  
println!("{}", grid.render());             // Full cell info: ☀△→
```

### Boundary Handling
The grid uses **toroidal wrapping** — cells on the edge connect to the opposite
edge, simulating a closed weather system with no boundaries.

## Educational Value

Designed for **Loom** — making agent coordination accessible:

1. **Ternary arithmetic** → Temperature addition, negation
2. **Spatial propagation** → Heat diffusion as ternary neighborhood rules
3. **Gradient dynamics** → Pressure creating wind (ternary convergence)
4. **Emergent phenomena** → Storms from ternary condition alignment
5. **System stability** → Neutral grids stay neutral (equilibrium)
6. **Forecasting** → Predicting N steps ahead (ternary trajectory)

## The Ternary Weather Philosophy

| Real Weather     | Ternary Model                    |
|-----------------|----------------------------------|
| Temperature      | Trit: Cold/Neutral/Hot           |
| Pressure systems | Trit: Low/Normal/High            |
| Wind direction   | Trit: Left/None/Right            |
| Heat transfer    | Ternary diffusion rules          |
| Fronts           | Pressure convergence zones       |
| Storms           | Aligned ternary conditions       |
| Forecast         | Iterated ternary step function   |

## API Overview

| Type          | Description                              |
|---------------|------------------------------------------|
| `Temperature` | Cold, Neutral, Hot                       |
| `Pressure`    | Low, Normal, High                        |
| `Wind`        | Left, None, Right                        |
| `WeatherCell` | Single cell with temp/pressure/wind      |
| `WeatherGrid` | 2D spatial simulation grid               |
| `WeatherStats`| Population counts, ratios, imbalances    |

## License

MIT
