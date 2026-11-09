use clap::{Parser, ValueEnum};

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

    /// Field to operate on
    #[arg(short, long, value_enum, default_value = "brightness")]
    pub field: Field,

    /// Controller selection mode
    #[arg(long, value_enum, default_value = "auto")]
    pub controller_mode: ControllerMode,

    /// Specific controller name
    #[arg(short, long)]
    pub controller: Option<String>,

    /// Value mode (raw, percent, exponential)
    #[arg(long, value_enum, default_value = "percent")]
    pub value_mode: ValueMode,

    /// Brightness value (for set/add/sub operations)
    #[arg(short, long)]
    pub value: Option<i64>,

    /// Transition time in microseconds
    #[arg(short, long, default_value = "0")]
    pub usec: i64,

    /// Verbosity level (0-8)
    #[arg(long, default_value = "0")]
    pub verbose: u8,
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
pub enum Field {
    /// Brightness value
    Brightness,
    /// Maximum brightness
    Max,
    /// Minimum capability
    MinCap,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum ControllerMode {
    /// Auto-detect best controller
    Auto,
    /// Apply to all controllers
    All,
    /// Use specified controller
    Specify,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum ValueMode {
    /// Raw sysfs values
    Raw,
    /// Percentage (0-100)
    Percent,
    /// Exponential percentage
    PercentExponential,
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
