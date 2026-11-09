//! DDC/CI monitor brightness control via ddcutil binary
//!
//! Calls the `ddcutil` command-line tool which interfaces with libddcutil.
//! This approach avoids version compatibility issues with libddcutil bindings.

use crate::controller::Controller;
use crate::config::Target;
use crate::error::{BrilloError, Result};
use std::process::Command;

const DDCUTIL_BIN: &str = "ddcutil";

/// Represents a detected monitor
#[derive(Debug, Clone)]
pub struct Monitor {
    pub display_id: String,
    pub device_path: String,
    pub model: String,
    pub serial: String,
}

/// Check if ddcutil is available
#[allow(dead_code)]
pub fn is_available() -> bool {
    Command::new(DDCUTIL_BIN)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// List all DDC/CI capable monitors
pub fn list_monitors() -> Result<()> {
    let output = Command::new(DDCUTIL_BIN)
        .arg("detect")
        .output()
        .map_err(|e| BrilloError::DdcutilError(format!("Failed to run ddcutil: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BrilloError::DdcutilError(stderr.to_string()));
    }

    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

/// Detect all monitors by parsing ddcutil detect output
pub fn detect_all_monitors() -> Result<Vec<Monitor>> {
    let output = Command::new(DDCUTIL_BIN)
        .arg("detect")
        .output()
        .map_err(|e| BrilloError::DdcutilError(format!("Failed to run ddcutil: {}", e)))?;

    if !output.status.success() {
        return Err(BrilloError::ControllerNotFound(
            "No DDC/CI monitors found".into()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut monitors = Vec::new();
    let mut current_monitor: Option<Monitor> = None;

    for line in stdout.lines() {
        if line.starts_with("Display ") {
            // Save previous monitor if exists
            if let Some(monitor) = current_monitor.take() {
                monitors.push(monitor);
            }

            // Start new monitor
            if let Some(id) = line.split_whitespace().nth(1) {
                current_monitor = Some(Monitor {
                    display_id: id.to_string(),
                    device_path: String::new(),
                    model: String::new(),
                    serial: String::new(),
                });
            }
        } else if let Some(ref mut monitor) = current_monitor {
            if line.contains("I2C bus:") {
                if let Some(path) = line.split(':').nth(1) {
                    monitor.device_path = path.trim().to_string();
                }
            } else if line.contains("Model:") {
                monitor.model = line.split(':').nth(1)
                    .map(|s| s.trim())
                    .unwrap_or("Unknown")
                    .to_string();
            } else if line.contains("Serial number:") {
                monitor.serial = line.split(':').nth(1)
                    .map(|s| s.trim())
                    .unwrap_or("Unknown")
                    .to_string();
            }
        }
    }

    // Don't forget the last monitor
    if let Some(monitor) = current_monitor {
        monitors.push(monitor);
    }

    if monitors.is_empty() {
        return Err(BrilloError::ControllerNotFound(
            "No monitors detected".into()
        ));
    }

    Ok(monitors)
}

/// Detect the best monitor (first available)
pub fn detect_best_monitor() -> Result<Controller> {
    let monitors = detect_all_monitors()?;
    let monitor = monitors.first()
        .ok_or_else(|| BrilloError::ControllerNotFound("No monitors found".into()))?;

    Ok(Controller {
        name: monitor.display_id.clone(),
        target: Target::Monitor,
        max_brightness: 100,
        is_ddc: true,
    })
}

/// Get current brightness from monitor
pub fn get_brightness(display: &str) -> Result<i64> {
    let output = Command::new(DDCUTIL_BIN)
        .args(["getvcp", "10", "--display", display])
        .output()
        .map_err(|e| BrilloError::DdcutilError(format!("Failed to run ddcutil: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BrilloError::DdcutilError(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse: "VCP code 0x10 (Brightness): current value =    85, max value =   100"
    for line in stdout.lines() {
        if line.contains("current value") {
            // Extract value after "current value ="
            if let Some(parts) = line.split("current value").nth(1) {
                if let Some(value_str) = parts.split('=').nth(1) {
                    // Extract the number before any comma
                    let value_str = value_str.split(',').next().unwrap_or(value_str);
                    if let Ok(value) = value_str.trim().parse::<i64>() {
                        return Ok(value);
                    }
                }
            }
        }
    }

    Err(BrilloError::DdcutilError(
        format!("Failed to parse brightness value from ddcutil: {:?}", output).into()
    ))
}

/// Set monitor brightness
pub fn set_brightness(display: &str, value: i64) -> Result<()> {
    let output = Command::new(DDCUTIL_BIN)
        .args([
            "setvcp", "10",
            &value.to_string(),
            "--display", display
        ])
        .output()
        .map_err(|e| BrilloError::DdcutilError(format!("Failed to run ddcutil: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BrilloError::DdcutilError(stderr.to_string()));
    }

    Ok(())
}

/// Get monitor capabilities
#[allow(dead_code)]
pub fn get_capabilities(display: &str) -> Result<String> {
    let output = Command::new(DDCUTIL_BIN)
        .args(["capabilities", "--display", display])
        .output()
        .map_err(|e| BrilloError::DdcutilError(format!("Failed to run ddcutil: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BrilloError::DdcutilError(stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Save brightness for all monitors to individual files
pub fn save_all_monitors_brightness(cache_dir: &str) -> Result<Vec<(String, i64)>> {
    let monitors = detect_all_monitors()?;
    let mut saved = Vec::new();

    for monitor in &monitors {
        let brightness = get_brightness(&monitor.display_id)?;
        saved.push((monitor.display_id.clone(), brightness));

        // Save to file: {cache_dir}/monitor.{display_id}.brightness
        let cache_file = format!("{}/monitor.{}.brightness", cache_dir, monitor.display_id);
        std::fs::write(&cache_file, brightness.to_string())
            .map_err(|e| BrilloError::IoError(e))?;
    }

    Ok(saved)
}

/// Restore brightness for all monitors from saved files
pub fn restore_all_monitors_brightness(cache_dir: &str) -> Result<Vec<(String, i64, String)>> {
    let monitors = detect_all_monitors()?;
    let mut restored = Vec::new();

    for monitor in &monitors {
        let cache_file = format!("{}/monitor.{}.brightness", cache_dir, monitor.display_id);

        let content = std::fs::read_to_string(&cache_file)
            .map_err(|_| BrilloError::ConfigError(
                format!("No saved brightness for monitor {} (run 'brillo -O' first)", monitor.display_id)
            ))?;

        let brightness: i64 = content.trim().parse()
            .map_err(|e| BrilloError::ConfigError(
                format!("Parsing saved value for {}: {}", monitor.display_id, e)
            ))?;

        set_brightness(&monitor.display_id, brightness)?;
        restored.push((monitor.display_id.clone(), brightness, cache_file));
    }

    Ok(restored)
}

/// Set brightness for all monitors to a specific value
pub fn set_all_monitors_brightness(value: i64) -> Result<Vec<(String, i64)>> {
    let monitors = detect_all_monitors()?;
    let mut results = Vec::new();

    for monitor in &monitors {
        set_brightness(&monitor.display_id, value)?;
        results.push((monitor.display_id.clone(), value));
    }

    Ok(results)
}

/// Get current brightness for all monitors
#[allow(dead_code)]
pub fn get_all_monitors_brightness() -> Result<Vec<(String, i64)>> {
    let monitors = detect_all_monitors()?;
    let mut results = Vec::new();

    for monitor in &monitors {
        let brightness = get_brightness(&monitor.display_id)?;
        results.push((monitor.display_id.clone(), brightness));
    }

    Ok(results)
}
