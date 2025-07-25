// icn-ccl/src/bin/ccl-debug.rs
//! CCL Debugger binary
//!
//! This binary provides an interactive debugger for CCL contracts.

use icn_ccl::{
    compile_ccl_source_to_wasm, create_console_debugger, debugger::StepMode, CclError,
    DebugSession,
};
use std::env;
use std::fs;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: ccl-debug <ccl_file> [function_name]");
        std::process::exit(1);
    }

    let ccl_file = &args[1];
    let function_name = args.get(2).map(|s| s.as_str()).unwrap_or("main");

    // Read and compile the CCL file
    let source = fs::read_to_string(ccl_file)?;
    let (wasm_bytes, metadata) = compile_ccl_source_to_wasm(&source)?;

    // Create debugger
    let mut debugger = create_console_debugger()?;

    // Load contract for debugging
    let session = debugger.load_contract(&wasm_bytes, &metadata).await?;

    println!("🐛 CCL Debugger");
    println!("File: {}", ccl_file);
    println!("Function: {}", function_name);
    println!("Commands: (s)tep, (c)ontinue, (b)reakpoint, (q)uit");
    println!();

    // Interactive debugger loop
    loop {
        print!("ccl-debug> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "s" | "step" => {
                println!("Step mode set");
            }
            "c" | "continue" => {
                println!("Continue execution");
            }
            cmd if cmd.starts_with("b ") => {
                if let Ok(offset) = cmd[2..].parse::<u32>() {
                    println!("Breakpoint set at instruction {}", offset);
                } else {
                    println!("Invalid breakpoint offset");
                }
            }
            "q" | "quit" => break,
            "h" | "help" => {
                println!("Commands:");
                println!("  s, step      - Step into next instruction");
                println!("  c, continue  - Continue execution");
                println!("  b <offset>   - Set breakpoint at instruction offset");
                println!("  q, quit      - Quit debugger");
                println!("  h, help      - Show this help");
            }
            "" => {
                println!("Debugger ready");
            }
            _ => {
                println!("Unknown command. Type 'h' for help.");
            }
        }
    }

    println!("Debugger exited.");
    Ok(())
}

async fn execute_function(
    session: &DebugSession<'_>,
    function_name: &str,
) -> Result<(), CclError> {
    // Simple execution with no arguments for now
    let args = vec![];
    let _result = session.execute_with_debug(function_name, &args).await?;
    Ok(())
}