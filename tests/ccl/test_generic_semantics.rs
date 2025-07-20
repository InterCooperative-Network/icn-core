use icn_ccl::{parser::parse_ccl_source, semantic_analyzer::SemanticAnalyzer};

fn main() {
    // Test basic generic function semantic analysis
    let source = r#"
        fn identity<T>(value: T) -> T {
            return value;
        }
    "#;
    
    let result = parse_ccl_source(source);
    match result {
        Ok(ast) => {
            println!("✓ Generic function parsed successfully");
            
            // Test semantic analysis
            let mut analyzer = SemanticAnalyzer::new();
            match analyzer.analyze(&ast) {
                Ok(()) => {
                    println!("✓ Generic function semantic analysis passed!");
                }
                Err(errors) => {
                    println!("✗ Semantic analysis failed:");
                    for error in errors {
                        println!("  - {:?}", error);
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to parse: {:?}", e);
        }
    }
    
    // Test generic struct with field usage
    let source2 = r#"
        struct Container<T> {
            value: T
        }
        
        fn get_value<T>(container: Container<T>) -> T {
            return container.value;
        }
    "#;
    
    let result2 = parse_ccl_source(source2);
    match result2 {
        Ok(ast) => {
            println!("✓ Generic struct + function parsed successfully");
            
            let mut analyzer = SemanticAnalyzer::new();
            match analyzer.analyze(&ast) {
                Ok(()) => {
                    println!("✓ Generic struct + function semantic analysis passed!");
                }
                Err(errors) => {
                    println!("✗ Semantic analysis failed:");
                    for error in errors {
                        println!("  - {:?}", error);
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to parse: {:?}", e);
        }
    }
}