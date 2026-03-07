use crate::world::skill::Skill;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn load_skills(dir_path: &Path) -> Result<HashMap<String, Skill>, std::io::Error> {
    let mut skills = HashMap::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file_content = fs::read_to_string(&path)?;
            let skill: Skill = serde_json::from_str(&file_content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            skills.insert(skill.name.clone(), skill);
        }
    }

    Ok(skills)
}
