use icn_ccl::parser::parse_ccl_source;

fn main() {
    // Test basic generic function parsing
    let source = r#"
        fn identity<T>(value: T) -> T {
            return value;
        }
    "#;
    
    match parse_ccl_source(source) {
        Ok(ast) => {
            println!("SUCCESS: Generic function parsed successfully!");
            println!("AST: {:#?}", ast);
        }
        Err(e) => {
            println!("ERROR: Failed to parse generic function: {:?}", e);
        }
    }
    
    // Test generic struct parsing
    let source2 = r#"
        struct Container<T> {
            value: T
        }
    "#;
    
    match parse_ccl_source(source2) {
        Ok(ast) => {
            println!("SUCCESS: Generic struct parsed successfully!");
            println!("AST: {:#?}", ast);
        }
        Err(e) => {
            println!("ERROR: Failed to parse generic struct: {:?}", e);
        }
    }
}