//! # ternary-weather
//!
//! Ternary weather simulation: weather phenomena modeled with ternary states.
//!
//! Weather is something everyone understands — hot, cold, calm, stormy. This crate
//! maps those intuitions to ternary mathematics ({-1, 0, +1}), making abstract
//! ternary operations tangible through weather everyone experiences.
//!
//! Each `WeatherCell` holds temperature, pressure, and wind as trits. The grid
//! simulates heat diffusion, pressure dynamics, storm formation, and forecasting
//! — all using ternary arithmetic.

use std::fmt;

/// Ternary wind direction: Left, None, or Right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wind {
    Left,
    None,
    Right,
}

impl Wind {
    pub fn to_trit(self) -> i8 {
        match self {
            Wind::Left => -1,
            Wind::None => 0,
            Wind::Right => 1,
        }
    }

    pub fn from_trit(t: i8) -> Self {
        match t {
            -1 => Wind::Left,
            0 => Wind::None,
            1 => Wind::Right,
            _ => panic!("Invalid wind trit: {}", t),
        }
    }
}

/// Ternary temperature state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Temperature {
    Cold,
    Neutral,
    Hot,
}

impl Temperature {
    pub fn to_trit(self) -> i8 {
        match self {
            Temperature::Cold => -1,
            Temperature::Neutral => 0,
            Temperature::Hot => 1,
        }
    }

    pub fn from_trit(t: i8) -> Self {
        match t {
            -1 => Temperature::Cold,
            0 => Temperature::Neutral,
            1 => Temperature::Hot,
            _ => panic!("Invalid temperature trit: {}", t),
        }
    }

    /// Ternary addition of temperatures.
    pub fn add(self, other: Temperature) -> Temperature {
        Temperature::from_trit(((self.to_trit() + other.to_trit() + 4) % 3) - 1)
    }

    /// Negate temperature: Hot↔Cold, Neutral stays.
    pub fn negate(self) -> Temperature {
        Temperature::from_trit(-self.to_trit())
    }
}

/// Ternary pressure state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pressure {
    Low,
    Normal,
    High,
}

impl Pressure {
    pub fn to_trit(self) -> i8 {
        match self {
            Pressure::Low => -1,
            Pressure::Normal => 0,
            Pressure::High => 1,
        }
    }

    pub fn from_trit(t: i8) -> Self {
        match t {
            -1 => Pressure::Low,
            0 => Pressure::Normal,
            1 => Pressure::High,
            _ => panic!("Invalid pressure trit: {}", t),
        }
    }
}

/// A single weather cell with ternary temperature, pressure, and wind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WeatherCell {
    pub temperature: Temperature,
    pub pressure: Pressure,
    pub wind: Wind,
}

impl WeatherCell {
    /// Create a new weather cell.
    pub fn new(temperature: Temperature, pressure: Pressure, wind: Wind) -> Self {
        WeatherCell { temperature, pressure, wind }
    }

    /// Default cell: neutral temperature, normal pressure, no wind.
    pub fn neutral() -> Self {
        WeatherCell {
            temperature: Temperature::Neutral,
            pressure: Pressure::Normal,
            wind: Wind::None,
        }
    }

    /// Is this cell stormy? Storms form when:
    /// - Temperature is extreme (Hot or Cold) AND
    /// - Pressure is Low AND
    /// - Wind is present (not None)
    pub fn is_stormy(&self) -> bool {
        let extreme_temp = self.temperature != Temperature::Neutral;
        let low_pressure = self.pressure == Pressure::Low;
        let has_wind = self.wind != Wind::None;
        extreme_temp && low_pressure && has_wind
    }

    /// Is this cell calm? Calm when temperature is neutral and no wind.
    pub fn is_calm(&self) -> bool {
        self.temperature == Temperature::Neutral && self.wind == Wind::None
    }
}

impl fmt::Display for WeatherCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match self.temperature {
            Temperature::Cold => '❄',
            Temperature::Neutral => '━',
            Temperature::Hot => '☀',
        };
        let p = match self.pressure {
            Pressure::Low => '▽',
            Pressure::Normal => '─',
            Pressure::High => '△',
        };
        let w = match self.wind {
            Wind::Left => '←',
            Wind::None => '·',
            Wind::Right => '→',
        };
        write!(f, "{}{}{}", t, p, w)
    }
}

/// A 2D grid of weather cells for spatial simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherGrid {
    width: usize,
    height: usize,
    cells: Vec<WeatherCell>,
}

impl WeatherGrid {
    /// Create a new grid filled with neutral weather.
    pub fn new(width: usize, height: usize) -> Self {
        WeatherGrid {
            width,
            height,
            cells: vec![WeatherCell::neutral(); width * height],
        }
    }

    /// Create from cells vector.
    pub fn from_cells(width: usize, height: usize, cells: Vec<WeatherCell>) -> Option<Self> {
        if cells.len() != width * height {
            return None;
        }
        Some(WeatherGrid { width, height, cells })
    }

    /// Width of the grid.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Height of the grid.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get cell at (x, y).
    pub fn get(&self, x: usize, y: usize) -> Option<&WeatherCell> {
        if x < self.width && y < self.height {
            Some(&self.cells[y * self.width + x])
        } else {
            None
        }
    }

    /// Set cell at (x, y).
    pub fn set(&mut self, x: usize, y: usize, cell: WeatherCell) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = cell;
        }
    }

    /// Get cardinal neighbors (up, down, left, right) of a cell.
    fn cardinal_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize, WeatherCell)> {
        let mut neighbors = Vec::new();
        let dirs: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        for (dx, dy) in &dirs {
            let nx = (x as i32 + dx).rem_euclid(self.width as i32) as usize;
            let ny = (y as i32 + dy).rem_euclid(self.height as i32) as usize;
            neighbors.push((nx, ny, self.cells[ny * self.width + nx]));
        }
        neighbors
    }

    /// Heat diffusion: ternary trit propagation.
    ///
    /// Temperature propagates from hot cells to neighbors and cold cells absorb.
    /// For each cell:
    /// - If 2+ cardinal neighbors are Hot and current is Cold → becomes Neutral
    /// - If 2+ cardinal neighbors are Cold and current is Hot → becomes Neutral
    /// - If majority of neighbors have same temperature → stays
    /// - Wind affects diffusion direction
    fn diffuse_temperature(&self) -> Vec<Temperature> {
        let mut temps = Vec::with_capacity(self.cells.len());
        for y in 0..self.height {
            for x in 0..self.width {
                let current = self.cells[y * self.width + x].temperature;
                let neighbors = self.cardinal_neighbors(x, y);

                let hot_count = neighbors.iter()
                    .filter(|&&(_, _, ref c)| c.temperature == Temperature::Hot)
                    .count();
                let cold_count = neighbors.iter()
                    .filter(|&&(_, _, ref c)| c.temperature == Temperature::Cold)
                    .count();

                let new_temp = match current {
                    Temperature::Cold => {
                        if hot_count >= 2 {
                            Temperature::Neutral
                        } else if hot_count >= 3 {
                            Temperature::Hot
                        } else {
                            Temperature::Cold
                        }
                    }
                    Temperature::Neutral => {
                        if hot_count >= 3 {
                            Temperature::Hot
                        } else if cold_count >= 3 {
                            Temperature::Cold
                        } else {
                            Temperature::Neutral
                        }
                    }
                    Temperature::Hot => {
                        if cold_count >= 2 {
                            Temperature::Neutral
                        } else if cold_count >= 3 {
                            Temperature::Cold
                        } else {
                            Temperature::Hot
                        }
                    }
                };
                temps.push(new_temp);
            }
        }
        temps
    }

    /// Pressure dynamics: convergence and divergence.
    ///
    /// - High pressure spreads to neighbors with Normal pressure
    /// - Low pressure deepens when surrounded by Low pressure neighbors
    /// - Wind is created by pressure differences
    fn update_pressure_and_wind(&self) -> Vec<(Pressure, Wind)> {
        let mut results = Vec::with_capacity(self.cells.len());
        for y in 0..self.height {
            for x in 0..self.width {
                let current = self.cells[y * self.width + x];
                let neighbors = self.cardinal_neighbors(x, y);

                // Count pressure types among neighbors
                let high_count = neighbors.iter()
                    .filter(|&&(_, _, ref c)| c.pressure == Pressure::High)
                    .count();
                let low_count = neighbors.iter()
                    .filter(|&&(_, _, ref c)| c.pressure == Pressure::Low)
                    .count();

                // Pressure update
                let new_pressure = match current.pressure {
                    Pressure::Low => {
                        if low_count >= 3 {
                            Pressure::Low // deepens
                        } else if high_count >= 3 {
                            Pressure::Normal
                        } else {
                            Pressure::Low
                        }
                    }
                    Pressure::Normal => {
                        if high_count >= 2 {
                            Pressure::High
                        } else if low_count >= 2 {
                            Pressure::Low
                        } else {
                            Pressure::Normal
                        }
                    }
                    Pressure::High => {
                        if low_count >= 3 {
                            Pressure::Normal
                        } else if high_count >= 3 {
                            Pressure::High // strengthens
                        } else {
                            Pressure::High
                        }
                    }
                };

                // Wind update: determined by pressure gradient
                // Wind blows from high pressure to low pressure
                let left_pressure = neighbors.iter()
                    .filter(|&&(nx, _, _)| nx == (x as i32 - 1).rem_euclid(self.width as i32) as usize)
                    .map(|&(_, _, c)| c.pressure.to_trit())
                    .next()
                    .unwrap_or(0);
                let right_pressure = neighbors.iter()
                    .filter(|&&(nx, _, _)| nx == (x as i32 + 1).rem_euclid(self.width as i32) as usize)
                    .map(|&(_, _, c)| c.pressure.to_trit())
                    .next()
                    .unwrap_or(0);

                let pressure_diff = right_pressure - left_pressure;
                let new_wind = if pressure_diff > 0 {
                    Wind::Left // wind flows from high (right) to low (left)
                } else if pressure_diff < 0 {
                    Wind::Right // wind flows from high (left) to low (right)
                } else {
                    Wind::None
                };

                results.push((new_pressure, new_wind));
            }
        }
        results
    }

    /// Advance the simulation by one step.
    ///
    /// 1. Diffuse temperature
    /// 2. Update pressure and wind
    /// 3. Combine results into new cells
    pub fn step(&self) -> WeatherGrid {
        let temps = self.diffuse_temperature();
        let pressure_wind = self.update_pressure_and_wind();

        let cells: Vec<WeatherCell> = temps.into_iter()
            .zip(pressure_wind.into_iter())
            .map(|(t, (p, w))| WeatherCell::new(t, p, w))
            .collect();

        WeatherGrid {
            width: self.width,
            height: self.height,
            cells,
        }
    }

    /// Run N steps and return all intermediate grids.
    pub fn forecast(&self, steps: usize) -> Vec<WeatherGrid> {
        let mut history = Vec::with_capacity(steps);
        let mut current = self.clone();
        for _ in 0..steps {
            current = current.step();
            history.push(current.clone());
        }
        history
    }

    /// Detect storm cells in the grid.
    pub fn find_storms(&self) -> Vec<(usize, usize)> {
        let mut storms = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.cells[y * self.width + x].is_stormy() {
                    storms.push((x, y));
                }
            }
        }
        storms
    }

    /// Count cells by state.
    pub fn statistics(&self) -> WeatherStats {
        let mut cold = 0;
        let mut neutral = 0;
        let mut hot = 0;
        let mut low_pressure = 0;
        let mut high_pressure = 0;
        let mut left_wind = 0;
        let mut right_wind = 0;
        let mut storms = 0;

        for cell in &self.cells {
            match cell.temperature {
                Temperature::Cold => cold += 1,
                Temperature::Neutral => neutral += 1,
                Temperature::Hot => hot += 1,
            }
            match cell.pressure {
                Pressure::Low => low_pressure += 1,
                Pressure::Normal => {}
                Pressure::High => high_pressure += 1,
            }
            match cell.wind {
                Wind::Left => left_wind += 1,
                Wind::None => {}
                Wind::Right => right_wind += 1,
            }
            if cell.is_stormy() {
                storms += 1;
            }
        }

        WeatherStats {
            cold,
            neutral,
            hot,
            low_pressure,
            high_pressure,
            left_wind,
            right_wind,
            storms,
            total: self.cells.len(),
        }
    }

    /// Render the grid as a string showing temperature only.
    pub fn render_temperature(&self) -> String {
        let mut out = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let ch = match self.cells[y * self.width + x].temperature {
                    Temperature::Cold => '❄',
                    Temperature::Neutral => '━',
                    Temperature::Hot => '☀',
                };
                out.push(ch);
            }
            out.push('\n');
        }
        out
    }

    /// Render the grid showing wind direction.
    pub fn render_wind(&self) -> String {
        let mut out = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let ch = match self.cells[y * self.width + x].wind {
                    Wind::Left => '←',
                    Wind::None => '·',
                    Wind::Right => '→',
                };
                out.push(ch);
            }
            out.push('\n');
        }
        out
    }

    /// Render full cell info.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                out.push_str(&format!("{} ", self.cells[y * self.width + x]));
            }
            out.push('\n');
        }
        out
    }
}

/// Statistics about weather conditions in the grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherStats {
    pub cold: usize,
    pub neutral: usize,
    pub hot: usize,
    pub low_pressure: usize,
    pub high_pressure: usize,
    pub left_wind: usize,
    pub right_wind: usize,
    pub storms: usize,
    pub total: usize,
}

impl WeatherStats {
    /// Storm ratio.
    pub fn storm_ratio(&self) -> f64 {
        self.storms as f64 / self.total as f64
    }

    /// Temperature imbalance: absolute difference between hot and cold ratios.
    pub fn temperature_imbalance(&self) -> f64 {
        let hot_ratio = self.hot as f64 / self.total as f64;
        let cold_ratio = self.cold as f64 / self.total as f64;
        (hot_ratio - cold_ratio).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_trit_roundtrip() {
        assert_eq!(Temperature::from_trit(Temperature::Cold.to_trit()), Temperature::Cold);
        assert_eq!(Temperature::from_trit(Temperature::Neutral.to_trit()), Temperature::Neutral);
        assert_eq!(Temperature::from_trit(Temperature::Hot.to_trit()), Temperature::Hot);
    }

    #[test]
    fn test_temperature_addition() {
        assert_eq!(Temperature::Cold.add(Temperature::Hot), Temperature::Neutral);
        assert_eq!(Temperature::Hot.add(Temperature::Hot), Temperature::Cold);
        assert_eq!(Temperature::Neutral.add(Temperature::Neutral), Temperature::Neutral);
        assert_eq!(Temperature::Cold.add(Temperature::Neutral), Temperature::Cold);
    }

    #[test]
    fn test_temperature_negation() {
        assert_eq!(Temperature::Cold.negate(), Temperature::Hot);
        assert_eq!(Temperature::Hot.negate(), Temperature::Cold);
        assert_eq!(Temperature::Neutral.negate(), Temperature::Neutral);
    }

    #[test]
    fn test_wind_trit_roundtrip() {
        assert_eq!(Wind::from_trit(Wind::Left.to_trit()), Wind::Left);
        assert_eq!(Wind::from_trit(Wind::None.to_trit()), Wind::None);
        assert_eq!(Wind::from_trit(Wind::Right.to_trit()), Wind::Right);
    }

    #[test]
    fn test_weather_cell_stormy() {
        let stormy = WeatherCell::new(Temperature::Hot, Pressure::Low, Wind::Right);
        assert!(stormy.is_stormy());

        let also_stormy = WeatherCell::new(Temperature::Cold, Pressure::Low, Wind::Left);
        assert!(also_stormy.is_stormy());
    }

    #[test]
    fn test_weather_cell_not_stormy() {
        // No wind
        let no_wind = WeatherCell::new(Temperature::Hot, Pressure::Low, Wind::None);
        assert!(!no_wind.is_stormy());

        // Normal pressure
        let normal_pressure = WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::Right);
        assert!(!normal_pressure.is_stormy());

        // Neutral temp
        let neutral_temp = WeatherCell::new(Temperature::Neutral, Pressure::Low, Wind::Right);
        assert!(!neutral_temp.is_stormy());
    }

    #[test]
    fn test_weather_cell_calm() {
        let calm = WeatherCell::neutral();
        assert!(calm.is_calm());

        let not_calm = WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None);
        assert!(!not_calm.is_calm());
    }

    #[test]
    fn test_grid_new() {
        let grid = WeatherGrid::new(3, 3);
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 3);
        for y in 0..3 {
            for x in 0..3 {
                let cell = grid.get(x, y).unwrap();
                assert_eq!(cell.temperature, Temperature::Neutral);
                assert_eq!(cell.pressure, Pressure::Normal);
                assert_eq!(cell.wind, Wind::None);
            }
        }
    }

    #[test]
    fn test_grid_set_get() {
        let mut grid = WeatherGrid::new(3, 3);
        let hot_cell = WeatherCell::new(Temperature::Hot, Pressure::High, Wind::Right);
        grid.set(1, 1, hot_cell);
        assert_eq!(grid.get(1, 1), Some(&hot_cell));
        assert_eq!(grid.get(0, 0), Some(&WeatherCell::neutral()));
    }

    #[test]
    fn test_heat_diffusion_cold_surrounded_by_hot() {
        let mut grid = WeatherGrid::new(3, 3);
        // Center is cold, all 4 cardinal neighbors are hot
        grid.set(1, 1, WeatherCell::new(Temperature::Cold, Pressure::Normal, Wind::None));
        grid.set(0, 1, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));
        grid.set(2, 1, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));
        grid.set(1, 0, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));
        grid.set(1, 2, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));

        let next = grid.step();
        // Center cold cell surrounded by 4 hot → should warm to Neutral
        assert_eq!(next.get(1, 1).unwrap().temperature, Temperature::Neutral);
    }

    #[test]
    fn test_heat_diffusion_hot_stays_hot() {
        let mut grid = WeatherGrid::new(3, 3);
        // Center is hot, neighbors are neutral
        grid.set(1, 1, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));

        let next = grid.step();
        // No cold neighbors, so hot should stay hot
        assert_eq!(next.get(1, 1).unwrap().temperature, Temperature::Hot);
    }

    #[test]
    fn test_pressure_convergence() {
        let mut grid = WeatherGrid::new(3, 3);
        // Center is normal, surrounded by 4 low pressure cells
        grid.set(1, 1, WeatherCell::new(Temperature::Neutral, Pressure::Normal, Wind::None));
        grid.set(0, 1, WeatherCell::new(Temperature::Neutral, Pressure::Low, Wind::None));
        grid.set(2, 1, WeatherCell::new(Temperature::Neutral, Pressure::Low, Wind::None));
        grid.set(1, 0, WeatherCell::new(Temperature::Neutral, Pressure::Low, Wind::None));
        grid.set(1, 2, WeatherCell::new(Temperature::Neutral, Pressure::Low, Wind::None));

        let next = grid.step();
        // 4 low-pressure neighbors ≥ 2 → center should become Low
        assert_eq!(next.get(1, 1).unwrap().pressure, Pressure::Low);
    }

    #[test]
    fn test_wind_from_pressure_gradient() {
        let mut grid = WeatherGrid::new(3, 1);
        // Left cell has high pressure, right cell has low pressure
        grid.set(0, 0, WeatherCell::new(Temperature::Neutral, Pressure::High, Wind::None));
        grid.set(1, 0, WeatherCell::new(Temperature::Neutral, Pressure::Normal, Wind::None));
        grid.set(2, 0, WeatherCell::new(Temperature::Neutral, Pressure::Low, Wind::None));

        let next = grid.step();
        // Middle cell: left neighbor is high (+1), right neighbor is low (-1)
        // pressure_diff = -1 - 1 = -2, which is < 0 → wind Right
        assert_eq!(next.get(1, 0).unwrap().wind, Wind::Right);
    }

    #[test]
    fn test_storm_detection() {
        let mut grid = WeatherGrid::new(3, 3);
        grid.set(0, 0, WeatherCell::new(Temperature::Hot, Pressure::Low, Wind::Right));
        grid.set(2, 2, WeatherCell::new(Temperature::Cold, Pressure::Low, Wind::Left));

        let storms = grid.find_storms();
        assert_eq!(storms.len(), 2);
        assert!(storms.contains(&(0, 0)));
        assert!(storms.contains(&(2, 2)));
    }

    #[test]
    fn test_forecast() {
        let mut grid = WeatherGrid::new(3, 3);
        grid.set(1, 1, WeatherCell::new(Temperature::Hot, Pressure::High, Wind::Right));
        let forecast = grid.forecast(5);
        assert_eq!(forecast.len(), 5);
    }

    #[test]
    fn test_statistics() {
        let mut grid = WeatherGrid::new(2, 2);
        grid.set(0, 0, WeatherCell::new(Temperature::Hot, Pressure::High, Wind::Right));
        grid.set(1, 0, WeatherCell::new(Temperature::Cold, Pressure::Low, Wind::Left));
        grid.set(0, 1, WeatherCell::neutral());
        grid.set(1, 1, WeatherCell::new(Temperature::Hot, Pressure::Low, Wind::Right));

        let stats = grid.statistics();
        assert_eq!(stats.hot, 2);
        assert_eq!(stats.cold, 1);
        assert_eq!(stats.neutral, 1);
        assert_eq!(stats.low_pressure, 2);
        assert_eq!(stats.high_pressure, 1);
        assert_eq!(stats.left_wind, 1);
        assert_eq!(stats.right_wind, 2);
        assert_eq!(stats.total, 4);
    }

    #[test]
    fn test_storm_ratio() {
        let mut grid = WeatherGrid::new(4, 1);
        grid.set(0, 0, WeatherCell::new(Temperature::Hot, Pressure::Low, Wind::Right));
        grid.set(1, 0, WeatherCell::neutral());
        grid.set(2, 0, WeatherCell::neutral());
        grid.set(3, 0, WeatherCell::new(Temperature::Cold, Pressure::Low, Wind::Left));

        let stats = grid.statistics();
        assert!((stats.storm_ratio() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_temperature_imbalance() {
        let mut grid = WeatherGrid::new(4, 1);
        grid.set(0, 0, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));
        grid.set(1, 0, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));
        grid.set(2, 0, WeatherCell::neutral());
        grid.set(3, 0, WeatherCell::neutral());

        let stats = grid.statistics();
        // 2 hot, 0 cold out of 4: imbalance = |0.5 - 0.0| = 0.5
        assert!((stats.temperature_imbalance() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_render_temperature() {
        let mut grid = WeatherGrid::new(3, 1);
        grid.set(0, 0, WeatherCell::new(Temperature::Cold, Pressure::Normal, Wind::None));
        grid.set(1, 0, WeatherCell::neutral());
        grid.set(2, 0, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));
        let rendered = grid.render_temperature();
        assert!(rendered.contains('❄'));
        assert!(rendered.contains('━'));
        assert!(rendered.contains('☀'));
    }

    #[test]
    fn test_render_wind() {
        let mut grid = WeatherGrid::new(3, 1);
        grid.set(0, 0, WeatherCell::new(Temperature::Neutral, Pressure::Normal, Wind::Left));
        grid.set(1, 0, WeatherCell::neutral());
        grid.set(2, 0, WeatherCell::new(Temperature::Neutral, Pressure::Normal, Wind::Right));
        let rendered = grid.render_wind();
        assert!(rendered.contains('←'));
        assert!(rendered.contains('·'));
        assert!(rendered.contains('→'));
    }

    #[test]
    fn test_from_cells() {
        let cells = vec![
            WeatherCell::neutral(),
            WeatherCell::new(Temperature::Hot, Pressure::High, Wind::Right),
            WeatherCell::new(Temperature::Cold, Pressure::Low, Wind::Left),
            WeatherCell::neutral(),
        ];
        let grid = WeatherGrid::from_cells(2, 2, cells).unwrap();
        assert_eq!(grid.get(1, 0).unwrap().temperature, Temperature::Hot);
        assert_eq!(grid.get(0, 1).unwrap().temperature, Temperature::Cold);
    }

    #[test]
    fn test_from_cells_wrong_size() {
        let cells = vec![WeatherCell::neutral()];
        assert!(WeatherGrid::from_cells(2, 2, cells).is_none());
    }

    #[test]
    fn test_out_of_bounds() {
        let grid = WeatherGrid::new(3, 3);
        assert!(grid.get(5, 5).is_none());
    }

    #[test]
    fn test_boundary_wrapping() {
        // Set up a grid where wrapping matters
        let mut grid = WeatherGrid::new(3, 3);
        // Place hot cells at edges
        grid.set(0, 1, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));
        grid.set(2, 1, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));
        grid.set(1, 0, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));
        grid.set(1, 2, WeatherCell::new(Temperature::Hot, Pressure::Normal, Wind::None));
        // Corner cell (0,0) — via wrapping, it sees cells on opposite edges
        // Cardinal neighbors of (0,0): up=(0,2), down=(0,1), left=(2,0), right=(1,0)
        // (0,1) is Hot, (1,0) is Hot → 2 hot neighbors
        let next = grid.step();
        // Corner cell starts cold (neutral), 2 hot neighbors isn't enough to change neutral
        // (needs 3 hot for neutral→hot)
        assert_eq!(next.get(0, 0).unwrap().temperature, Temperature::Neutral);
    }

    #[test]
    fn test_forecast_stability() {
        // A neutral grid should stay neutral forever
        let grid = WeatherGrid::new(5, 5);
        let forecast = grid.forecast(10);
        for (i, step) in forecast.iter().enumerate() {
            for y in 0..5 {
                for x in 0..5 {
                    let cell = step.get(x, y).unwrap();
                    assert_eq!(cell.temperature, Temperature::Neutral, "Step {}: ({},{}) temp changed", i, x, y);
                    assert_eq!(cell.pressure, Pressure::Normal, "Step {}: ({},{}) pressure changed", i, x, y);
                    assert_eq!(cell.wind, Wind::None, "Step {}: ({},{}) wind changed", i, x, y);
                }
            }
        }
    }

    #[test]
    fn test_cell_display() {
        let cell = WeatherCell::new(Temperature::Hot, Pressure::High, Wind::Right);
        let s = format!("{}", cell);
        assert!(s.contains('☀'));
        assert!(s.contains('△'));
        assert!(s.contains('→'));
    }

    #[test]
    fn test_pressure_roundtrip() {
        assert_eq!(Pressure::from_trit(Pressure::Low.to_trit()), Pressure::Low);
        assert_eq!(Pressure::from_trit(Pressure::Normal.to_trit()), Pressure::Normal);
        assert_eq!(Pressure::from_trit(Pressure::High.to_trit()), Pressure::High);
    }
}
