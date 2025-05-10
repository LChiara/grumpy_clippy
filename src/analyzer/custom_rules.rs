use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RuleConfig {
    pub name: String,
    pub enabled: bool,
    pub threshold: Option<u32>,
    pub option: Option<String>,
}

fn no_todo_comments(source: &str) -> bool {
    source.to_lowercase().contains("todo")
}

fn contains_forbidden_word(source: &str, word: &str) -> bool {
    source.contains(word)
}

pub fn load_custom_rules_from_toml(path: &str) -> Result<Option<Vec<RuleConfig>>, String> {
    if !std::path::Path::new(path).exists() {
        return Ok(None);
    }

    let toml_str = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => return Err(format!("Failed to read ruleset file '{}': {}", path, e)),
    };
    let parsed: toml::Value = toml::from_str(&toml_str).expect("Invalid TOML");

    parsed["rules"]
        .as_array()
        .expect("Expected 'rules' array")
        .iter()
        .map(|rule| toml::from_str(&rule.to_string()).expect("Invalid rule format"))
        .collect()
}

pub fn apply_rules(rules: Vec<RuleConfig>, source: &str) -> Result<(bool, Vec<String>), String> {
    let mut messages = vec![];
    let mut successful = true;

    for rule in rules {
        if !rule.enabled {
            continue;
        }

        match rule.name.as_str() {
            "no_todo_comments" => {
                if no_todo_comments(source) {
                    successful = false;
                    messages.push(generate_message(
                        rule.name,
                        Some(String::from("TODO comments found!")),
                    ));
                }
            }
            "forbid_word" => {
                if let Some(forbidden_word) = rule.option {
                    if contains_forbidden_word(source, &forbidden_word) {
                        successful = false;
                        messages.push(generate_message(
                            rule.name,
                            format!("Use of forbidden word: {}", forbidden_word).into(),
                        ));
                    }
                }
            }
            _ => {
                Err(format!("Unknown rule: {}", rule.name))?;
            }
        }
    }

    Ok((successful, messages))
}

fn generate_message(rule: String, message: Option<String>) -> String {
    return format!("Rule violation: {}\nmessage {:?}", rule, Some(message));
}
