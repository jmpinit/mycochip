use std::io;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MycochipConfig {
    pub devices: std::collections::HashMap<String, Device>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub mcu: String,
    pub firmware: String,
    pub peers: Vec<String>,
}

pub fn load(config_file_path: &str) -> Result<MycochipConfig, io::Error> {
    // Check that the config file exists
    if !std::path::Path::new(config_file_path).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Config file not found"));
    }

    // Load the config file
    let yaml = std::fs::read_to_string(config_file_path).unwrap();
    let config:MycochipConfig = serde_yaml::from_str(yaml.as_str()).unwrap();

    // FIXME: make error messages nicer when the config is wrong

    return Ok(config);
}
