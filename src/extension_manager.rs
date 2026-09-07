use std::fs;
use std::path::Path;

use mlua::{Function, Lua, RegistryKey, Table};

struct Extension {
    key: RegistryKey,
    prefix: String,
    name: String,
}

pub struct ExtensionManager {
    lua: Lua,
    extensions: Vec<Extension>,
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
            Ok(en) => en,
            Err(e) => {
                eprintln!("Error reading extensions directory: {e}");
                return Self { lua, extensions };
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Error reading directory entry: {e}");
                    continue;
                }
            };
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("lua") {
                continue;
            }

            let name = match path.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };
            let stem = match path.file_stem() {
                Some(s) => s.to_string_lossy().to_string(),
                None => continue,
            };
            let default_prefix = format!("{stem}_");

            match fs::read_to_string(&path) {
                Ok(source) => {
                    if source.trim().is_empty() {
                        continue;
                    }

                    let result: Result<Table, _> = lua.load(&source).set_name(&name).eval();

                    match result {
                        Ok(table) => {
                            let prefix = table.get::<String>("prefix").unwrap_or(default_prefix);

                            match lua.create_registry_value(table) {
                                Ok(key) => extensions.push(Extension { key, prefix, name }),
                                Err(e) => eprintln!("Failed to load extension {name}: {e}"),
                            }
                        }
                        Err(e) => eprintln!("Failed to load extension {name}: {e}"),
                    };
                }
                Err(e) => eprintln!("Failed to read {name}: {e}"),
            }
        }

        Self { lua, extensions }
    }

    pub fn trigger_pre_hook(
        &self,
        command: &str,
        key: Option<&str>,
        value: Option<&str>,
    ) -> Result<Option<String>, String> {
        let key = match key {
            Some(k) => k,
            None => return Ok(value.map(str::to_string)),
        };

        let ctx = self
            .create_context_table(command, Some(key), value, None)
            .map_err(|e| format!("Failed to create context table: {e}"))?;

        for ext in &self.extensions {
            if !key.starts_with(&ext.prefix) {
                continue;
            }

            if let Ok(table) = self.lua.registry_value::<Table>(&ext.key) {
                if let Ok(hook) = table.get::<Function>("pre_hook") {
                    let ret: mlua::Value = hook
                        .call(ctx.clone())
                        .map_err(|e| format!("[{}] {}", ext.name, e))?;

                    if let mlua::Value::String(s) = ret {
                        if let Ok(str_val) = s.to_str() {
                            let _ = ctx.set("value", str_val);
                        }
                    }
                }
            }
        }

        let new_value = ctx
            .get::<Option<String>>("value")
            .unwrap_or_else(|_| value.map(str::to_string));
        Ok(new_value)
    }

    pub fn trigger_post_hook(
        &self,
        command: &str,
        key: Option<&str>,
        value: Option<&str>,
        result: Option<&str>,
    ) -> Result<Option<String>, String> {
        let key = match key {
            Some(k) => k,
            None => return Ok(result.map(str::to_string)),
        };

        let ctx = self
            .create_context_table(command, Some(key), value, result)
            .map_err(|e| format!("Failed to create context table: {e}"))?;

        for ext in &self.extensions {
            if !key.starts_with(&ext.prefix) {
                continue;
            }

            if let Ok(table) = self.lua.registry_value::<Table>(&ext.key) {
                if let Ok(hook) = table.get::<Function>("post_hook") {
                    let ret: mlua::Value = hook
                        .call(ctx.clone())
                        .map_err(|e| format!("[{}] {}", ext.name, e))?;

                    if let mlua::Value::String(s) = ret {
                        if let Ok(str_val) = s.to_str() {
                            let _ = ctx.set("result", str_val);
                        }
                    }
                }
            }
        }

        let new_result = ctx
            .get::<Option<String>>("result")
            .unwrap_or_else(|_| result.map(str::to_string));
        Ok(new_result)
    }

    fn create_context_table(
        &self,
        command: &str,
        key: Option<&str>,
        value: Option<&str>,
        result: Option<&str>,
    ) -> mlua::Result<Table> {
        let table = self.lua.create_table()?;
        table.set("command", command)?;
        if let Some(key) = key {
            table.set("key", key)?;
        }
        if let Some(value) = value {
            table.set("value", value)?;
        }
        if let Some(result) = result {
            table.set("result", result)?;
        }
        Ok(table)
    }
}
