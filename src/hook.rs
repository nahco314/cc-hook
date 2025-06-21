use crate::config::Hook;
use regex::Regex;
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::task;

pub struct CompiledHook {
    pub name: String,
    pub regex: Regex,
    pub command: String,
    pub cooldown: Option<Duration>,
    last_fired: Option<Instant>,
}

pub struct HookEngine {
    hooks: Vec<CompiledHook>,
}

impl HookEngine {
    pub fn new(hooks: Vec<Hook>) -> Result<Self, regex::Error> {
        let mut compiled_hooks = Vec::new();

        for hook in hooks {
            let regex = Regex::new(&hook.regex)?;
            compiled_hooks.push(CompiledHook {
                name: hook.name,
                regex,
                command: hook.command,
                cooldown: hook.cooldown_ms.map(Duration::from_millis),
                last_fired: None,
            });
        }

        Ok(Self {
            hooks: compiled_hooks,
        })
    }

    pub fn evaluate(&mut self, previous: &str, current: &str) -> Vec<String> {
        let mut triggered = Vec::new();

        for hook in &mut self.hooks {
            let prev_match = hook.regex.is_match(previous);
            let curr_match = hook.regex.is_match(current);

            // Only trigger if pattern appears in current but not in previous
            if curr_match && !prev_match {
                if let Some(cooldown) = hook.cooldown {
                    if let Some(last_fired) = hook.last_fired {
                        if Instant::now().duration_since(last_fired) < cooldown {
                            // Silently skip due to cooldown
                            continue;
                        }
                    }
                }

                hook.last_fired = Some(Instant::now());
                triggered.push(hook.command.clone());
            }
        }

        triggered
    }

    pub fn execute_commands(commands: Vec<String>) {
        for command in commands {
            task::spawn(async move {
                match Command::new("sh").arg("-c").arg(&command).spawn() {
                    Ok(mut child) => {
                        // Silently wait for completion
                        let _ = child.wait();
                    }
                    Err(_) => {
                        // Silently ignore execution errors
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Hook;

    #[test]
    fn test_hook_triggering() {
        let hooks = vec![Hook {
            name: "test1".to_string(),
            regex: "Hello.*World".to_string(),
            command: "echo matched".to_string(),
            cooldown_ms: None,
        }];

        let mut engine = HookEngine::new(hooks).unwrap();

        let triggered = engine.evaluate("", "Hello, World!");
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], "echo matched");

        let triggered = engine.evaluate("Hello, World!", "Hello, World!");
        assert_eq!(triggered.len(), 0);
    }
}
