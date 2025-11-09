//! REPL Command Parser
//!
//! Parses user input into structured REPL commands.
use super::commands::{BreakTarget, InspectTarget, ReplCommand};
use crate::dsl::debugger::{BreakCondition, VariableScope};
use crate::dsl::task_graph::TaskStatus;
use crate::error::{Error, Result};

/// Parse a command line into a ReplCommand
pub fn parse_command(input: &str) -> Result<ReplCommand> {
    let input = input.trim();

    if input.is_empty() {
        return Err(Error::InvalidInput("Empty command".to_string()));
    }

    // Split into command and arguments
    let parts: Vec<&str> = input.split_whitespace().collect();
    let cmd = parts[0].to_lowercase();
    let args = &parts[1..];

    match cmd.as_str() {
        // Execution Control
        "continue" | "c" => Ok(ReplCommand::Continue),
        "step" | "s" => Ok(ReplCommand::Step),
        "stepi" | "si" => Ok(ReplCommand::StepInto),
        "next" | "n" => Ok(ReplCommand::StepOver),
        "finish" | "fin" => Ok(ReplCommand::StepOut),
        "stepit" => Ok(ReplCommand::StepIteration),
        "pause" => Ok(ReplCommand::Pause),
        "resume" => Ok(ReplCommand::Resume),
        "restart" => Ok(ReplCommand::Restart),

        "stepback" | "sb" => {
            let steps = if args.is_empty() {
                1
            } else {
                args[0]
                    .parse()
                    .map_err(|_| Error::InvalidInput("Invalid step count".to_string()))?
            };
            Ok(ReplCommand::StepBack { steps })
        }

        "stepforward" | "sf" => {
            let steps = if args.is_empty() {
                1
            } else {
                args[0]
                    .parse()
                    .map_err(|_| Error::InvalidInput("Invalid step count".to_string()))?
            };
            Ok(ReplCommand::StepForward { steps })
        }

        // Breakpoints
        "break" | "b" => parse_break_command(args),
        "delete" | "d" => {
            if args.is_empty() {
                return Err(Error::InvalidInput(
                    "Usage: delete <breakpoint_id>".to_string(),
                ));
            }
            Ok(ReplCommand::Delete {
                id: args[0].to_string(),
            })
        }
        "breaks" | "info" if args.first().map(|s| *s == "breaks").unwrap_or(true) => {
            Ok(ReplCommand::ListBreaks)
        }
        "enable" => {
            if args.is_empty() {
                return Err(Error::InvalidInput(
                    "Usage: enable <breakpoint_id>".to_string(),
                ));
            }
            Ok(ReplCommand::Enable {
                id: args[0].to_string(),
            })
        }
        "disable" => {
            if args.is_empty() {
                return Err(Error::InvalidInput(
                    "Usage: disable <breakpoint_id>".to_string(),
                ));
            }
            Ok(ReplCommand::Disable {
                id: args[0].to_string(),
            })
        }
        "clearbreaks" | "bclear" => Ok(ReplCommand::ClearBreaks),

        // Inspection
        "inspect" | "i" => parse_inspect_command(args),
        "print" | "p" => {
            if args.is_empty() {
                return Err(Error::InvalidInput("Usage: print <expression>".to_string()));
            }
            Ok(ReplCommand::Print {
                expression: args.join(" "),
            })
        }
        "vars" => {
            let scope = if args.is_empty() {
                None
            } else {
                Some(parse_variable_scope(args[0])?)
            };
            Ok(ReplCommand::Vars { scope })
        }
        "stack" | "bt" | "backtrace" => Ok(ReplCommand::Stack),
        "timeline" | "tl" => {
            let limit = if args.is_empty() {
                None
            } else {
                Some(
                    args[0]
                        .parse()
                        .map_err(|_| Error::InvalidInput("Invalid limit".to_string()))?,
                )
            };
            Ok(ReplCommand::Timeline { limit })
        }
        "snapshots" | "snaps" => Ok(ReplCommand::Snapshots),
        "status" => Ok(ReplCommand::Status),

        // Navigation
        "goto" => {
            if args.is_empty() {
                return Err(Error::InvalidInput("Usage: goto <snapshot_id>".to_string()));
            }
            let snapshot_id = args[0]
                .parse()
                .map_err(|_| Error::InvalidInput("Invalid snapshot ID".to_string()))?;
            Ok(ReplCommand::Goto { snapshot_id })
        }
        "back" => {
            let snapshots = if args.is_empty() {
                1
            } else {
                args[0]
                    .parse()
                    .map_err(|_| Error::InvalidInput("Invalid snapshot count".to_string()))?
            };
            Ok(ReplCommand::Back { snapshots })
        }
        "forward" => {
            let snapshots = if args.is_empty() {
                1
            } else {
                args[0]
                    .parse()
                    .map_err(|_| Error::InvalidInput("Invalid snapshot count".to_string()))?
            };
            Ok(ReplCommand::Forward { snapshots })
        }

        // Modification
        "set" => parse_set_command(args),

        // Utility
        "help" | "h" | "?" => {
            let command = args.first().map(|s| s.to_string());
            Ok(ReplCommand::Help { command })
        }
        "quit" | "q" | "exit" => Ok(ReplCommand::Quit),
        "pwd" => Ok(ReplCommand::Pwd),
        "ls" => {
            let path = args.first().map(|s| s.to_string());
            Ok(ReplCommand::Ls { path })
        }
        "echo" => Ok(ReplCommand::Echo {
            text: args.join(" "),
        }),
        "clear" | "cls" => Ok(ReplCommand::Clear),
        "history" => Ok(ReplCommand::History),
        "workflow" | "wf" | "tree" => Ok(ReplCommand::PrintWorkflow),
        "save" | "w" => {
            if args.is_empty() {
                return Err(Error::InvalidInput("Usage: save <file.yaml>".to_string()));
            }
            Ok(ReplCommand::SaveWorkflow {
                path: args[0].to_string(),
            })
        }
        "saveconfig" => {
            let path = args.first().map(|s| s.to_string());
            Ok(ReplCommand::SaveConfig { path })
        }

        // AI Commands
        "ai-generate" | "aigen" => {
            if args.is_empty() {
                return Err(Error::InvalidInput(
                    "Usage: ai-generate <description>".to_string(),
                ));
            }
            Ok(ReplCommand::AiGenerate {
                description: args.join(" "),
            })
        }
        "ai-fix" | "aifix" => {
            if args.is_empty() {
                return Err(Error::InvalidInput(
                    "Usage: ai-fix <error_message>".to_string(),
                ));
            }
            Ok(ReplCommand::AiFix {
                error: args.join(" "),
            })
        }
        "ai-analyze" | "aianalyze" => {
            let workflow = args.first().map(|s| s.to_string());
            Ok(ReplCommand::AiAnalyze { workflow })
        }
        "ai-explain" | "aiexplain" => {
            let workflow = args.first().map(|s| s.to_string());
            Ok(ReplCommand::AiExplain { workflow })
        }
        "ai-provider" | "aiprovider" => {
            if args.is_empty() {
                return Err(Error::InvalidInput(
                    "Usage: ai-provider <provider> [model]".to_string(),
                ));
            }
            let provider = args[0].to_string();
            let model = args.get(1).map(|s| s.to_string());
            Ok(ReplCommand::AiProvider { provider, model })
        }
        "ai-config" | "aiconfig" => Ok(ReplCommand::AiConfig),

        _ => Err(Error::InvalidInput(format!(
            "Unknown command: '{}'. Type 'help' for available commands.",
            cmd
        ))),
    }
}

/// Parse break command
fn parse_break_command(args: &[&str]) -> Result<ReplCommand> {
    if args.is_empty() {
        return Err(Error::InvalidInput(
            "Usage: break <task_id> | break condition <expr> | break watch <var>".to_string(),
        ));
    }

    let target = match args[0] {
        "condition" | "cond" => {
            if args.len() < 2 {
                return Err(Error::InvalidInput(
                    "Usage: break condition <expr>".to_string(),
                ));
            }

            // Parse condition
            let condition_str = args[1..].join(" ");
            let condition = parse_break_condition(&condition_str)?;
            BreakTarget::Condition(condition)
        }

        "watch" | "w" => {
            if args.len() < 2 {
                return Err(Error::InvalidInput("Usage: break watch <var>".to_string()));
            }

            // Parse variable (scope.name)
            let var_str = args[1];
            let (scope, name) = parse_scoped_variable(var_str)?;
            BreakTarget::Watch { scope, name }
        }

        _ => {
            // Task breakpoint or iteration breakpoint
            let task_str = args[0];

            // Check for iteration syntax: task:iteration
            if task_str.contains(':') {
                let parts: Vec<&str> = task_str.split(':').collect();
                if parts.len() != 2 {
                    return Err(Error::InvalidInput(
                        "Invalid iteration syntax. Use: break task:iteration".to_string(),
                    ));
                }

                let task = parts[0].to_string();
                let iteration = parts[1]
                    .parse()
                    .map_err(|_| Error::InvalidInput("Invalid iteration number".to_string()))?;

                BreakTarget::Iteration { task, iteration }
            } else {
                // Simple task breakpoint
                BreakTarget::Task(task_str.to_string())
            }
        }
    };

    Ok(ReplCommand::Break { target })
}

/// Parse break condition
fn parse_break_condition(condition_str: &str) -> Result<BreakCondition> {
    if condition_str == "error" || condition_str == "onerror" {
        return Ok(BreakCondition::OnError);
    }

    // Try to parse task status condition
    if condition_str.starts_with("task") {
        // Format: task:task_id status:failed
        let parts: Vec<&str> = condition_str.split_whitespace().collect();
        if parts.len() >= 2 {
            let task_id = parts[0]
                .strip_prefix("task:")
                .unwrap_or(parts[0])
                .to_string();

            let status_str = parts[1].strip_prefix("status:").unwrap_or(parts[1]);
            let status = parse_task_status(status_str)?;

            return Ok(BreakCondition::TaskStatus { task_id, status });
        }
    }

    // Default: expression (not fully implemented)
    Ok(BreakCondition::Expression(condition_str.to_string()))
}

/// Parse task status
fn parse_task_status(status_str: &str) -> Result<TaskStatus> {
    match status_str.to_lowercase().as_str() {
        "pending" => Ok(TaskStatus::Pending),
        "ready" => Ok(TaskStatus::Ready),
        "running" => Ok(TaskStatus::Running),
        "completed" | "complete" | "success" => Ok(TaskStatus::Completed),
        "failed" | "fail" | "error" => Ok(TaskStatus::Failed),
        "skipped" | "skip" => Ok(TaskStatus::Skipped),
        _ => Err(Error::InvalidInput(format!(
            "Unknown task status: {}",
            status_str
        ))),
    }
}

/// Parse inspect command
fn parse_inspect_command(args: &[&str]) -> Result<ReplCommand> {
    if args.is_empty() {
        return Err(Error::InvalidInput(
            "Usage: inspect <task|var|state|effects|position>".to_string(),
        ));
    }

    let target = match args[0] {
        "state" => InspectTarget::State,
        "effects" | "side" | "sideeffects" => InspectTarget::SideEffects,
        "position" | "pos" => InspectTarget::Position,
        "task" => {
            if args.len() < 2 {
                return Err(Error::InvalidInput(
                    "Usage: inspect task <task_id>".to_string(),
                ));
            }
            InspectTarget::Task(args[1].to_string())
        }
        "var" | "variable" => {
            if args.len() < 2 {
                return Err(Error::InvalidInput("Usage: inspect var <name>".to_string()));
            }
            InspectTarget::Variable(args[1].to_string())
        }
        _ => {
            // Assume it's a variable or task name
            if args[0].contains('.') {
                InspectTarget::Variable(args[0].to_string())
            } else {
                InspectTarget::Task(args[0].to_string())
            }
        }
    };

    Ok(ReplCommand::Inspect { target })
}

/// Parse set command
fn parse_set_command(args: &[&str]) -> Result<ReplCommand> {
    if args.len() < 3 {
        return Err(Error::InvalidInput(
            "Usage: set <scope>.<var> = <value>".to_string(),
        ));
    }

    // Parse "scope.var = value"
    let var_part = args[0];
    let (scope, name) = parse_scoped_variable(var_part)?;

    // Skip '=' if present
    let value_start = if args[1] == "=" { 2 } else { 1 };
    let value = args[value_start..].join(" ");

    Ok(ReplCommand::Set { scope, name, value })
}

/// Parse scoped variable (e.g., "workflow.project_name")
fn parse_scoped_variable(var_str: &str) -> Result<(VariableScope, String)> {
    let parts: Vec<&str> = var_str.split('.').collect();

    if parts.len() < 2 {
        return Err(Error::InvalidInput(
            "Invalid variable format. Use: scope.name".to_string(),
        ));
    }

    let scope = parse_variable_scope(parts[0])?;
    let name = parts[1..].join(".");

    Ok((scope, name))
}

/// Parse variable scope
fn parse_variable_scope(scope_str: &str) -> Result<VariableScope> {
    match scope_str.to_lowercase().as_str() {
        "workflow" | "wf" => Ok(VariableScope::Workflow),
        scope if scope.starts_with("agent:") || scope.starts_with("a:") => {
            let agent_name = scope
                .strip_prefix("agent:")
                .or_else(|| scope.strip_prefix("a:"))
                .unwrap();
            Ok(VariableScope::Agent(agent_name.to_string()))
        }
        scope if scope.starts_with("task:") || scope.starts_with("t:") => {
            let task_name = scope
                .strip_prefix("task:")
                .or_else(|| scope.strip_prefix("t:"))
                .unwrap();
            Ok(VariableScope::Task(task_name.to_string()))
        }
        _ => {
            // Try to parse as agent or task name directly
            Ok(VariableScope::Task(scope_str.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_commands() {
        assert!(matches!(
            parse_command("continue").unwrap(),
            ReplCommand::Continue
        ));
        assert!(matches!(parse_command("c").unwrap(), ReplCommand::Continue));
        assert!(matches!(parse_command("step").unwrap(), ReplCommand::Step));
        assert!(matches!(parse_command("s").unwrap(), ReplCommand::Step));
        assert!(matches!(parse_command("quit").unwrap(), ReplCommand::Quit));
    }

    #[test]
    fn test_parse_step_commands() {
        assert!(matches!(
            parse_command("stepback").unwrap(),
            ReplCommand::StepBack { steps: 1 }
        ));
        assert!(matches!(
            parse_command("stepback 5").unwrap(),
            ReplCommand::StepBack { steps: 5 }
        ));
        assert!(matches!(
            parse_command("stepforward 3").unwrap(),
            ReplCommand::StepForward { steps: 3 }
        ));
    }

    #[test]
    fn test_parse_break_task() {
        let cmd = parse_command("break my_task").unwrap();
        assert!(matches!(cmd, ReplCommand::Break { .. }));
    }

    #[test]
    fn test_parse_break_iteration() {
        let cmd = parse_command("break loop_task:5").unwrap();
        if let ReplCommand::Break {
            target: BreakTarget::Iteration { task, iteration },
        } = cmd
        {
            assert_eq!(task, "loop_task");
            assert_eq!(iteration, 5);
        } else {
            panic!("Expected iteration breakpoint");
        }
    }

    #[test]
    fn test_parse_vars() {
        assert!(matches!(
            parse_command("vars").unwrap(),
            ReplCommand::Vars { scope: None }
        ));
        assert!(matches!(
            parse_command("vars workflow").unwrap(),
            ReplCommand::Vars { scope: Some(_) }
        ));
    }

    #[test]
    fn test_parse_inspect() {
        let cmd = parse_command("inspect task my_task").unwrap();
        assert!(matches!(cmd, ReplCommand::Inspect { .. }));

        let cmd = parse_command("inspect state").unwrap();
        assert!(matches!(
            cmd,
            ReplCommand::Inspect {
                target: InspectTarget::State
            }
        ));
    }

    #[test]
    fn test_parse_timeline() {
        assert!(matches!(
            parse_command("timeline").unwrap(),
            ReplCommand::Timeline { limit: None }
        ));
        assert!(matches!(
            parse_command("timeline 10").unwrap(),
            ReplCommand::Timeline { limit: Some(10) }
        ));
    }

    #[test]
    fn test_parse_goto() {
        let cmd = parse_command("goto 5").unwrap();
        assert!(matches!(cmd, ReplCommand::Goto { snapshot_id: 5 }));
    }

    #[test]
    fn test_parse_help() {
        assert!(matches!(
            parse_command("help").unwrap(),
            ReplCommand::Help { command: None }
        ));
        assert!(matches!(
            parse_command("help break").unwrap(),
            ReplCommand::Help { command: Some(_) }
        ));
    }

    #[test]
    fn test_parse_unknown_command() {
        assert!(parse_command("foobar").is_err());
    }

    #[test]
    fn test_parse_empty_command() {
        assert!(parse_command("").is_err());
        assert!(parse_command("   ").is_err());
    }
}
