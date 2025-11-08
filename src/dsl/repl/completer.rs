//! Tab Completion for REPL Commands
//!
//! Provides intelligent command and argument completion for the debugging REPL.

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::borrow::Cow;

/// REPL command completer and helper
#[derive(Default)]
pub struct ReplHelper {
    commands: Vec<String>,
    aliases: Vec<String>,
}

impl ReplHelper {
    /// Create a new REPL helper
    pub fn new() -> Self {
        let mut commands = Vec::new();
        let mut aliases = Vec::new();

        // Collect all command names and their aliases
        Self::add_execution_commands(&mut commands, &mut aliases);
        Self::add_breakpoint_commands(&mut commands, &mut aliases);
        Self::add_inspection_commands(&mut commands, &mut aliases);
        Self::add_navigation_commands(&mut commands, &mut aliases);
        Self::add_modification_commands(&mut commands, &mut aliases);
        Self::add_utility_commands(&mut commands, &mut aliases);
        Self::add_ai_commands(&mut commands, &mut aliases);

        // Sort for better completion
        commands.sort();
        aliases.sort();

        Self { commands, aliases }
    }

    fn add_execution_commands(commands: &mut Vec<String>, aliases: &mut Vec<String>) {
        commands.extend_from_slice(&[
            "continue".to_string(),
            "step".to_string(),
            "stepi".to_string(),
            "next".to_string(),
            "finish".to_string(),
            "stepit".to_string(),
            "stepback".to_string(),
            "stepforward".to_string(),
            "restart".to_string(),
            "pause".to_string(),
            "resume".to_string(),
        ]);

        aliases.extend_from_slice(&[
            "c".to_string(),   // continue
            "s".to_string(),   // step
            "si".to_string(),  // stepi
            "n".to_string(),   // next
            "fin".to_string(), // finish
            "sb".to_string(),  // stepback
            "sf".to_string(),  // stepforward
        ]);
    }

    fn add_breakpoint_commands(commands: &mut Vec<String>, aliases: &mut Vec<String>) {
        commands.extend_from_slice(&[
            "break".to_string(),
            "delete".to_string(),
            "breaks".to_string(),
            "enable".to_string(),
            "disable".to_string(),
            "clearbreaks".to_string(),
        ]);

        aliases.extend_from_slice(&[
            "b".to_string(),      // break
            "d".to_string(),      // delete
            "bclear".to_string(), // clearbreaks
        ]);
    }

    fn add_inspection_commands(commands: &mut Vec<String>, aliases: &mut Vec<String>) {
        commands.extend_from_slice(&[
            "inspect".to_string(),
            "print".to_string(),
            "vars".to_string(),
            "stack".to_string(),
            "timeline".to_string(),
            "snapshots".to_string(),
            "status".to_string(),
        ]);

        aliases.extend_from_slice(&[
            "i".to_string(),  // inspect
            "p".to_string(),  // print
            "bt".to_string(), // stack (backtrace)
            "backtrace".to_string(),
            "tl".to_string(),    // timeline
            "snaps".to_string(), // snapshots
            "info".to_string(),  // status
        ]);
    }

    fn add_navigation_commands(commands: &mut Vec<String>, _aliases: &mut Vec<String>) {
        commands.extend_from_slice(&[
            "goto".to_string(),
            "back".to_string(),
            "forward".to_string(),
        ]);
    }

    fn add_modification_commands(commands: &mut Vec<String>, _aliases: &mut Vec<String>) {
        commands.push("set".to_string());
    }

    fn add_utility_commands(commands: &mut Vec<String>, aliases: &mut Vec<String>) {
        commands.extend_from_slice(&[
            "help".to_string(),
            "quit".to_string(),
            "pwd".to_string(),
            "ls".to_string(),
            "echo".to_string(),
            "clear".to_string(),
            "history".to_string(),
        ]);

        aliases.extend_from_slice(&[
            "h".to_string(),    // help
            "?".to_string(),    // help
            "q".to_string(),    // quit
            "exit".to_string(), // quit
            "cls".to_string(),  // clear
        ]);
    }

    fn add_ai_commands(commands: &mut Vec<String>, aliases: &mut Vec<String>) {
        commands.extend_from_slice(&[
            "ai-generate".to_string(),
            "ai-fix".to_string(),
            "ai-analyze".to_string(),
            "ai-explain".to_string(),
            "ai-provider".to_string(),
            "ai-config".to_string(),
        ]);

        aliases.extend_from_slice(&[
            "aigen".to_string(),      // ai-generate
            "aifix".to_string(),      // ai-fix
            "aianalyze".to_string(),  // ai-analyze
            "aiexplain".to_string(),  // ai-explain
            "aiprovider".to_string(), // ai-provider
            "aiconfig".to_string(),   // ai-config
        ]);
    }

    /// Get all possible completions for a partial command
    fn get_completions(&self, partial: &str) -> Vec<String> {
        let mut completions = Vec::new();

        // Check commands
        for cmd in &self.commands {
            if cmd.starts_with(partial) {
                completions.push(cmd.clone());
            }
        }

        // Check aliases
        for alias in &self.aliases {
            if alias.starts_with(partial) {
                completions.push(alias.clone());
            }
        }

        completions.sort();
        completions.dedup();
        completions
    }
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line_up_to_cursor = &line[..pos];

        // Split by whitespace to get command and args
        let parts: Vec<&str> = line_up_to_cursor.split_whitespace().collect();

        if parts.is_empty() {
            // Complete from beginning
            let completions = self.get_completions("");
            return Ok((
                0,
                completions
                    .into_iter()
                    .map(|c| Pair {
                        display: c.clone(),
                        replacement: c,
                    })
                    .collect(),
            ));
        }

        // If we're still on the first word (command), complete the command
        if parts.len() == 1 && !line_up_to_cursor.ends_with(' ') {
            let partial = parts[0];
            let completions = self.get_completions(partial);
            let start_pos = line_up_to_cursor.len() - partial.len();

            return Ok((
                start_pos,
                completions
                    .into_iter()
                    .map(|c| Pair {
                        display: c.clone(),
                        replacement: c,
                    })
                    .collect(),
            ));
        }

        // For arguments, we could add context-specific completion here
        // For now, return empty (no argument completion)
        Ok((pos, Vec::new()))
    }
}

impl Helper for ReplHelper {}

impl Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        // Only hint if we're on the first word (command)
        if parts.len() == 1 {
            let partial = parts[0];
            let completions = self.get_completions(partial);

            // Return the first completion as a hint
            if !completions.is_empty() && completions[0] != partial {
                let hint = &completions[0][partial.len()..];
                return Some(hint.to_string());
            }
        }

        None
    }
}

impl Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Basic syntax highlighting - could be enhanced with colors
        Cow::Borrowed(line)
    }
}

impl Validator for ReplHelper {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completer_basic() {
        let helper = ReplHelper::new();

        // Test command completion
        let completions = helper.get_completions("ste");
        assert!(completions.contains(&"step".to_string()));
        assert!(completions.contains(&"stepi".to_string()));
        assert!(completions.contains(&"stepback".to_string()));
        assert!(completions.contains(&"stepforward".to_string()));
        assert!(completions.contains(&"stepit".to_string()));
    }

    #[test]
    fn test_completer_aliases() {
        let helper = ReplHelper::new();

        // Test alias completion
        let completions = helper.get_completions("c");
        assert!(completions.contains(&"c".to_string()));
        assert!(completions.contains(&"continue".to_string()));
        assert!(completions.contains(&"clear".to_string()));
        assert!(completions.contains(&"cls".to_string()));
    }

    #[test]
    fn test_completer_ai_commands() {
        let helper = ReplHelper::new();

        let completions = helper.get_completions("ai-");
        assert!(completions.contains(&"ai-generate".to_string()));
        assert!(completions.contains(&"ai-fix".to_string()));
        assert!(completions.contains(&"ai-analyze".to_string()));
        assert!(completions.contains(&"ai-explain".to_string()));
        assert!(completions.contains(&"ai-provider".to_string()));
        assert!(completions.contains(&"ai-config".to_string()));
    }

    #[test]
    fn test_completer_exact_match() {
        let helper = ReplHelper::new();

        let completions = helper.get_completions("help");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0], "help");
    }

    #[test]
    fn test_completer_no_match() {
        let helper = ReplHelper::new();

        let completions = helper.get_completions("xyz");
        assert!(completions.is_empty());
    }
}
