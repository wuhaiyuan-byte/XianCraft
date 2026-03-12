#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Attack { target: String },
    Perform { skill: String, target: Option<String> },
    Look,
    Score,
    Talk { target: String },
    Quest,
    Accept { quest_id: String },
    Say { message: String },
    Go { direction: String },
    Get { item: String },
    Drop { item: String },
    Inventory,
    Alias { name: Option<String>, command: Option<String> },
    Unalias { name: String },
    Rest,
    Work,
    Help, // New Help command
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
        "talk" | "chat" => {
            if let Some(target) = parts.next() {
                Command::Talk {
                    target: target.to_string(),
                }
            } else {
                Command::Invalid("Talk to who?".to_string())
            }
        }
        "quest" | "qs" => Command::Quest,
        "accept" => {
            if let Some(id) = parts.next() {
                Command::Accept {
                    quest_id: id.to_string(),
                }
            } else {
                Command::Invalid("Accept which quest?".to_string())
            }
        }
        "say" => {
            let message = parts.collect::<Vec<&str>>().join(" ");
            if message.is_empty() {
                Command::Invalid("Say what?".to_string())
            } else {
                Command::Say { message }
            }
        }
        "go" => {
            if let Some(dir) = parts.next() {
                let dir_lower = dir.to_lowercase();
                let direction = match dir_lower.as_str() {
                    "n" => "north",
                    "s" => "south",
                    "e" => "east",
                    "w" => "west",
                    "u" => "up",
                    "d" => "down",
                    full => full,
                };
                Command::Go { direction: direction.to_string() }
            } else {
                Command::Invalid("Go where?".to_string())
            }
        }
        "n" | "s" | "e" | "w" | "u" | "d" => {
            let direction = match command.to_lowercase().as_str() {
                "n" => "north",
                "s" => "south",
                "e" => "east",
                "w" => "west",
                "u" => "up",
                "d" => "down",
                _ => command,
            };
            Command::Go { direction: direction.to_string() }
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
        "rest" => Command::Rest,
        "work" | "job" => Command::Work,
        "help" => Command::Help, // Handle 'help'
        _ => Command::Unknown(command.to_string()),
    }
}
