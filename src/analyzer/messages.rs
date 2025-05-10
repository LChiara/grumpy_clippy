pub mod clippy {
    use crate::config::GrumpinessLevel;

    pub fn success(level: &GrumpinessLevel) -> &'static str {
        match level {
            GrumpinessLevel::Mild => "✅ cargo clippy successful",
            GrumpinessLevel::Sarcastic => "✅🙈 Oh, you did not break anything. Strange!",
            GrumpinessLevel::Rude => {
                "✅🙄 Oh, you managed not to break anything? Well, there is a first time for everything."
            }
        }
    }

    pub fn failure(level: &GrumpinessLevel) -> &'static str {
        match level {
            GrumpinessLevel::Mild => "❌ Clippy failed (see terminal for details)",
            GrumpinessLevel::Sarcastic => "❌🙄 Oh, you did break something (as usual):",
            GrumpinessLevel::Rude => "❌💣 Of course you broke something—how utterly predictable.",
        }
    }
}

pub mod complexity {
    pub use crate::config::GrumpinessLevel;

    pub fn warning(level: &GrumpinessLevel, name: &str, complexity: usize, max: u8) -> String {
        match level {
            GrumpinessLevel::Mild => format!(
                "Function '{}': Cyclomatic complexity too high ({} > {}). Consider simplifying it.",
                name, complexity, max
            ),
            GrumpinessLevel::Sarcastic => format!(
                "Function '{}': Wow, cyclomatic complexity ({} > {})! Are you trying to write a novel?",
                name, complexity, max
            ),
            GrumpinessLevel::Rude => format!(
                "Function '{}': Cyclomatic complexity ({} > {})? What is this monstrosity?",
                name, complexity, max
            ),
        }
    }
}

pub mod function_size {
    use crate::config::GrumpinessLevel;

    pub fn warning(level: &GrumpinessLevel, name: &str, size: usize, max: u8) -> String {
        match level {
            GrumpinessLevel::Mild => format!(
                "Function '{}': Too many lines ({} > {}). Consider refactoring.",
                name, size, max
            ),
            GrumpinessLevel::Sarcastic => format!(
                "Function '{}': Wow, {} lines ({} > {})! Are you writing a novel?",
                name, size, size, max
            ),
            GrumpinessLevel::Rude => format!(
                "Function '{}': {} lines ({} > {})? This is absurd!",
                name, size, size, max
            ),
        }
    }
}

pub mod git_is_stale {
    use crate::config::GrumpinessLevel;

    pub fn info(level: &GrumpinessLevel) -> String {
        match level {
            GrumpinessLevel::Mild => {
                format!("Git: Hey there! Just a heads-up: file hasn’t been updated in a while.",)
            }
            GrumpinessLevel::Sarcastic => {
                format!("Git: file looks stale. Consider revisiting it.")
            }
            GrumpinessLevel::Rude => {
                format!("Git: file is gathering dust. Are you asleep at the keyboard?",)
            }
        }
    }
}

pub mod git_most_frequent_author {
    use crate::config::GrumpinessLevel;

    pub fn info(level: &GrumpinessLevel, author: &str) -> String {
        match level {
            GrumpinessLevel::Mild => {
                format!("Git: file mostly edited by our star `{}`!", author)
            }
            GrumpinessLevel::Sarcastic => format!(
                "Git: file mostly authored by `{}`. Check if they're still around.",
                author
            ),
            GrumpinessLevel::Rude => {
                format!("Git: Looks like here is {}'s personal playground.", author)
            }
        }
    }
}
