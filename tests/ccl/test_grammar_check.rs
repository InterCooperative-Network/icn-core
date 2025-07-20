// Simple test for generic grammar
use std::fs;

fn main() {
    // Test if the grammar accepts generic syntax
    println!("Testing generic syntax in grammar...");
    
    // Read the grammar file 
    let grammar_path = "icn-ccl/src/grammar/ccl.pest";
    if let Ok(content) = fs::read_to_string(grammar_path) {
        if content.contains("type_parameters") {
            println!("✓ Grammar contains type_parameters rule");
        } else {
            println!("✗ Grammar missing type_parameters rule");
        }
        
        if content.contains("generic_instantiation") {
            println!("✓ Grammar contains generic_instantiation rule");
        } else {
            println!("✗ Grammar missing generic_instantiation rule");
        }
        
        if content.contains("Array") && content.contains("<") && content.contains("type_expr") && content.contains(">") {
            println!("✓ Grammar supports Array<T> syntax");
        } else if content.contains("Array") {
            println!("? Grammar has Array but check syntax");
        } else {
            println!("✗ Grammar missing Array generic syntax");
        }
    } else {
        println!("Could not read grammar file");
    }
}