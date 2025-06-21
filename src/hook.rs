use crate::config::Hook;
use regex::Regex;
use std::collections::HashMap;
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
    previous_matches: HashMap<String, bool>,
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
            previous_matches: HashMap::new(),
        })
    }
    
    pub fn evaluate(&mut self, _previous: &str, current: &str) -> Vec<String> {
        let mut triggered = Vec::new();
        
        for hook in &mut self.hooks {
            let prev_match = self.previous_matches.get(&hook.name).copied().unwrap_or(false);
            let curr_match = hook.regex.is_match(current);
            
            self.previous_matches.insert(hook.name.clone(), curr_match);
            
            if curr_match && !prev_match {
                if let Some(cooldown) = hook.cooldown {
                    if let Some(last_fired) = hook.last_fired {
                        if Instant::now().duration_since(last_fired) < cooldown {
                            eprintln!("[cc-hook] Skipping hook '{}' due to cooldown", hook.name);
                            continue;
                        }
                    }
                }
                
                hook.last_fired = Some(Instant::now());
                triggered.push(hook.command.clone());
                eprintln!("[cc-hook] Triggered hook: {}", hook.name);
            }
        }
        
        triggered
    }
    
    pub fn execute_commands(commands: Vec<String>) {
        for command in commands {
            task::spawn(async move {
                match Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .spawn()
                {
                    Ok(mut child) => {
                        match child.wait() {
                            Ok(status) => {
                                eprintln!("[cc-hook] Command '{}' exited with: {}", command, status);
                            }
                            Err(e) => {
                                eprintln!("[cc-hook] Failed to wait for command '{}': {}", command, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[cc-hook] Failed to execute command '{}': {}", command, e);
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
        let hooks = vec![
            Hook {
                name: "test1".to_string(),
                regex: "Hello.*World".to_string(),
                command: "echo matched".to_string(),
                cooldown_ms: None,
            },
        ];
        
        let mut engine = HookEngine::new(hooks).unwrap();
        
        let triggered = engine.evaluate("", "Hello, World!");
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], "echo matched");
        
        let triggered = engine.evaluate("Hello, World!", "Hello, World!");
        assert_eq!(triggered.len(), 0);
    }
}