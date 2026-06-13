#![forbid(unsafe_code)]

//! Environmental conditions and their effects on agent operations.
//!
//! Models weather-like phenomena that affect room operations: load storms,
//! network weather, resource climate, and gradual condition changes.
//! Agents use this to adapt behavior — shed load during storms, cache during
//! droughts, throttle during high-latency conditions.

use std::collections::HashMap;

/// Severity of a weather condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    Calm,
    Mild,
    Moderate,
    Severe,
    Critical,
}

impl Severity {
    pub fn level(&self) -> u8 {
        match self {
            Severity::Calm => 0,
            Severity::Mild => 1,
            Severity::Moderate => 2,
            Severity::Severe => 3,
            Severity::Critical => 4,
        }
    }

    pub fn from_level(level: u8) -> Option<Self> {
        match level {
            0 => Some(Severity::Calm),
            1 => Some(Severity::Mild),
            2 => Some(Severity::Moderate),
            3 => Some(Severity::Severe),
            4 => Some(Severity::Critical),
            _ => None,
        }
    }

    pub fn is_dangerous(&self) -> bool {
        matches!(self, Severity::Severe | Severity::Critical)
    }
}

/// A measurable environmental condition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Condition {
    pub name: &'static str,
    pub value: f64,
    pub severity: Severity,
}

impl Condition {
    pub fn new(name: &'static str, value: f64, severity: Severity) -> Self {
        Self { name, value, severity }
    }
}

/// Current weather state for a room or region.
#[derive(Clone, Debug)]
pub struct Weather {
    /// Which room/region this weather applies to.
    pub location: String,
    /// Timestamp-like counter for when this was observed.
    pub observed_at: u64,
    conditions: HashMap<String, Condition>,
}

impl Weather {
    pub fn new(location: &str, observed_at: u64) -> Self {
        Self {
            location: location.to_string(),
            observed_at,
            conditions: HashMap::new(),
        }
    }

    /// Add or update a condition.
    pub fn set_condition(&mut self, condition: Condition) {
        self.conditions.insert(condition.name.to_string(), condition);
    }

    /// Get a condition by name.
    pub fn get_condition(&self, name: &str) -> Option<&Condition> {
        self.conditions.get(name)
    }

    /// Overall severity: the worst of all conditions.
    pub fn overall_severity(&self) -> Severity {
        self.conditions
            .values()
            .map(|c| c.severity.level())
            .max()
            .and_then(Severity::from_level)
            .unwrap_or(Severity::Calm)
    }

    /// All condition names.
    pub fn condition_names(&self) -> Vec<&str> {
        self.conditions.keys().map(|s| s.as_str()).collect()
    }

    pub fn condition_count(&self) -> usize {
        self.conditions.len()
    }

    /// Is it safe to operate normally?
    pub fn is_safe(&self) -> bool {
        !self.overall_severity().is_dangerous()
    }
}

/// A predicted future weather state.
#[derive(Clone, Debug)]
pub struct Forecast {
    /// Predictions indexed by time offset (relative ticks).
    predictions: Vec<(u64, Weather)>,
}

impl Forecast {
    pub fn new() -> Self {
        Self { predictions: Vec::new() }
    }

    /// Add a prediction at a future time offset.
    pub fn predict(&mut self, at_offset: u64, weather: Weather) {
        self.predictions.push((at_offset, weather));
    }

    /// Get the predicted weather at a specific offset (exact match).
    pub fn at(&self, offset: u64) -> Option<&Weather> {
        self.predictions
            .iter()
            .find(|(t, _)| *t == offset)
            .map(|(_, w)| w)
    }

    /// Get the nearest prediction to an offset.
    pub fn nearest_to(&self, offset: u64) -> Option<&Weather> {
        self.predictions
            .iter()
            .min_by_key(|(t, _)| (*t as i64 - offset as i64).unsigned_abs())
            .map(|(_, w)| w)
    }

    /// Get the worst predicted severity across all time points.
    pub fn worst_predicted_severity(&self) -> Severity {
        self.predictions
            .iter()
            .map(|(_, w)| w.overall_severity().level())
            .max()
            .and_then(Severity::from_level)
            .unwrap_or(Severity::Calm)
    }

    pub fn prediction_count(&self) -> usize {
        self.predictions.len()
    }

    /// Predictions sorted by time.
    pub fn sorted_predictions(&self) -> Vec<(u64, &Weather)> {
        let mut v: Vec<_> = self.predictions.iter().map(|(t, w)| (*t, w)).collect();
        v.sort_by_key(|(t, _)| *t);
        v
    }
}

impl Default for Forecast {
    fn default() -> Self {
        Self::new()
    }
}

/// An adverse event — a storm hitting the system.
#[derive(Clone, Debug, PartialEq)]
pub struct Storm {
    pub name: String,
    pub severity: Severity,
    pub affected_locations: Vec<String>,
    pub estimated_duration_ticks: u64,
    pub started_at: u64,
}

impl Storm {
    pub fn new(name: &str, severity: Severity, started_at: u64, duration: u64) -> Self {
        Self {
            name: name.to_string(),
            severity,
            affected_locations: Vec::new(),
            estimated_duration_ticks: duration,
            started_at,
        }
    }

    /// Add a location affected by this storm.
    pub fn affect(&mut self, location: &str) {
        if !self.affected_locations.contains(&location.to_string()) {
            self.affected_locations.push(location.to_string());
        }
    }

    /// Is a location affected?
    pub fn is_affected(&self, location: &str) -> bool {
        self.affected_locations.iter().any(|l| l == location)
    }

    /// Is the storm still active at the given tick?
    pub fn is_active_at(&self, tick: u64) -> bool {
        tick >= self.started_at && tick < self.started_at + self.estimated_duration_ticks
    }

    /// Fraction of storm elapsed at a given tick (0.0 to 1.0).
    pub fn progress_at(&self, tick: u64) -> f64 {
        if tick < self.started_at {
            return 0.0;
        }
        let elapsed = tick - self.started_at;
        (elapsed as f64 / self.estimated_duration_ticks as f64).min(1.0)
    }

    pub fn affected_count(&self) -> usize {
        self.affected_locations.len()
    }
}

/// Long-term environmental patterns for a region.
#[derive(Clone, Debug)]
pub struct Climate {
    pub region: String,
    /// Average condition values.
    averages: HashMap<String, f64>,
    /// Known storm frequency (storms per N ticks).
    pub storm_frequency: f64,
}

impl Climate {
    pub fn new(region: &str, storm_frequency: f64) -> Self {
        Self {
            region: region.to_string(),
            averages: HashMap::new(),
            storm_frequency,
        }
    }

    /// Set average value for a condition.
    pub fn set_average(&mut self, condition: &str, value: f64) {
        self.averages.insert(condition.to_string(), value);
    }

    /// Get average value for a condition.
    pub fn average(&self, condition: &str) -> Option<f64> {
        self.averages.get(condition).copied()
    }

    /// Is a given value anomalous (more than 2x the average)?
    pub fn is_anomalous(&self, condition: &str, value: f64) -> bool {
        self.averages
            .get(condition)
            .map(|&avg| value > avg * 2.0 || (avg > 0.0 && value < avg * 0.5))
            .unwrap_or(false)
    }

    /// Expected time between storms.
    pub fn expected_storm_interval(&self) -> f64 {
        if self.storm_frequency > 0.0 {
            1.0 / self.storm_frequency
        } else {
            f64::INFINITY
        }
    }

    pub fn condition_count(&self) -> usize {
        self.averages.len()
    }
}

/// Detects changes in environmental conditions over time.
#[derive(Clone, Debug)]
pub struct WeatherVane {
    /// Previous condition readings.
    previous: HashMap<String, f64>,
    /// Change threshold to consider significant.
    threshold: f64,
}

impl WeatherVane {
    pub fn new(threshold: f64) -> Self {
        Self {
            previous: HashMap::new(),
            threshold,
        }
    }

    /// Update readings and return which conditions changed significantly.
    pub fn update(&mut self, conditions: &HashMap<String, f64>) -> Vec<String> {
        let mut changed = Vec::new();
        for (name, &value) in conditions {
            if let Some(&prev) = self.previous.get(name) {
                if (value - prev).abs() > self.threshold {
                    changed.push(name.clone());
                }
            }
            self.previous.insert(name.clone(), value);
        }
        changed
    }

    /// Get the current reading for a condition.
    pub fn current(&self, name: &str) -> Option<f64> {
        self.previous.get(name).copied()
    }

    /// Reset all readings.
    pub fn reset(&mut self) {
        self.previous.clear();
    }
}

/// Protection strategy during adverse events.
#[derive(Clone, Debug)]
pub struct StormShelter {
    /// Locations that provide shelter.
    shelters: HashMap<String, ShelterInfo>,
}

#[derive(Clone, Debug)]
struct ShelterInfo {
    capacity: u32,
    current_occupants: u32,
    protected_conditions: Vec<String>,
}

impl StormShelter {
    pub fn new() -> Self {
        Self {
            shelters: HashMap::new(),
        }
    }

    /// Register a shelter location.
    pub fn register(&mut self, name: &str, capacity: u32, protected_conditions: Vec<String>) {
        self.shelters.insert(
            name.to_string(),
            ShelterInfo {
                capacity,
                current_occupants: 0,
                protected_conditions,
            },
        );
    }

    /// Enter a shelter. Returns false if full.
    pub fn enter(&mut self, name: &str) -> bool {
        if let Some(shelter) = self.shelters.get_mut(name) {
            if shelter.current_occupants < shelter.capacity {
                shelter.current_occupants += 1;
                return true;
            }
        }
        false
    }

    /// Leave a shelter.
    pub fn leave(&mut self, name: &str) -> bool {
        if let Some(shelter) = self.shelters.get_mut(name) {
            if shelter.current_occupants > 0 {
                shelter.current_occupants -= 1;
                return true;
            }
        }
        false
    }

    /// Check if a shelter protects against a specific condition.
    pub fn protects_against(&self, name: &str, condition: &str) -> bool {
        self.shelters
            .get(name)
            .map(|s| s.protected_conditions.iter().any(|c| c == condition))
            .unwrap_or(false)
    }

    /// Available capacity of a shelter.
    pub fn available_capacity(&self, name: &str) -> u32 {
        self.shelters
            .get(name)
            .map(|s| s.capacity - s.current_occupants)
            .unwrap_or(0)
    }

    pub fn shelter_count(&self) -> usize {
        self.shelters.len()
    }
}

impl Default for StormShelter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_levels() {
        assert_eq!(Severity::Calm.level(), 0);
        assert_eq!(Severity::Critical.level(), 4);
        assert_eq!(Severity::from_level(2), Some(Severity::Moderate));
        assert_eq!(Severity::from_level(10), None);
    }

    #[test]
    fn test_severity_is_dangerous() {
        assert!(!Severity::Calm.is_dangerous());
        assert!(!Severity::Mild.is_dangerous());
        assert!(Severity::Severe.is_dangerous());
        assert!(Severity::Critical.is_dangerous());
    }

    #[test]
    fn test_weather_set_get_condition() {
        let mut w = Weather::new("room-1", 0);
        w.set_condition(Condition::new("latency", 50.0, Severity::Mild));
        assert_eq!(w.condition_count(), 1);
        let cond = w.get_condition("latency").unwrap();
        assert_eq!(cond.value, 50.0);
    }

    #[test]
    fn test_weather_overall_severity() {
        let mut w = Weather::new("room-1", 0);
        w.set_condition(Condition::new("latency", 50.0, Severity::Mild));
        w.set_condition(Condition::new("packet_loss", 0.3, Severity::Severe));
        assert_eq!(w.overall_severity(), Severity::Severe);
    }

    #[test]
    fn test_weather_is_safe() {
        let mut w = Weather::new("room-1", 0);
        w.set_condition(Condition::new("load", 0.5, Severity::Mild));
        assert!(w.is_safe());
        w.set_condition(Condition::new("load", 0.99, Severity::Critical));
        assert!(!w.is_safe());
    }

    #[test]
    fn test_weather_empty_is_calm() {
        let w = Weather::new("empty-room", 0);
        assert_eq!(w.overall_severity(), Severity::Calm);
        assert!(w.is_safe());
    }

    #[test]
    fn test_forecast_predict_and_at() {
        let mut f = Forecast::new();
        let w1 = Weather::new("room-1", 10);
        f.predict(10, w1);
        assert!(f.at(10).is_some());
        assert!(f.at(20).is_none());
    }

    #[test]
    fn test_forecast_nearest_to() {
        let mut f = Forecast::new();
        f.predict(10, Weather::new("r", 10));
        f.predict(20, Weather::new("r", 20));
        let nearest = f.nearest_to(15);
        assert!(nearest.is_some());
    }

    #[test]
    fn test_forecast_worst_severity() {
        let mut f = Forecast::new();
        let mut w1 = Weather::new("r", 5);
        w1.set_condition(Condition::new("x", 1.0, Severity::Mild));
        let mut w2 = Weather::new("r", 10);
        w2.set_condition(Condition::new("x", 1.0, Severity::Critical));
        f.predict(5, w1);
        f.predict(10, w2);
        assert_eq!(f.worst_predicted_severity(), Severity::Critical);
    }

    #[test]
    fn test_forecast_sorted() {
        let mut f = Forecast::new();
        f.predict(30, Weather::new("r", 30));
        f.predict(10, Weather::new("r", 10));
        f.predict(20, Weather::new("r", 20));
        let sorted = f.sorted_predictions();
        assert_eq!(sorted[0].0, 10);
        assert_eq!(sorted[1].0, 20);
        assert_eq!(sorted[2].0, 30);
    }

    #[test]
    fn test_storm_affect_and_active() {
        let mut s = Storm::new("load-spike", Severity::Severe, 100, 50);
        s.affect("room-1");
        s.affect("room-2");
        assert!(s.is_affected("room-1"));
        assert!(!s.is_affected("room-3"));
        assert!(s.is_active_at(120));
        assert!(!s.is_active_at(50));
        assert!(!s.is_active_at(160));
    }

    #[test]
    fn test_storm_progress() {
        let s = Storm::new("storm", Severity::Moderate, 0, 100);
        assert_eq!(s.progress_at(0), 0.0);
        assert!((s.progress_at(50) - 0.5).abs() < 1e-10);
        assert_eq!(s.progress_at(200), 1.0);
    }

    #[test]
    fn test_storm_no_duplicate_locations() {
        let mut s = Storm::new("s", Severity::Mild, 0, 10);
        s.affect("a");
        s.affect("a");
        assert_eq!(s.affected_count(), 1);
    }

    #[test]
    fn test_climate_average_and_anomaly() {
        let mut c = Climate::new("region-1", 0.1);
        c.set_average("latency", 50.0);
        assert_eq!(c.average("latency"), Some(50.0));
        assert!(!c.is_anomalous("latency", 60.0));
        assert!(c.is_anomalous("latency", 150.0));
        assert!(c.is_anomalous("latency", 10.0));
    }

    #[test]
    fn test_climate_storm_interval() {
        let c = Climate::new("r", 0.1);
        assert!((c.expected_storm_interval() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_climate_no_storms() {
        let c = Climate::new("r", 0.0);
        assert_eq!(c.expected_storm_interval(), f64::INFINITY);
    }

    #[test]
    fn test_weather_vane_detects_change() {
        let mut vane = WeatherVane::new(5.0);
        let mut readings = HashMap::new();
        readings.insert("temp".to_string(), 20.0);
        let changed = vane.update(&readings);
        assert!(changed.is_empty()); // first reading, no previous

        readings.insert("temp".to_string(), 30.0);
        let changed = vane.update(&readings);
        assert!(changed.contains(&"temp".to_string()));
    }

    #[test]
    fn test_weather_vane_below_threshold() {
        let mut vane = WeatherVane::new(10.0);
        let mut readings = HashMap::new();
        readings.insert("temp".to_string(), 20.0);
        vane.update(&readings);
        readings.insert("temp".to_string(), 25.0);
        let changed = vane.update(&readings);
        assert!(!changed.contains(&"temp".to_string()));
    }

    #[test]
    fn test_weather_vane_reset() {
        let mut vane = WeatherVane::new(1.0);
        let mut readings = HashMap::new();
        readings.insert("temp".to_string(), 20.0);
        vane.update(&readings);
        vane.reset();
        assert_eq!(vane.current("temp"), None);
    }

    #[test]
    fn test_storm_shelter_enter_leave() {
        let mut shelter = StormShelter::new();
        shelter.register("safe-room", 2, vec!["load".to_string()]);
        assert!(shelter.enter("safe-room"));
        assert!(shelter.enter("safe-room"));
        assert!(!shelter.enter("safe-room")); // full
        assert!(shelter.leave("safe-room"));
        assert_eq!(shelter.available_capacity("safe-room"), 1);
    }

    #[test]
    fn test_storm_shelter_protects() {
        let mut shelter = StormShelter::new();
        shelter.register("s1", 5, vec!["load".to_string(), "latency".to_string()]);
        assert!(shelter.protects_against("s1", "load"));
        assert!(!shelter.protects_against("s1", "fire"));
    }

    #[test]
    fn test_storm_shelter_nonexistent() {
        let shelter = StormShelter::new();
        assert_eq!(shelter.available_capacity("ghost"), 0);
        assert!(!shelter.protects_against("ghost", "anything"));
    }
}
