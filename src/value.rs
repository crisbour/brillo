//! Brightness value conversion utilities

use crate::config::ValueMode;

/// Convert raw value to display value based on mode
#[allow(dead_code)]
pub fn to_display(raw: i64, max: i64, mode: ValueMode) -> f64 {
    match mode {
        ValueMode::Raw => raw as f64,
        ValueMode::Percent => {
            (raw as f64 / max as f64) * 100.0
        }
        ValueMode::PercentExponential => {
            // Exponential curve for more natural brightness perception
            let normalized = raw as f64 / max as f64;
            (normalized * normalized) * 100.0
        }
    }
}

/// Convert display value to raw value based on mode
#[allow(dead_code)]
pub fn from_display(display: f64, max: i64, mode: ValueMode) -> i64 {
    match mode {
        ValueMode::Raw => display as i64,
        ValueMode::Percent => {
            ((display / 100.0) * max as f64) as i64
        }
        ValueMode::PercentExponential => {
            // Inverse of exponential curve
            let normalized = (display / 100.0).sqrt();
            (normalized * max as f64) as i64
        }
    }
}

/// Clamp value to valid range
#[allow(dead_code)]
pub fn clamp(value: i64, min: i64, max: i64) -> i64 {
    value.clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_conversion() {
        let max = 1000;
        assert_eq!(to_display(500, max, ValueMode::Percent) as i64, 50);
        assert_eq!(from_display(50.0, max, ValueMode::Percent), 500);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(1500, 0, 1000), 1000);
        assert_eq!(clamp(-100, 0, 1000), 0);
        assert_eq!(clamp(500, 0, 1000), 500);
    }
}
