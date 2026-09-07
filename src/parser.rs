pub fn parse(input: &str) -> Result<(&str, Vec<String>), String> {
    let mut tokens = input.split_whitespace();
    let command_name = tokens.next().ok_or("No command provided".to_string())?;

    let command = match command_name.to_ascii_uppercase().as_str() {
        "ADD" => {
            let key = tokens.next().ok_or("No key provided".to_string())?;
            let value = tokens.clone().collect::<Vec<&str>>().join(" ");
            if value.is_empty() {
                return Err("No value provided".to_string());
            }
            return Ok(("ADD", vec![key.to_string(), value]));
        }
        "GET" => {
            let key = tokens.next().ok_or("No key provided".to_string())?;
            ("GET", vec![key.to_string()])
        }
        "EXIT" => ("EXIT", vec![]),
        other => return Err(format!("Unknown command: {other}")),
    };

    if tokens.next().is_some() {
        return Err("Too many arguments".to_string());
    }

    Ok(command)
}
