//! Brightness value conversion utilities

use crate::config::BrightnessValue;

/// Convert raw value to display value based on brightness value type
#[allow(dead_code)]
pub fn to_display(raw: i64, max: i64, brightness_value: BrightnessValue) -> f64 {
    match brightness_value {
        BrightnessValue::Raw(_) => raw as f64,
        BrightnessValue::Percent(_) => {
            (raw as f64 / max as f64) * 100.0
        }
    }
}

/// Convert display value to raw value based on brightness value type
#[allow(dead_code)]
pub fn from_display(display: f64, max: i64, brightness_value: BrightnessValue) -> i64 {
    match brightness_value {
        BrightnessValue::Raw(_) => display as i64,
        BrightnessValue::Percent(_) => {
            ((display / 100.0) * max as f64) as i64
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
        assert_eq!(to_display(500, max, BrightnessValue::Percent(50)) as i64, 50);
        assert_eq!(from_display(50.0, max, BrightnessValue::Percent(50)), 500);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(1500, 0, 1000), 1000);
        assert_eq!(clamp(-100, 0, 1000), 0);
        assert_eq!(clamp(500, 0, 1000), 500);
    }
}
