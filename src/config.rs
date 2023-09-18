use std::io;
use std::path::{Path};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MycochipConfig {
    pub devices: std::collections::HashMap<String, Device>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub mcu: String,
    pub firmware: String,
    pub eeprom: Option<Vec<u8>>,

    #[serde(default = "Vec::new")]
    pub peers: Vec<String>,
}

pub fn load(config_file_path_str: &str) -> Result<MycochipConfig, io::Error> {
    let config_file_path = Path::new(config_file_path_str);

    // Check that the config file exists
    if !config_file_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Config file not found"));
    }

    // Load the config file
    let yaml = std::fs::read_to_string(config_file_path).unwrap();
    let mut config: MycochipConfig = serde_yaml::from_str(yaml.as_str()).unwrap();

    // FIXME: make error messages nicer when the config is wrong

    let config_dir = config_file_path.parent().unwrap();
    for (_, device) in &mut config.devices {
        let raw_path = Path::new(&device.firmware);

        let firmware_path = if raw_path.is_absolute() {
            raw_path.to_owned()
        } else {
            config_dir.join(raw_path)
        };

        if !firmware_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("Firmware file not found: {}", firmware_path.display())));
        }

        device.firmware = firmware_path.to_str().unwrap().to_owned();
    }

    return Ok(config);
}
