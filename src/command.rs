#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Attack { target: String },
    Perform { skill: String, target: Option<String> },
    Look,
    Score,
    Say { message: String },
    Go { direction: String },
    Get { item: String },
    Drop { item: String },
    Inventory,
    Alias { name: Option<String>, command: Option<String> },
    Unalias { name: String },
    Invalid(String),
    Unknown(String),
}

pub fn parse(input: &str) -> Command {
    let input = input.trim();
    if input.is_empty() {
        return Command::Invalid("Empty command".to_string());
    }

    let mut parts = input.split_whitespace();
    let command = parts.next().unwrap_or("");

    match command.to_lowercase().as_str() {
        "attack" => {
            if let Some(target) = parts.next() {
                Command::Attack {
                    target: target.to_string(),
                }
            } else {
                Command::Invalid("Attack who?".to_string())
            }
        }
        "perform" => {
            if let Some(skill) = parts.next() {
                Command::Perform {
                    skill: skill.to_string(),
                    target: parts.next().map(|s| s.to_string()),
                }
            } else {
                Command::Invalid("Perform what?".to_string())
            }
        }
        "look" => Command::Look,
        "score" | "status" | "attr" => Command::Score,
        "say" => {
            let message = parts.collect::<Vec<&str>>().join(" ");
            if message.is_empty() {
                Command::Invalid("Say what?".to_string())
            } else {
                Command::Say { message }
            }
        }
        "go" => {
             if let Some(direction_part) = parts.next() {
                let direction_lowercase = direction_part.to_lowercase(); // Fix: Store the temporary value
                let direction = match direction_lowercase.as_str() { // Now borrow from the longer-lived value
                    "n" => "north",
                    "s" => "south",
                    "e" => "east",
                    "w" => "west",
                    "u" => "up",
                    "d" => "down",
                    full_dir => full_dir,
                };
                Command::Go { direction: direction.to_string() }
            } else {
                Command::Invalid("Go where?".to_string())
            }
        }
        "get" | "take" => {
            if let Some(item) = parts.next() {
                Command::Get { item: item.to_string() }
            } else {
                Command::Invalid("Get what?".to_string())
            }
        }
        "drop" => {
            if let Some(item) = parts.next() {
                Command::Drop { item: item.to_string() }
            } else {
                Command::Invalid("Drop what?".to_string())
            }
        }
        "i" | "inventory" => Command::Inventory,
        "alias" => {
            if let Some(name) = parts.next() {
                let command_to_alias = parts.collect::<Vec<&str>>().join(" ");
                if command_to_alias.is_empty() {
                    Command::Invalid(format!("What should the alias '{}' do?", name))
                } else {
                    Command::Alias {
                        name: Some(name.to_string()),
                        command: Some(command_to_alias),
                    }
                }
            } else {
                Command::Alias { name: None, command: None }
            }
        }
        "unalias" => {
            if let Some(name) = parts.next() {
                Command::Unalias { name: name.to_string() }
            } else {
                Command::Invalid("Unalias what?".to_string())
            }
        }
        _ => Command::Unknown(command.to_string()),
    }
}
