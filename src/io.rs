use std::io::{self, stdin, IsTerminal, Write};

fn is_interactive() -> bool {
    return stdin().is_terminal();
}

pub fn read() -> io::Result<Option<String>> {
    if is_interactive() {
        print!("> ");
        io::stdout().flush()?
    }

    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer)? {
        0 => Ok(None),
        _ => Ok(Some(buffer.trim().to_string())),
    }
}

pub fn print_success() {
    if is_interactive() {
        println!("OK");
    }
}

pub fn print_error(message: &str) {
    eprintln!("[ERROR] {}", message);
}
