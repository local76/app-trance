use winreg::enums::*;
use winreg::RegKey;

#[derive(Clone, Debug)]
pub enum ConfigField {
    Integer {
        name: &'static str,
        label: &'static str,
        min: i32,
        max: i32,
        default: i32,
        value: i32,
    },
    Boolean {
        name: &'static str,
        label: &'static str,
        default: bool,
        value: bool,
    },
}

#[derive(Clone, Debug)]
pub struct ScreensaverConfig {
    pub name: &'static str,
    pub registry_name: &'static str,
    pub binary_name: &'static str,
    pub fields: Vec<ConfigField>,
}

pub struct GlobalConfig {
    pub active_scr: String,
    pub active: bool,
    pub timeout: u32,
}

pub fn get_screensavers() -> Vec<ScreensaverConfig> {
    vec![
        ScreensaverConfig {
            name: "Omaxi Beams",
            registry_name: "OmaxiBeams",
            binary_name: "omaxi-beams.scr",
            fields: vec![
                ConfigField::Integer {
                    name: "BeamCount",
                    label: "Beam Count",
                    min: 2,
                    max: 8,
                    default: 2,
                    value: 2,
                },
                ConfigField::Integer {
                    name: "StarCount",
                    label: "Star Count",
                    min: 10,
                    max: 150,
                    default: 40,
                    value: 40,
                },
                ConfigField::Integer {
                    name: "BeamSpeed",
                    label: "Beam Speed",
                    min: 10,
                    max: 100,
                    default: 35,
                    value: 35,
                },
            ],
        },
        ScreensaverConfig {
            name: "Omaxi Bounce",
            registry_name: "OmaxiBounce",
            binary_name: "omaxi-bounce.scr",
            fields: vec![
                ConfigField::Integer {
                    name: "BeamSpeed",
                    label: "Bounce Speed",
                    min: 10,
                    max: 100,
                    default: 35,
                    value: 35,
                },
                ConfigField::Integer {
                    name: "StarCount",
                    label: "Star Count",
                    min: 10,
                    max: 150,
                    default: 40,
                    value: 40,
                },
            ],
        },
        ScreensaverConfig {
            name: "Omaxi Matrix",
            registry_name: "OmaxiMatrix",
            binary_name: "omaxi-matrix.scr",
            fields: vec![
                ConfigField::Integer {
                    name: "RainDensity",
                    label: "Rain Density",
                    min: 10,
                    max: 150,
                    default: 80,
                    value: 80,
                },
                ConfigField::Integer {
                    name: "RainSpeed",
                    label: "Rain Speed",
                    min: 10,
                    max: 100,
                    default: 40,
                    value: 40,
                },
            ],
        },
        ScreensaverConfig {
            name: "Omaxi Pour",
            registry_name: "OmaxiPour",
            binary_name: "omaxi-pour.scr",
            fields: vec![
                ConfigField::Integer {
                    name: "DropCount",
                    label: "Drop Count",
                    min: 20,
                    max: 150,
                    default: 60,
                    value: 60,
                },
                ConfigField::Integer {
                    name: "DropSpeed",
                    label: "Drop Speed",
                    min: 10,
                    max: 100,
                    default: 35,
                    value: 35,
                },
            ],
        },
        ScreensaverConfig {
            name: "Omaxi Vectors",
            registry_name: "OmaxiVectors",
            binary_name: "omaxi-vectors.scr",
            fields: vec![
                ConfigField::Integer {
                    name: "ParticleSpeed",
                    label: "Particle Speed",
                    min: 10,
                    max: 100,
                    default: 30,
                    value: 30,
                },
                ConfigField::Integer {
                    name: "NodeCount",
                    label: "Node Count",
                    min: 20,
                    max: 80,
                    default: 50,
                    value: 50,
                },
            ],
        },
    ]
}

pub fn load_global_config() -> GlobalConfig {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let desktop = hkcu.open_subkey("Control Panel\\Desktop").ok();
    
    let active_scr = desktop.as_ref()
        .and_then(|key| key.get_value::<String, _>("SCRNSAVE.EXE").ok())
        .unwrap_or_default();
        
    let active_str = desktop.as_ref()
        .and_then(|key| key.get_value::<String, _>("ScreenSaveActive").ok())
        .unwrap_or_else(|| "0".to_string());
    let active = active_str == "1";
    
    let timeout_str = desktop.as_ref()
        .and_then(|key| key.get_value::<String, _>("ScreenSaveTimeOut").ok())
        .unwrap_or_else(|| "600".to_string());
    let timeout = timeout_str.parse::<u32>().unwrap_or(600);
    
    GlobalConfig {
        active_scr,
        active,
        timeout,
    }
}

pub fn save_global_config(config: &GlobalConfig) -> std::io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (desktop, _) = hkcu.create_subkey("Control Panel\\Desktop")?;
    
    // Windows expects SCRNSAVE.EXE to be written, and if ScreenSaveActive is 0, we can still set it.
    desktop.set_value("SCRNSAVE.EXE", &config.active_scr)?;
    let active_str = if config.active { "1" } else { "0" };
    desktop.set_value("ScreenSaveActive", &active_str)?;
    desktop.set_value("ScreenSaveTimeOut", &config.timeout.to_string())?;
    
    Ok(())
}

pub fn load_screensaver_fields(registry_name: &str, fields: &mut [ConfigField]) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let subkey_path = format!("Software\\{}", registry_name);
    if let Ok(key) = hkcu.open_subkey(&subkey_path) {
        for field in fields {
            match field {
                ConfigField::Integer { name, default, value, .. } => {
                    let reg_val: u32 = key.get_value(name).unwrap_or(*default as u32);
                    *value = reg_val as i32;
                }
                ConfigField::Boolean { name, default, value, .. } => {
                    let reg_val: u32 = key.get_value(name).unwrap_or(if *default { 1 } else { 0 });
                    *value = reg_val != 0;
                }
            }
        }
    } else {
        // Fallback to default
        for field in fields {
            match field {
                ConfigField::Integer { default, value, .. } => {
                    *value = *default;
                }
                ConfigField::Boolean { default, value, .. } => {
                    *value = *default;
                }
            }
        }
    }
}

pub fn save_screensaver_fields(registry_name: &str, fields: &[ConfigField]) -> std::io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let subkey_path = format!("Software\\{}", registry_name);
    let (key, _) = hkcu.create_subkey(&subkey_path)?;
    for field in fields {
        match field {
            ConfigField::Integer { name, value, .. } => {
                key.set_value(name, &(*value as u32))?;
            }
            ConfigField::Boolean { name, value, .. } => {
                key.set_value(name, &if *value { 1u32 } else { 0u32 })?;
            }
        }
    }
    Ok(())
}
