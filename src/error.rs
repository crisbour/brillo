use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrilloError {
    #[error("Controller not found: {0}")]
    ControllerNotFound(String),
    
    #[error("DDCUTIL error: {0}")]
    DdcutilError(String),
    
    #[error("Sysfs error: {0}")]
    SysfsError(String),
    
    #[error("Value error: {0}")]
    #[allow(dead_code)]
    ValueError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, BrilloError>;
