use crate::config::{Config, Target, ControllerMode, BrightnessValue};
use crate::ddcutil;
use crate::error::{BrilloError, Result};
use log::{debug, info};
use std::fs;
use std::path::Path;

pub struct Controller {
    pub name: String,
    #[allow(dead_code)]
    pub target: Target,
    pub max_brightness: i64,
    pub is_ddc: bool,
}

impl Controller {
    /// Detect and select the best controller based on config
    pub fn detect(config: &Config) -> Result<Self> {
        match config.controller_mode {
            ControllerMode::Auto => Self::auto_detect(config),
            ControllerMode::Specify => {
                let name = config.controller.clone()
                    .ok_or_else(|| BrilloError::ControllerNotFound("No controller specified".into()))?;
                Self::from_name(&name, config.target.unwrap_or(Target::Backlight))
            }
        }
    }

    /// Auto-detect the best controller (highest max_brightness)
    fn auto_detect(config: &Config) -> Result<Self> {
        if config.target.unwrap_or(Target::Backlight) == Target::Monitor {
            return ddcutil::detect_best_monitor();
        }

        let sys_prefix = config.sys_prefix();
        let mut best_controller: Option<Self> = None;
        let mut max_max_brightness = 0;

        if !Path::new(sys_prefix).exists() {
            return Err(BrilloError::ControllerNotFound(
                format!("Sysfs path {} not found", sys_prefix)
            ));
        }

        for entry in fs::read_dir(sys_prefix)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            if let Ok(max) = Self::read_max_brightness(&name, sys_prefix, false) {
                if max > max_max_brightness {
                    max_max_brightness = max;
                    best_controller = Some(Controller {
                        name: name.clone(),
                        target: config.target.unwrap_or(Target::Backlight),
                        max_brightness: max,
                        is_ddc: false,
                    });
                }
            }
        }

        best_controller.ok_or_else(|| {
            BrilloError::ControllerNotFound("No accessible controller found".into())
        })
    }

    /// Create controller from specific name
    fn from_name(name: &str, target: Target) -> Result<Self> {
        let (sys_prefix, is_ddc) = match target {
            Target::Monitor => ("", true),
            Target::Keyboard => ("/sys/class/leds", false),
            Target::Backlight => ("/sys/class/backlight", false),
        };

        let max_brightness = if is_ddc {
            100
        } else {
            Self::read_max_brightness(name, sys_prefix, true)?
        };

        Ok(Controller {
            name: name.to_string(),
            target,
            max_brightness,
            is_ddc,
        })
    }

    /// Read max_brightness from sysfs
    fn read_max_brightness(name: &str, prefix: &str, is_ddc: bool) -> Result<i64> {
        if is_ddc {
            return Ok(100);
        }

        let path = format!("{}/{}/max_brightness", prefix, name);
        let content = fs::read_to_string(&path)
            .map_err(|e| BrilloError::SysfsError(format!("Reading {}: {}", path, e)))?;

        content.trim().parse()
            .map_err(|e| BrilloError::SysfsError(format!("Parsing max_brightness: {}", e)))
    }

    /// Get current brightness value
    pub fn get_brightness(&self, config: &Config) -> Result<()> {
        let raw_value = if self.is_ddc {
            ddcutil::get_brightness(&self.name)?
        } else {
            let path = format!("{}/{}/brightness", config.sys_prefix(), self.name);
            let content = fs::read_to_string(&path)
                .map_err(|e| BrilloError::SysfsError(format!("Reading {}: {}", path, e)))?;
            content.trim().parse()
                .map_err(|e| BrilloError::SysfsError(format!("Parsing brightness: {}", e)))?
        };

        // Display value is always shown as percentage for user-friendliness
        let percent = (raw_value as f64 / self.max_brightness as f64 * 100.0) as i64;
        println!("{:.2}", percent as f64 / 100.0);

        Ok(())
    }

    /// Set brightness to specific value
    pub fn set_brightness(&self, config: &Config) -> Result<()> {
        let brightness_value = config.value
            .ok_or_else(|| BrilloError::ConfigError("No value specified".into()))?;

        let raw_value = brightness_value.to_raw(self.max_brightness);

        let clamped = raw_value.clamp(0, self.max_brightness);

        if self.is_ddc {
            ddcutil::set_brightness(&self.name, clamped)?;
        } else {
            let path = format!("{}/{}/brightness", config.sys_prefix(), self.name);
            fs::write(&path, clamped.to_string())
                .map_err(|e| BrilloError::SysfsError(format!("Writing {}: {}", path, e)))?;
        }

        Ok(())
    }

    /// Adjust brightness (add or subtract)
    pub fn adjust_brightness(&self, config: &Config, add: bool) -> Result<()> {
        let current = if self.is_ddc {
            ddcutil::get_brightness(&self.name)?
        } else {
            let path = format!("{}/{}/brightness", config.sys_prefix(), self.name);
            fs::read_to_string(&path)?
                .trim()
                .parse()
                .map_err(|e| BrilloError::SysfsError(format!("Parsing brightness: {}", e)))?
        };

        let adjustment_value = config.value
            .ok_or_else(|| BrilloError::ConfigError("No value specified".into()))?;

        // Convert adjustment to raw value
        let adjustment = adjustment_value.to_raw(self.max_brightness);

        let new_value = if add {
            current + adjustment
        } else {
            current - adjustment
        };

        let mut new_config = config.clone();
        new_config.value = Some(BrightnessValue::Raw(new_value));
        new_config.set = true;

        self.set_brightness(&new_config)
    }

    /// Save current brightness to cache
    pub fn save_brightness(&self, config: &Config) -> Result<()> {
        let cache_dir = config.cache_dir();
        fs::create_dir_all(&cache_dir)?;

        let cache_file = format!("{}/{}.brightness", cache_dir, self.name);

        // Get current brightness before saving
        let current = if self.is_ddc {
            ddcutil::get_brightness(&self.name)?
        } else {
            let path = format!("{}/{}/brightness", config.sys_prefix(), self.name);
            fs::read_to_string(&path)?
                .trim()
                .parse()
                .map_err(|e| BrilloError::SysfsError(format!("Parsing brightness: {}", e)))?
        };

        fs::write(&cache_file, current.to_string())?;
        info!("Saving current brightness to {}", cache_file);
        Ok(())
    }

    /// Restore brightness from cache
    pub fn restore_brightness(&self, config: &Config) -> Result<()> {
        let cache_dir = config.cache_dir();
        let cache_file = format!("{}/{}.brightness", cache_dir, self.name);

        let content = fs::read_to_string(&cache_file)
            .map_err(|_| BrilloError::ConfigError("No saved brightness found".into()))?;

        let value: i64 = content.trim().parse()
            .map_err(|e| BrilloError::ConfigError(format!("Parsing saved value: {}", e)))?;

        let mut new_config = config.clone();
        new_config.value = Some(BrightnessValue::Raw(value));
        new_config.set = true;

        self.set_brightness(&new_config)?;
        info!("Restored brightness for {} to {}", self.name, value);
        Ok(())
    }
}

/// List all available controllers
pub fn list_controllers(config: &Config) -> Result<()> {
    let target = config.target.unwrap_or(Target::Backlight);

    if target == Target::Monitor {
        return ddcutil::list_monitors();
    }

    let sys_prefix = match target {
        Target::Backlight => "/sys/class/backlight",
        Target::Keyboard => "/sys/class/leds",
        Target::Monitor => "",
    };

    if !Path::new(sys_prefix).exists() {
        info!("No {} devices found",
            match target {
                Target::Backlight => "backlight",
                Target::Keyboard => "keyboard LED",
                Target::Monitor => "monitor",
            }
        );
        return Ok(());
    }

    for entry in fs::read_dir(sys_prefix)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with('.') {
            println!("{}", name);
        }
    }

    Ok(())
}

/// Save brightness for all monitors (DDC/CI workflow step 1)
pub fn save_all_monitors(config: &Config) -> Result<()> {
    let cache_dir = config.cache_dir();
    fs::create_dir_all(&cache_dir)?;

    let saved = ddcutil::save_all_monitors_brightness(&cache_dir)?;

    info!("Saved brightness for {} monitors:", saved.len());
    for (monitor, brightness) in &saved {
        info!("  Monitor {}: {}%", monitor, brightness);
    }

    Ok(())
}

/// Set all monitors to minimum brightness (DDC/CI workflow step 2)
pub fn dim_all_monitors(_config: &Config, target_percent: i64) -> Result<()> {
    let results = ddcutil::set_all_monitors_brightness(target_percent)?;

    info!("Set {} monitors to {}% brightness:", results.len(), target_percent);
    for (monitor, brightness) in &results {
        info!("  Monitor {}: {}%", monitor, brightness);
    }

    Ok(())
}

/// Restore all monitors to saved brightness (DDC/CI workflow step 3)
pub fn restore_all_monitors(config: &Config) -> Result<()> {
    let cache_dir = config.cache_dir();

    let restored = ddcutil::restore_all_monitors_brightness(&cache_dir)?;

    info!("Restored brightness for {} monitors:", restored.len());
    for (monitor, brightness, cache_file) in &restored {
        info!("  Monitor {}: {}%  -  from {}", monitor, brightness, cache_file);
    }

    Ok(())
}
