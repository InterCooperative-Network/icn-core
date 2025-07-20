// debug_equality.rs
// Debug the equality parsing to see what Pest generates

use icn_ccl::parser::parse_ccl_source;

fn main() {
    let source = r#"
        contract TestDebug {
            scope: "test";
            version: "1.0.0";
            
            fn test_eq() -> Integer {
                let a = 5;
                let b = 5;
                return a == b;
            }
        }
    "#;
    
    println!("Parsing source to see AST structure...");
    match parse_ccl_source(source) {
        Ok(ast) => {
            println!("✅ AST parsed successfully!");
            println!("AST structure: {:#?}", ast);
        }
        Err(e) => {
            println!("❌ AST parsing failed: {}", e);
        }
    }
    
    // Test even simpler case
    let simple_source = r#"
        contract TestSimple {
            scope: "test";
            version: "1.0.0";
            
            fn test() -> Integer {
                return 5 == 5;
            }
        }
    "#;
    
    println!("\nParsing simple equality...");
    match parse_ccl_source(simple_source) {
        Ok(ast) => {
            println!("✅ Simple equality parsed successfully!");
            println!("AST structure: {:#?}", ast);
        }
        Err(e) => {
            println!("❌ Simple equality parsing failed: {}", e);
        }
    }
} 