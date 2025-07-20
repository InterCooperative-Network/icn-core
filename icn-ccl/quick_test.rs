// Quick test to verify CCL 0.1 contract parsing works
use icn_ccl::parser::parse_ccl_source;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing CCL 0.1 Contract Parsing...");
    
    let contract_source = r#"
contract TestContract {
    scope: "test.example.org";
    version: "1.0.0";
    
    fn get_answer() -> Integer {
        return 42;
    }
}
"#;

    match parse_ccl_source(contract_source) {
        Ok(ast) => {
            println!("✅ SUCCESS: Contract parsed correctly!");
            println!("AST: {:#?}", ast);
            Ok(())
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
            Err(e.into())
        }
    }
} 