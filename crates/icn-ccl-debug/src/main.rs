use icn_ccl_debug::{CclDebugger, DebugState};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 ICN CCL Debugger");
    println!("================");
    
    let mut debugger = CclDebugger::new();
    
    loop {
        print!("ccl-debug> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        match parse_command(input) {
            Command::Help => print_help(),
            Command::CreateSession { name, source } => {
                match create_debug_session(&mut debugger, &name, &source) {
                    Ok(session_id) => println!("✅ Created debug session: {}", session_id),
                    Err(e) => println!("❌ Failed to create session: {}", e),
                }
            }
            Command::ListSessions => {
                match debugger.list_sessions() {
                    Ok(sessions) => {
                        if sessions.is_empty() {
                            println!("No active debug sessions");
                        } else {
                            println!("Active sessions:");
                            for session in sessions {
                                println!("  - {}", session);
                            }
                        }
                    }
                    Err(e) => println!("❌ Failed to list sessions: {}", e),
                }
            }
            Command::SetBreakpoint { session, line } => {
                match debugger.set_breakpoint(&session, line) {
                    Ok(_) => println!("✅ Breakpoint set at line {}", line),
                    Err(e) => println!("❌ Failed to set breakpoint: {}", e),
                }
            }
            Command::Start { session } => {
                match debugger.start_execution(&session) {
                    Ok(state) => println!("🚀 Execution started: {:?}", state),
                    Err(e) => println!("❌ Failed to start execution: {}", e),
                }
            }
            Command::Step { session } => {
                match debugger.step_next(&session) {
                    Ok(state) => println!("👣 Stepped: {:?}", state),
                    Err(e) => println!("❌ Failed to step: {}", e),
                }
            }
            Command::Continue { session } => {
                match debugger.continue_execution(&session) {
                    Ok(state) => println!("▶️ Continued: {:?}", state),
                    Err(e) => println!("❌ Failed to continue: {}", e),
                }
            }
            Command::Inspect { session, variable } => {
                match debugger.inspect_variable(&session, &variable) {
                    Ok(value) => println!("🔍 {} = {:?}", variable, value),
                    Err(e) => println!("❌ Failed to inspect: {}", e),
                }
            }
            Command::Stack { session } => {
                match debugger.get_call_stack(&session) {
                    Ok(stack) => {
                        println!("📚 Call stack:");
                        for (i, frame) in stack.iter().enumerate() {
                            println!("  {}. {} ({}:{})", i, frame.function_name, frame.line, frame.column);
                        }
                    }
                    Err(e) => println!("❌ Failed to get stack: {}", e),
                }
            }
            Command::Quit => {
                println!("👋 Goodbye!");
                break;
            }
            Command::Unknown(cmd) => {
                println!("❓ Unknown command: {}. Type 'help' for available commands.", cmd);
            }
        }
    }
    
    Ok(())
}

#[derive(Debug)]
enum Command {
    Help,
    CreateSession { name: String, source: String },
    ListSessions,
    SetBreakpoint { session: String, line: u32 },
    Start { session: String },
    Step { session: String },
    Continue { session: String },
    Inspect { session: String, variable: String },
    Stack { session: String },
    Quit,
    Unknown(String),
}

fn parse_command(input: &str) -> Command {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Command::Unknown(input.to_string());
    }
    
    match parts[0] {
        "help" | "h" => Command::Help,
        "create" | "new" => {
            if parts.len() >= 3 {
                Command::CreateSession {
                    name: parts[1].to_string(),
                    source: parts[2..].join(" "),
                }
            } else {
                Command::Unknown(input.to_string())
            }
        }
        "list" | "ls" => Command::ListSessions,
        "break" | "b" => {
            if parts.len() >= 3 {
                if let Ok(line) = parts[2].parse::<u32>() {
                    Command::SetBreakpoint {
                        session: parts[1].to_string(),
                        line,
                    }
                } else {
                    Command::Unknown(input.to_string())
                }
            } else {
                Command::Unknown(input.to_string())
            }
        }
        "start" | "run" => {
            if parts.len() >= 2 {
                Command::Start {
                    session: parts[1].to_string(),
                }
            } else {
                Command::Unknown(input.to_string())
            }
        }
        "step" | "s" => {
            if parts.len() >= 2 {
                Command::Step {
                    session: parts[1].to_string(),
                }
            } else {
                Command::Unknown(input.to_string())
            }
        }
        "continue" | "c" => {
            if parts.len() >= 2 {
                Command::Continue {
                    session: parts[1].to_string(),
                }
            } else {
                Command::Unknown(input.to_string())
            }
        }
        "inspect" | "print" | "p" => {
            if parts.len() >= 3 {
                Command::Inspect {
                    session: parts[1].to_string(),
                    variable: parts[2].to_string(),
                }
            } else {
                Command::Unknown(input.to_string())
            }
        }
        "stack" | "backtrace" | "bt" => {
            if parts.len() >= 2 {
                Command::Stack {
                    session: parts[1].to_string(),
                }
            } else {
                Command::Unknown(input.to_string())
            }
        }
        "quit" | "exit" | "q" => Command::Quit,
        _ => Command::Unknown(input.to_string()),
    }
}

fn print_help() {
    println!(r#"
🔍 CCL Debugger Commands:

  create <name> <source>     Create a new debug session
  list                       List all active sessions
  break <session> <line>     Set breakpoint at line
  start <session>            Start execution
  step <session>             Step to next line
  continue <session>         Continue execution
  inspect <session> <var>    Inspect variable value
  stack <session>            Show call stack
  help                       Show this help message
  quit                       Exit debugger

Examples:
  create test "fn main() -> Integer { return 42; }"
  break test 1
  start test
  step test
  inspect test balance
"#);
}

fn create_debug_session(
    debugger: &mut CclDebugger,
    name: &str,
    source: &str,
) -> Result<String, String> {
    let session_id = format!("session_{}", name);
    debugger.create_session(
        session_id.clone(),
        name.to_string(),
        format!("{}.ccl", name),
        source,
    )?;
    Ok(session_id)
}