use mlua::{Function, Lua, RegistryKey, Table};
use std::fs;
use std::path::Path;

pub struct ExtensionManager {
    lua: Lua,
    extensions: Vec<(String, RegistryKey)>,
}

impl ExtensionManager {
    pub fn load(dir_name: &str) -> Self {
        let lua = Lua::new();
        let mut extensions = Vec::new();

        let dir = Path::new(dir_name);
        if !dir.is_dir() {
            return Self { lua, extensions };
        }

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error reading extensions directory: {e}");
                return Self { lua, extensions };
            }
        };

        for entry in entries {
            let path = entry.unwrap().path();

            if path.extension().and_then(|s| s.to_str()) == Some("lua") {
                let filename = path.file_name().unwrap().to_string_lossy().to_string();

                match fs::read_to_string(&path) {
                    Ok(source) => {
                        let result: Result<Table, _> = lua.load(&source).set_name(&filename).eval();

                        match result {
                            Ok(table) => match lua.create_registry_value(table) {
                                Ok(key) => extensions.push((filename, key)),
                                Err(e) => {
                                    eprintln!("Error loading extension {filename}: {e}");
                                }
                            },
                            Err(e) => {
                                eprintln!("Error loading extension {filename}: {e}");
                            }
                        };
                    }
                    Err(e) => {
                        eprintln!("Error reading {filename}: {e}");
                    }
                }
            }
        }

        Self { lua, extensions }
    }

    pub fn trigger_pre_hook(&self, command: &str, args: &[String]) {
        for (name, key) in &self.extensions {
            if let Ok(table) = self.lua.registry_value::<Table>(key) {
                if let Ok(hook) = table.get::<Function>("pre_hook") {
                    if let Err(e) = hook.call::<()>((command, args.to_vec())) {
                        eprintln!("Error calling pre_hook for {name}: {e}");
                    }
                }
            }
        }
    }

    pub fn trigger_post_hook(&self, command: &str, args: &[String]) {
        for (name, key) in &self.extensions {
            if let Ok(table) = self.lua.registry_value::<Table>(key) {
                if let Ok(hook) = table.get::<Function>("post_hook") {
                    if let Err(e) = hook.call::<()>((command, args.to_vec())) {
                        eprintln!("Error calling post_hook for {name}: {e}");
                    }
                }
            }
        }
    }   
}
