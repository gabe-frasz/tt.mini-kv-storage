mod extension_manager;
mod io;
mod parser;
mod storage;

use crate::{extension_manager::ExtensionManager, parser::Command};

fn main() {
    let ext_manager = ExtensionManager::load("extensions");

    loop {
        let raw_input = match io::read() {
            Ok(Some(i)) => i,
            Ok(None) => break, // EOF
            Err(e) => {
                io::print_error(&e.to_string());
                continue;
            }
        };

        let mut cmd = match parser::parse(&raw_input) {
            Ok(c) => c,
            Err(e) => {
                io::print_error(&e);
                continue;
            }
        };

        let (k, v) = cmd.params_as_tuple();
        match ext_manager.trigger_pre_hook(cmd.as_str(), k, v) {
            Ok(new_value) => {
                if let (Command::Add { value, .. }, Some(nv)) = (&mut cmd, new_value) {
                    *value = nv;
                }
            }
            Err(e) => {
                io::print_error(&e);
                continue;
            }
        };

        let storage_result = match &cmd {
            Command::Add { key, value } => storage::add(key, value),
            Command::Get { key } => storage::get(key),
            Command::Exit => break,
        };

        let result = match &storage_result {
            Ok(r) => r.as_deref(),
            Err(e) => {
                io::print_error(e);
                continue;
            }
        };

        let (k, v) = cmd.params_as_tuple();
        match ext_manager.trigger_post_hook(cmd.as_str(), k, v, result) {
            Ok(Some(formatted_result)) => io::print_value(&formatted_result),
            Ok(None) => {},
            Err(e) => {
                io::print_error(&e);
                continue;
            }
        };

        io::print_success();
    }
}
