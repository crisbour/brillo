use clap::{Parser, ValueEnum};
use std::str::FromStr;

#[derive(Parser, Debug, Clone)]
#[command(name = "brillo")]
#[command(about = "Control brightness of backlight and LED devices")]
#[command(version)]
pub struct Config {
    // Operations (mutually exclusive)
    /// Get current brightness
    #[arg(short = 'G', long, conflicts_with_all = ["set", "add", "sub", "save", "restore", "list_controllers"])]
    pub get: bool,

    /// Set brightness (requires -v)
    #[arg(short = 'S', long, conflicts_with_all = ["get", "add", "sub", "save", "restore", "list_controllers"])]
    pub set: bool,

    /// Add to brightness (requires -v)
    #[arg(short = 'A', long, conflicts_with_all = ["get", "set", "sub", "save", "restore", "list_controllers"])]
    pub add: bool,

    /// Subtract from brightness (requires -v)
    #[arg(short = 'U', long, conflicts_with_all = ["get", "set", "add", "save", "restore", "list_controllers"])]
    pub sub: bool,

    /// Save current brightness
    #[arg(short = 'O', long, conflicts_with_all = ["get", "set", "add", "sub", "restore", "list_controllers"])]
    pub save: bool,

    /// Restore saved brightness
    #[arg(short = 'I', long, conflicts_with_all = ["get", "set", "add", "sub", "save", "list_controllers", "monitor_workflow"])]
    pub restore: bool,

    /// List all controllers
    #[arg(short = 'L', long, conflicts_with_all = ["get", "set", "add", "sub", "save", "restore", "monitor_workflow"])]
    pub list_controllers: bool,

    /// Monitor workflow: save, dim to 5%, then restore
    #[arg(long = "monitor-workflow", conflicts_with_all = ["get", "set", "add", "sub", "save", "restore", "list_controllers"])]
    pub monitor_workflow: bool,

    /// Target device type
    #[arg(short, long, value_enum)]
    pub target: Option<Target>,

    /// Controller selection mode
    #[arg(long, value_enum, default_value = "auto")]
    pub controller_mode: ControllerMode,

    /// Specific controller name
    #[arg(short, long)]
    pub controller: Option<String>,

    /// Brightness value (for set/add/sub operations)
    /// Use "50%" for percentage or "50" for raw value
    #[arg(short, long)]
    pub value: Option<BrightnessValue>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum Target {
    /// Backlight devices
    Backlight,
    /// Keyboard LEDs
    Keyboard,
    /// External monitors (DDC/CI)
    Monitor,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum ControllerMode {
    /// Auto-detect all controllers
    Auto,
    /// Use specified controller
    Specify,
}

/// Brightness value that can be either raw or percentage
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrightnessValue {
    /// Raw sysfs value
    Raw(i64),
    /// Percentage (0-100)
    Percent(i64),
}

impl FromStr for BrightnessValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.ends_with('%') {
            // Percentage format: "50%"
            let num_str = s.trim_end_matches('%').trim();
            let value = num_str.parse::<i64>()
                .map_err(|e| format!("Invalid percentage value '{}': {}", s, e))?;

            if value < 0 || value > 100 {
                return Err(format!("Percentage must be between 0 and 100, got {}", value));
            }

            Ok(BrightnessValue::Percent(value))
        } else {
            // Raw format: "50"
            let value = s.parse::<i64>()
                .map_err(|e| format!("Invalid raw value '{}': {}", s, e))?;

            if value < 0 {
                return Err(format!("Raw value must be non-negative, got {}", value));
            }

            Ok(BrightnessValue::Raw(value))
        }
    }
}

impl BrightnessValue {
    /// Get the raw value, converting from percentage if needed
    pub fn to_raw(&self, max: i64) -> i64 {
        match self {
            BrightnessValue::Raw(v) => *v,
            BrightnessValue::Percent(v) => (*v as f64 / 100.0 * max as f64) as i64,
        }
    }

    /// Get the value as percentage (0-100)
    pub fn to_percent(&self, max: i64) -> f64 {
        match self {
            BrightnessValue::Raw(v) => (*v as f64 / max as f64) * 100.0,
            BrightnessValue::Percent(v) => *v as f64,
        }
    }

    /// Get the underlying i64 value
    pub fn value(&self) -> i64 {
        match self {
            BrightnessValue::Raw(v) => *v,
            BrightnessValue::Percent(v) => *v,
        }
    }

    /// Check if this is a percentage value
    pub fn is_percent(&self) -> bool {
        matches!(self, BrightnessValue::Percent(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Get,
    Set,
    Add,
    Sub,
    Save,
    Restore,
    List,
    MonitorWorkflow,
}

impl Config {
    /// Get the effective operation from flags
    pub fn operation(&self) -> Operation {
        if self.monitor_workflow {
            Operation::MonitorWorkflow
        } else if self.list_controllers {
            Operation::List
        } else if self.restore {
            Operation::Restore
        } else if self.save {
            Operation::Save
        } else if self.sub {
            Operation::Sub
        } else if self.add {
            Operation::Add
        } else if self.set {
            Operation::Set
        } else {
            Operation::Get
        }
    }

    pub fn sys_prefix(&self) -> &'static str {
        match self.target.unwrap_or(Target::Backlight) {
            Target::Backlight => "/sys/class/backlight",
            Target::Keyboard => "/sys/class/leds",
            Target::Monitor => "",
        }
    }

    pub fn cache_dir(&self) -> String {
        let base_cache = std::env::var("XDG_CACHE_HOME")
            .unwrap_or_else(|_| format!("{}/.cache", std::env::var("HOME").unwrap_or_default()));
        format!("{}/brillo", base_cache)
    }
}
