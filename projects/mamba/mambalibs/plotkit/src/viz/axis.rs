//! Axis configuration and nice-tick algorithm.

/// Axis configuration.
#[derive(Debug, Clone)]
pub struct Axis {
    pub label: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub tick_count: usize,
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            label: None,
            min: None,
            max: None,
            tick_count: 5,
        }
    }
}

/// Computed tick positions and labels for rendering.
#[derive(Debug, Clone)]
pub struct TickInfo {
    pub min: f64,
    pub max: f64,
    pub ticks: Vec<f64>,
    pub step: f64,
}

/// Compute "nice" tick marks for an axis range.
///
/// Based on the Wilkinson algorithm: finds a "nice" step size (1, 2, 5 × 10^k)
/// and rounds min/max to nice boundaries.
pub fn nice_ticks(data_min: f64, data_max: f64, tick_count: usize) -> TickInfo {
    if (data_max - data_min).abs() < 1e-15 {
        // Constant data
        let v = data_min;
        return TickInfo {
            min: v - 1.0,
            max: v + 1.0,
            ticks: vec![v - 1.0, v, v + 1.0],
            step: 1.0,
        };
    }

    let range = data_max - data_min;
    let rough_step = range / tick_count.max(1) as f64;
    let mag = 10.0_f64.powf(rough_step.log10().floor());
    let norm = rough_step / mag;

    let nice_step = if norm <= 1.5 {
        1.0 * mag
    } else if norm <= 3.5 {
        2.0 * mag
    } else if norm <= 7.5 {
        5.0 * mag
    } else {
        10.0 * mag
    };

    let nice_min = (data_min / nice_step).floor() * nice_step;
    let nice_max = (data_max / nice_step).ceil() * nice_step;

    let mut ticks = Vec::new();
    let mut val = nice_min;
    while val <= nice_max + nice_step * 0.01 {
        ticks.push(val);
        val += nice_step;
    }

    TickInfo {
        min: nice_min,
        max: nice_max,
        ticks,
        step: nice_step,
    }
}

/// Format a tick value for display.
pub fn format_tick(val: f64) -> String {
    if val.abs() < 1e-10 {
        "0".to_string()
    } else if val.abs() >= 1e6 || (val.abs() < 0.01 && val.abs() > 0.0) {
        format!("{:.1e}", val)
    } else if (val - val.round()).abs() < 1e-10 {
        format!("{:.0}", val)
    } else {
        format!("{:.1}", val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nice_ticks_basic() {
        let info = nice_ticks(0.0, 100.0, 5);
        assert!(info.min <= 0.0);
        assert!(info.max >= 100.0);
        assert!(info.ticks.len() >= 3);
        // Step should be "nice" (20 for 0-100 with 5 ticks)
        assert!([10.0, 20.0, 25.0, 50.0].contains(&info.step));
    }

    #[test]
    fn test_nice_ticks_small_range() {
        let info = nice_ticks(0.1, 0.9, 5);
        assert!(info.step > 0.0);
        assert!(info.ticks.len() >= 3);
    }

    #[test]
    fn test_nice_ticks_negative() {
        let info = nice_ticks(-50.0, 50.0, 5);
        assert!(info.min <= -50.0);
        assert!(info.max >= 50.0);
    }

    #[test]
    fn test_nice_ticks_constant() {
        let info = nice_ticks(5.0, 5.0, 5);
        assert_eq!(info.ticks.len(), 3);
    }

    #[test]
    fn test_format_tick() {
        assert_eq!(format_tick(0.0), "0");
        assert_eq!(format_tick(100.0), "100");
        assert_eq!(format_tick(3.5), "3.5");
    }
}
