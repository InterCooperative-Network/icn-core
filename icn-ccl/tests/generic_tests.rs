// Test generic functionality
#[cfg(test)]
mod tests {
    use icn_ccl::ast::{AstNode, TopLevelNode, TypeExprNode};
    use icn_ccl::parser::parse_ccl_source;

    #[test]
    fn test_generic_function_parsing() {
        let source = r#"
            fn identity<T>(value: T) -> T {
                return value;
            }
        "#;

        let result = parse_ccl_source(source);
        assert!(
            result.is_ok(),
            "Failed to parse generic function: {:?}",
            result.err()
        );

        if let Ok(AstNode::Program(nodes)) = result {
            assert_eq!(nodes.len(), 1);
            if let TopLevelNode::Function(func) = &nodes[0] {
                assert_eq!(func.name, "identity");
                assert_eq!(func.type_parameters.len(), 1);
                assert_eq!(func.type_parameters[0].name, "T");
                assert_eq!(func.parameters.len(), 1);
                assert_eq!(func.parameters[0].name, "value");

                // Check that parameter type is a type parameter reference
                if let TypeExprNode::Custom(type_name) = &func.parameters[0].type_expr {
                    assert_eq!(type_name, "T");
                }
            } else {
                panic!("Expected function node");
            }
        }
    }

    #[test]
    fn test_generic_struct_parsing() {
        let source = r#"
            struct Container<T> {
                value: T
            }
        "#;

        let result = parse_ccl_source(source);
        assert!(
            result.is_ok(),
            "Failed to parse generic struct: {:?}",
            result.err()
        );

        if let Ok(AstNode::Program(nodes)) = result {
            assert_eq!(nodes.len(), 1);
            if let TopLevelNode::Struct(struct_def) = &nodes[0] {
                assert_eq!(struct_def.name, "Container");
                assert_eq!(struct_def.type_parameters.len(), 1);
                assert_eq!(struct_def.type_parameters[0].name, "T");
                assert_eq!(struct_def.fields.len(), 1);
                assert_eq!(struct_def.fields[0].name, "value");
            } else {
                panic!("Expected struct node");
            }
        }
    }

    #[test]
    fn test_generic_array_type() {
        let source = r#"
            fn create_array() -> Array<String> {
                return [];
            }
        "#;

        let result = parse_ccl_source(source);
        assert!(
            result.is_ok(),
            "Failed to parse Array<String>: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_generic_map_type() {
        let source = r#"
            fn create_map() -> Map<String, Integer> {
                return {};
            }
        "#;

        let result = parse_ccl_source(source);
        assert!(
            result.is_ok(),
            "Failed to parse Map<String, Integer>: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_multiple_type_parameters() {
        let source = r#"
            fn combine<T, U>(first: T, second: U) -> String {
                return "combined";
            }
        "#;

        let result = parse_ccl_source(source);
        assert!(
            result.is_ok(),
            "Failed to parse multiple type parameters: {:?}",
            result.err()
        );

        if let Ok(AstNode::Program(nodes)) = result {
            if let TopLevelNode::Function(func) = &nodes[0] {
                assert_eq!(func.type_parameters.len(), 2);
                assert_eq!(func.type_parameters[0].name, "T");
                assert_eq!(func.type_parameters[1].name, "U");
            }
        }
    }
}
