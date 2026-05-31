use std::path::PathBuf;

pub fn discover_screensavers() -> Vec<(String, PathBuf)> {
    let mut list = Vec::new();
    let appdata = match std::env::var("APPDATA") {
        Ok(val) => PathBuf::from(val).join(".omaxi").join("apps").join("ssm"),
        Err(_) => return list,
    };
    
    // Scan for any screensaver file in AppData
    if let Ok(entries) = std::fs::read_dir(appdata) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext.to_ascii_lowercase() == "scr" {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // Prettify name: strip "omaxi-" and ".scr"
                        let pretty_name = if name.to_lowercase().starts_with("omaxi-") {
                            name.strip_prefix("omaxi-")
                                .and_then(|s| s.strip_suffix(".scr"))
                                .map(|s| {
                                    let mut chars = s.chars();
                                    match chars.next() {
                                        None => String::new(),
                                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                                    }
                                })
                                .unwrap_or_else(|| name.to_string())
                        } else {
                            name.strip_suffix(".scr").unwrap_or(name).to_string()
                        };
                        
                        list.push((pretty_name, path));
                    }
                }
            }
        }
    }
    list.sort_by(|a, b| a.0.cmp(&b.0));
    list
}
