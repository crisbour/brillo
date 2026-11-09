mod config;
mod controller;
mod value;
mod ddcutil;
mod error;

use anyhow::Result;
use clap::Parser;
use config::{Config, Operation};
use env_logger::Env;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger with default "info" level unless RUST_LOG is set
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    let config = Config::parse();

    // Handle List operation early
    if config.operation() == Operation::List {
        return controller::list_controllers(&config)
            .map_err(|e| anyhow::anyhow!(e));
    }

    // Save/Restore operations: when no target specified, save ALL devices (monitors + backlight + keyboard)
    if matches!(config.operation(), Operation::Save | Operation::Restore) && config.target.is_none() {
        // Save all monitors first
        if ddcutil::is_available() {
            if let Ok(monitors) = ddcutil::detect_all_monitors() {
                if !monitors.is_empty() {
                    let result = match config.operation() {
                        Operation::Save => controller::save_all_monitors(&config),
                        Operation::Restore => controller::restore_all_monitors(&config),
                        _ => Ok(()),
                    };
                    result.map_err(|e| anyhow::anyhow!(e))?;
                }
            }
        }
        
        // Save backlight (laptop display)
        let backlight_config = {
            let mut cfg = config.clone();
            cfg.target = Some(config::Target::Backlight);
            cfg
        };
        
        if let Ok(controller) = controller::Controller::detect(&backlight_config) {
            let result = match config.operation() {
                Operation::Save => controller.save_brightness(&backlight_config),
                Operation::Restore => controller.restore_brightness(&backlight_config),
                _ => Ok(()),
            };
            if let Err(e) = result {
                log::debug!("Failed to save/restore backlight: {}", e);
            }
        }
        
        // Save keyboard backlight
        let keyboard_config = {
            let mut cfg = config.clone();
            cfg.target = Some(config::Target::Keyboard);
            cfg
        };
        
        if let Ok(controller) = controller::Controller::detect(&keyboard_config) {
            let result = match config.operation() {
                Operation::Save => controller.save_brightness(&keyboard_config),
                Operation::Restore => controller.restore_brightness(&keyboard_config),
                _ => Ok(()),
            };
            if let Err(e) = result {
                log::debug!("Failed to save/restore keyboard: {}", e);
            }
        }
        
        return Ok(());
    }

    // For explicit target (monitor, keyboard, backlight), use target-specific logic
    if matches!(config.operation(), Operation::Save | Operation::Restore) {
        if config.target == Some(config::Target::Monitor) {
            let result = match config.operation() {
                Operation::Save => controller::save_all_monitors(&config),
                Operation::Restore => controller::restore_all_monitors(&config),
                _ => Ok(()),
            };
            return result.map_err(|e| anyhow::anyhow!(e));
        }
    }

    // For other operations, work on single controller
    // Use default target (backlight) if not specified
    let config_with_default = if config.target.is_none() {
        let mut cfg = config.clone();
        cfg.target = Some(config::Target::Backlight);
        cfg
    } else {
        config.clone()
    };

    let controller = controller::Controller::detect(&config_with_default)
        .map_err(|e| anyhow::anyhow!(e))?;

    match config.operation() {
        Operation::Get => controller.get_brightness(&config),
        Operation::Set => controller.set_brightness(&config),
        Operation::Add => controller.adjust_brightness(&config, true),
        Operation::Sub => controller.adjust_brightness(&config, false),
        Operation::Save => controller.save_brightness(&config),
        Operation::Restore => controller.restore_brightness(&config),
        Operation::Test => execute_test_workflow(&config),
        Operation::List => Ok(()), // Already handled above
    }
    .map_err(|e| anyhow::anyhow!(e))
}

/// Execute test workflow:
/// 1. Save current brightness for all monitors
/// 2. Dim all monitors to 5%
/// 3. Restore each monitor to its saved value
fn execute_test_workflow(config: &Config) -> crate::error::Result<()> {
    if config.target != Some(config::Target::Monitor) {
        return Err(crate::error::BrilloError::ConfigError(
            "Test workflow requires --target monitor".into()
        ));
    }

    info!("=== Test Workflow ===");

    // Step 1: Save current brightness for all monitors
    info!("Step 1: Saving current brightness for all monitors...");
    controller::save_all_monitors(config)?;

    // Step 2: Dim all monitors to 5%
    info!("Step 2: Dimming all monitors to 5%...");
    controller::dim_all_monitors(config, 5)?;

    // Step 3: Restore saved brightness
    info!("Step 3: Restoring saved brightness values...");
    controller::restore_all_monitors(config)?;

    info!("=== Test Complete ===");
    Ok(())
}
