mod extension_manager;
mod io;
mod parser;
mod storage;

use crate::extension_manager::ExtensionManager;

fn main() {
    let ext_manager = ExtensionManager::load("extensions");

    loop {
        let raw_input = match io::read() {
            Ok(Some(input)) => input,
            Ok(None) => break, // EOF
            Err(e) => {
                io::print_error(&e.to_string());
                continue;
            }
        };

        let (cmd, mut args) = match parser::parse(&raw_input) {
            Ok(c) => c,
            Err(e) => {
                io::print_error(&e);
                continue;
            }
        };

        // TODO: return the new value
        ext_manager.trigger_pre_hook(cmd, &args);

        // TODO: find a new way to match on the command to avoid duplication
        let storage_result = match cmd {
            "ADD" => {
                let key = args[0].clone();
                let value = args[1].clone();
                storage::add(&key, &value)
            }
            "GET" => {
                let key = args[0].clone();
                storage::get(&key)
            }
            "EXIT" => break,
            _ => {
                io::print_error("Unknown command");
                continue;
            }
        };

        match storage_result {
            Ok(Some(r)) => args.push(r.clone()),
            Ok(None) => {}
            Err(e) => io::print_error(&e),
        }

        // TODO: return the formatted output
        ext_manager.trigger_post_hook(cmd, &args);
        io::print_success();
    }
}
