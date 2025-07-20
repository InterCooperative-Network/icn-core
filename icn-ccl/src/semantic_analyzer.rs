// icn-ccl/src/semantic_analyzer.rs
use crate::ast::{
    AstNode, BinaryOperator, BlockNode, ExpressionNode, LiteralNode,
    ParameterNode, PolicyStatementNode, StatementNode, TypeAnnotationNode,
    TypeExprNode, UnaryOperator, ContractDeclarationNode,
    RoleDeclarationNode, ProposalDeclarationNode, StateDeclarationNode,
    ConstDeclarationNode, EnumDefinitionNode,
    FieldNode, LValueNode,
};
use crate::error::CclError;
use crate::stdlib::StdLibrary;
use std::collections::HashMap;

/// Symbol table entry containing type information
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: TypeAnnotationNode,
    pub is_mutable: bool,
    pub scope_level: usize,
}

/// Function signature for type checking
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<TypeAnnotationNode>,
    pub return_type: TypeAnnotationNode,
}

/// Represents a struct type definition
#[derive(Debug, Clone)]
pub struct StructType {
    pub name: String,
    pub fields: HashMap<String, TypeAnnotationNode>,
}

/// The semantic analyzer performs type checking and ensures the AST is semantically valid
pub struct SemanticAnalyzer {
    symbol_table: HashMap<String, Symbol>,
    function_table: HashMap<String, FunctionSignature>,
    struct_table: HashMap<String, StructType>, // Track struct definitions
    current_scope_level: usize,
    current_return_type: Option<TypeAnnotationNode>,
    errors: Vec<CclError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = SemanticAnalyzer {
            symbol_table: HashMap::new(),
            function_table: HashMap::new(),
            struct_table: HashMap::new(),
            current_scope_level: 0,
            current_return_type: None,
            errors: Vec::new(),
        };
        
        // Add built-in functions
        analyzer.add_builtin_functions();
        
        analyzer
    }

    /// Add built-in functions from the standard library to the function table
    fn add_builtin_functions(&mut self) {
        let stdlib = StdLibrary::new();
        
        // Add all standard library functions to the function table
        for (name, std_func) in stdlib.get_all_function_pairs() {
            self.function_table.insert(name.clone(), FunctionSignature {
                name: std_func.name.clone(),
                params: std_func.params.clone(),
                return_type: std_func.return_type.clone(),
            });
        }
    }

    /// Analyze the entire AST for semantic correctness
    pub fn analyze(&mut self, ast: &AstNode) -> Result<(), Vec<CclError>> {
        match self.analyze_node(ast) {
            Ok(()) => {
                if self.errors.is_empty() {
                    Ok(())
                } else {
                    Err(std::mem::take(&mut self.errors))
                }
            }
            Err(err) => {
                self.errors.push(err);
                Err(std::mem::take(&mut self.errors))
            }
        }
    }

    /// Analyze a specific AST node
    fn analyze_node(&mut self, node: &AstNode) -> Result<(), CclError> {
        match node {
            AstNode::Program(nodes) => {
                for node in nodes {
                    match node {
                        crate::ast::TopLevelNode::Import(_) => {
                            // Import validation would be done during parsing/linking
                        }
                        crate::ast::TopLevelNode::Contract(contract) => {
                            self.analyze_contract(contract)?;
                        }
                    }
                }
            }
            AstNode::Policy(statements) => {
                for stmt in statements {
                    self.analyze_policy_statement(stmt)?;
                }
            }
            AstNode::FunctionDefinition { name, parameters, return_type, body } => {
                self.analyze_function_definition(name, parameters, return_type.as_ref(), body)?;
            }
            AstNode::StructDefinition { name, fields } => {
                self.analyze_struct_definition(name, fields)?;
            }
            AstNode::ContractDeclaration { name, metadata: _, body } => {
                self.analyze_contract_body(name, body)?;
            }
            AstNode::Block(block) => {
                self.analyze_block(block)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn analyze_contract(&mut self, contract: &ContractDeclarationNode) -> Result<(), CclError> {
        // Enter contract scope
        self.enter_scope();
        
        // First pass: Register struct types so they're available for function analysis
        for item in &contract.body {
            if let crate::ast::ContractBodyNode::Struct(struct_def) = item {
                self.analyze_struct_definition(&struct_def.name, &struct_def.fields)?;
                self.register_struct_type(struct_def)?;
            }
        }
        
        // Second pass: Analyze everything else (now that struct types are known)
        for item in &contract.body {
            match item {
                crate::ast::ContractBodyNode::Role(role) => {
                    self.analyze_role(role)?;
                }
                crate::ast::ContractBodyNode::Proposal(proposal) => {
                    self.analyze_proposal(proposal)?;
                }
                crate::ast::ContractBodyNode::Function(func) => {
                    self.analyze_function_definition(&func.name, &func.parameters, func.return_type.as_ref(), &func.body)?;
                }
                crate::ast::ContractBodyNode::State(state) => {
                    self.analyze_state_declaration(state)?;
                }
                crate::ast::ContractBodyNode::Enum(enum_def) => {
                    self.analyze_enum_definition(enum_def)?;
                }
                crate::ast::ContractBodyNode::Const(const_def) => {
                    self.analyze_const_declaration(const_def)?;
                }
                _ => {
                    // Structs already processed in first pass
                }
            }
        }
        
        // Exit contract scope
        self.exit_scope();
        
        Ok(())
    }

    fn analyze_role(&mut self, _role: &RoleDeclarationNode) -> Result<(), CclError> {
        // Role analysis - mostly syntactic validation
        // Could add permission validation logic here
        Ok(())
    }

    fn analyze_proposal(&mut self, proposal: &ProposalDeclarationNode) -> Result<(), CclError> {
        // Analyze proposal fields
        for field in &proposal.fields {
            match field {
                crate::ast::ProposalFieldNode::Execution(block) => {
                    self.analyze_block(block)?;
                }
                _ => {
                    // Other fields are mostly metadata
                }
            }
        }
        Ok(())
    }

    fn analyze_function_definition(
        &mut self,
        name: &str,
        parameters: &[ParameterNode],
        return_type: Option<&TypeExprNode>,
        body: &BlockNode,
    ) -> Result<(), CclError> {
        // Register function in function table
        let return_type_ann = return_type
            .map(|rt| rt.to_type_annotation())
            .unwrap_or(TypeAnnotationNode::Custom("void".to_string()));
            
        self.function_table.insert(name.to_string(), FunctionSignature {
            name: name.to_string(),
            params: parameters.iter().map(|p| p.type_expr.to_type_annotation()).collect(),
            return_type: return_type_ann.clone(),
        });

        // Enter function scope
        self.enter_scope();
        self.current_return_type = Some(return_type_ann);

        // Add parameters to symbol table
        for param in parameters {
            self.symbol_table.insert(param.name.clone(), Symbol {
                name: param.name.clone(),
                symbol_type: param.type_expr.to_type_annotation(),
                is_mutable: false,
                scope_level: self.current_scope_level,
            });
        }

        // Analyze function body
        self.analyze_block(body)?;

        // Exit function scope
        self.exit_scope();
        self.current_return_type = None;

        Ok(())
    }

    fn analyze_struct_definition(&mut self, name: &str, fields: &[FieldNode]) -> Result<(), CclError> {
        // Validate field names are unique
        let mut field_names = std::collections::HashSet::new();
        for field in fields {
            if !field_names.insert(&field.name) {
                return Err(CclError::DuplicateFieldError {
                    struct_name: name.to_string(),
                    field_name: field.name.clone(),
                });
            }
        }

        // Register struct type
        let struct_type = TypeAnnotationNode::Custom(name.to_string());
        self.symbol_table.insert(name.to_string(), Symbol {
            name: name.to_string(),
            symbol_type: struct_type,
            is_mutable: false,
            scope_level: self.current_scope_level,
        });

        Ok(())
    }

    fn analyze_enum_definition(&mut self, enum_def: &EnumDefinitionNode) -> Result<(), CclError> {
        // Register enum type
        let enum_type = TypeAnnotationNode::Custom(enum_def.name.clone());
        self.symbol_table.insert(enum_def.name.clone(), Symbol {
            name: enum_def.name.clone(),
            symbol_type: enum_type.clone(), // Clone to avoid move
            is_mutable: false,
            scope_level: self.current_scope_level,
        });

        // Register enum variants
        for variant in &enum_def.variants {
            let variant_type = if variant.type_expr.is_some() {
                // Constructor function
                TypeAnnotationNode::Custom(format!("{}::{}", enum_def.name, variant.name))
            } else {
                // Unit variant
                enum_type.clone()
            };
            
            self.symbol_table.insert(
                format!("{}::{}", enum_def.name, variant.name),
                Symbol {
                    name: format!("{}::{}", enum_def.name, variant.name),
                    symbol_type: variant_type,
                    is_mutable: false,
                    scope_level: self.current_scope_level,
                }
            );
        }

        Ok(())
    }

    fn analyze_state_declaration(&mut self, state: &StateDeclarationNode) -> Result<(), CclError> {
        // Check initial value type if present
        if let Some(ref initial_value) = state.initial_value {
            let value_type = self.evaluate_expression(initial_value)?;
            let expected_type = state.type_expr.to_type_annotation();
            
            if !value_type.compatible_with(&expected_type) {
                return Err(CclError::TypeMismatchError {
                    expected: expected_type,
                    found: value_type,
                });
            }
        }

        // Register state variable
        self.symbol_table.insert(state.name.clone(), Symbol {
            name: state.name.clone(),
            symbol_type: state.type_expr.to_type_annotation(),
            is_mutable: true, // State variables are mutable by default
            scope_level: self.current_scope_level,
        });

        Ok(())
    }

    fn analyze_const_declaration(&mut self, const_def: &ConstDeclarationNode) -> Result<(), CclError> {
        // Check value type
        let value_type = self.evaluate_expression(&const_def.value)?;
        let expected_type = const_def.type_expr.to_type_annotation();
        
        if !value_type.compatible_with(&expected_type) {
            return Err(CclError::TypeMismatchError {
                expected: expected_type,
                found: value_type,
            });
        }

        // Register constant
        self.symbol_table.insert(const_def.name.clone(), Symbol {
            name: const_def.name.clone(),
            symbol_type: expected_type,
            is_mutable: false,
            scope_level: self.current_scope_level,
        });

        Ok(())
    }

    fn analyze_contract_body(&mut self, _name: &str, body: &[crate::ast::ContractBodyNode]) -> Result<(), CclError> {
        // First pass: Register all struct types
        for item in body {
            if let crate::ast::ContractBodyNode::Struct(struct_def) = item {
                self.analyze_struct_definition(&struct_def.name, &struct_def.fields)?;
                self.register_struct_type(struct_def)?;
            }
        }
        
        // Second pass: Analyze functions (now that struct types are known)
        for item in body {
            match item {
                crate::ast::ContractBodyNode::Function(func) => {
                    self.analyze_function_definition(&func.name, &func.parameters, func.return_type.as_ref(), &func.body)?;
                }
                _ => {
                    // Other items already processed or analyzed separately
                }
            }
        }
        Ok(())
    }

    /// Register a struct type in the struct table for later type checking
    fn register_struct_type(&mut self, struct_def: &crate::ast::StructDefinitionNode) -> Result<(), CclError> {
        let mut fields = HashMap::new();
        
        for field in &struct_def.fields {
            let field_type = field.type_expr.to_type_annotation();
            fields.insert(field.name.clone(), field_type);
        }
        
        let struct_type = StructType {
            name: struct_def.name.clone(),
            fields,
        };
        
        self.struct_table.insert(struct_def.name.clone(), struct_type);
        Ok(())
    }

    fn analyze_policy_statement(&mut self, stmt: &PolicyStatementNode) -> Result<(), CclError> {
        match stmt {
            PolicyStatementNode::FunctionDef(func_ast) => {
                if let AstNode::FunctionDefinition { name, parameters, return_type, body } = func_ast {
                    self.analyze_function_definition(name, parameters, return_type.as_ref(), body)?;
                }
            }
            PolicyStatementNode::StructDef(struct_ast) => {
                if let AstNode::StructDefinition { name, fields } = struct_ast {
                    self.analyze_struct_definition(name, fields)?;
                }
            }
            PolicyStatementNode::ConstDef { name, value, type_ann } => {
                let value_type = self.evaluate_expression(value)?;
                if !value_type.compatible_with(type_ann) {
                    return Err(CclError::TypeMismatchError {
                        expected: type_ann.clone(),
                        found: value_type,
                    });
                }
                self.symbol_table.insert(name.clone(), Symbol {
                    name: name.clone(),
                    symbol_type: type_ann.clone(),
                    is_mutable: false,
                    scope_level: self.current_scope_level,
                });
            }
            _ => {
                // Other statements don't need special analysis
            }
        }
        Ok(())
    }

    fn analyze_block(&mut self, block: &BlockNode) -> Result<(), CclError> {
        self.enter_scope();
        
        for stmt in &block.statements {
            self.analyze_statement(stmt)?;
        }
        
        self.exit_scope();
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &StatementNode) -> Result<(), CclError> {
        match stmt {
            StatementNode::Let { mutable, name, type_expr, value } => {
                let value_type = self.evaluate_expression(value)?;
                
                let expected_type = if let Some(type_expr) = type_expr {
                    type_expr.to_type_annotation()
                } else {
                    value_type.clone()
                };

                if !value_type.compatible_with(&expected_type) {
                    return Err(CclError::TypeMismatchError {
                        expected: expected_type.clone(),
                        found: value_type,
                    });
                }

                self.symbol_table.insert(name.clone(), Symbol {
                    name: name.clone(),
                    symbol_type: expected_type,
                    is_mutable: *mutable,
                    scope_level: self.current_scope_level,
                });
            }
            StatementNode::Assignment { lvalue, value } => {
                let value_type = self.evaluate_expression(value)?;
                let lvalue_type = self.evaluate_lvalue(lvalue)?;
                
                if !value_type.compatible_with(&lvalue_type) {
                    return Err(CclError::TypeMismatchError {
                        expected: lvalue_type,
                        found: value_type,
                    });
                }
            }
            StatementNode::If { condition, then_block, else_ifs, else_block } => {
                let cond_type = self.evaluate_expression(condition)?;
                if !matches!(cond_type, TypeAnnotationNode::Bool) {
                    return Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Bool,
                        found: cond_type,
                    });
                }

                self.analyze_block(then_block)?;
                
                for (elif_cond, elif_block) in else_ifs {
                    let elif_cond_type = self.evaluate_expression(elif_cond)?;
                    if !matches!(elif_cond_type, TypeAnnotationNode::Bool) {
                        return Err(CclError::TypeMismatchError {
                            expected: TypeAnnotationNode::Bool,
                            found: elif_cond_type,
                        });
                    }
                    self.analyze_block(elif_block)?;
                }
                
                if let Some(else_blk) = else_block {
                    self.analyze_block(else_blk)?;
                }
            }
            StatementNode::While { condition, body } => {
                let cond_type = self.evaluate_expression(condition)?;
                if !matches!(cond_type, TypeAnnotationNode::Bool) {
                    return Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Bool,
                        found: cond_type,
                    });
                }
                self.analyze_block(body)?;
            }
            StatementNode::For { iterator, iterable, body } => {
                let iterable_type = self.evaluate_expression(iterable)?;
                
                // Determine the element type based on the iterable type
                let element_type = match &iterable_type {
                    TypeAnnotationNode::Array(inner_type) => (**inner_type).clone(),
                    _ => {
                        return Err(CclError::SemanticError(format!(
                            "Cannot iterate over non-array type: {:?}",
                            iterable_type
                        )));
                    }
                };
                
                // Create a new scope for the loop body
                self.enter_scope();
                
                // Declare the iterator variable in the loop scope
                self.symbol_table.insert(iterator.clone(), Symbol {
                    name: iterator.clone(),
                    symbol_type: element_type,
                    is_mutable: false, // Iterator variables are immutable
                    scope_level: self.current_scope_level,
                });
                
                // Analyze the loop body
                self.analyze_block(body)?;
                
                // Exit the loop scope
                self.exit_scope();
            }
            StatementNode::Match { expr, arms } => {
                let _expr_type = self.evaluate_expression(expr)?;
                
                for arm in arms {
                    // TODO: Pattern type checking
                    match &arm.body {
                        crate::ast::Either::Left(expr) => {
                            self.evaluate_expression(expr)?;
                        }
                        crate::ast::Either::Right(block) => {
                            self.analyze_block(block)?;
                        }
                    }
                }
            }
            StatementNode::Return(expr) => {
                if let Some(expr) = expr {
                    let expr_type = self.evaluate_expression(expr)?;
                    if let Some(expected_return_type) = &self.current_return_type {
                        if !expr_type.compatible_with(expected_return_type) {
                            return Err(CclError::TypeMismatchError {
                                expected: expected_return_type.clone(),
                                found: expr_type,
                            });
                        }
                    }
                }
            }
            StatementNode::Emit { event_name: _, fields } => {
                for field in fields {
                    self.evaluate_expression(&field.value)?;
                }
            }
            StatementNode::Require(expr) => {
                let expr_type = self.evaluate_expression(expr)?;
                if !matches!(expr_type, TypeAnnotationNode::Bool) {
                    return Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Bool,
                        found: expr_type,
                    });
                }
            }
            StatementNode::ExpressionStatement(expr) => {
                self.evaluate_expression(expr)?;
            }
            // Legacy statements
            StatementNode::WhileLoop { condition, body } => {
                let cond_type = self.evaluate_expression(condition)?;
                if !matches!(cond_type, TypeAnnotationNode::Bool) {
                    return Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Bool,
                        found: cond_type,
                    });
                }
                self.analyze_block(body)?;
            }
            StatementNode::ForLoop { iterator: _, iterable, body } => {
                let _iterable_type = self.evaluate_expression(iterable)?;
                self.analyze_block(body)?;
            }
            StatementNode::Break | StatementNode::Continue => {
                // Control flow statements don't need type checking
            }
        }
        Ok(())
    }

    fn evaluate_lvalue(&mut self, lvalue: &LValueNode) -> Result<TypeAnnotationNode, CclError> {
        match lvalue {
            LValueNode::Identifier(name) => {
                if let Some(symbol) = self.symbol_table.get(name) {
                    if !symbol.is_mutable {
                        return Err(CclError::ImmutableAssignmentError {
                            variable: name.clone(),
                        });
                    }
                    Ok(symbol.symbol_type.clone())
                } else {
                    Err(CclError::UndefinedVariableError {
                        variable: name.clone(),
                    })
                }
            }
            LValueNode::MemberAccess { object, member: _ } => {
                let _object_type = self.evaluate_expression(object)?;
                // TODO: Implement struct member type lookup
                Ok(TypeAnnotationNode::Custom("member".to_string()))
            }
            LValueNode::IndexAccess { object, index } => {
                let object_type = self.evaluate_expression(object)?;
                let index_type = self.evaluate_expression(index)?;
                
                if !matches!(index_type, TypeAnnotationNode::Integer) {
                    return Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Integer,
                        found: index_type,
                    });
                }
                
                match object_type {
                    TypeAnnotationNode::Array(element_type) => Ok(*element_type),
                    _ => Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Custom("T".to_string()))),
                        found: object_type,
                    }),
                }
            }
        }
    }

    /// Evaluate the type of an expression
    fn evaluate_expression(
        &mut self,
        expr: &ExpressionNode,
    ) -> Result<TypeAnnotationNode, CclError> {
        match expr {
            ExpressionNode::Literal(lit) => Ok(self.literal_type(lit)),
            ExpressionNode::Identifier(name) => {
                if let Some(symbol) = self.symbol_table.get(name) {
                    Ok(symbol.symbol_type.clone())
                } else {
                    Err(CclError::UndefinedVariableError {
                        variable: name.clone(),
                    })
                }
            }
            ExpressionNode::BinaryOp { left, operator, right } => {
                let left_type = self.evaluate_expression(left)?;
                let right_type = self.evaluate_expression(right)?;
                self.check_binary_op(&left_type, operator, &right_type)
            }
            ExpressionNode::UnaryOp { operator, operand } => {
                let operand_type = self.evaluate_expression(operand)?;
                self.check_unary_op(operator, &operand_type)
            }
            ExpressionNode::FunctionCall { name, args } => {
                // Clone the signature to avoid borrow conflicts
                if let Some(signature) = self.function_table.get(name).cloned() {
                    if args.len() != signature.params.len() {
                        return Err(CclError::ArgumentCountMismatchError {
                            function: name.clone(),
                            expected: signature.params.len(),
                            found: args.len(),
                        });
                    }
                    
                    for (arg, expected_type) in args.iter().zip(&signature.params) {
                        let arg_type = self.evaluate_expression(arg)?;
                        if !arg_type.compatible_with(expected_type) {
                            return Err(CclError::TypeMismatchError {
                                expected: expected_type.clone(),
                                found: arg_type,
                            });
                        }
                    }
                    
                    Ok(signature.return_type.clone())
                } else {
                    Err(CclError::UndefinedFunctionError {
                        function: name.clone(),
                    })
                }
            }
            ExpressionNode::MethodCall { object, method, args } => {
                let object_type = self.evaluate_expression(object)?;
                
                match method.as_str() {
                    "length" => {
                        // Validate that object is an array or string
                        match object_type {
                            TypeAnnotationNode::Array(_) | TypeAnnotationNode::String => {
                                // length() takes no arguments
                                if !args.is_empty() {
                                    return Err(CclError::ArgumentCountMismatchError {
                                        function: format!("{}.length", "array_or_string"),
                                        expected: 0,
                                        found: args.len(),
                                    });
                                }
                                Ok(TypeAnnotationNode::Integer)
                            }
                            _ => {
                                Err(CclError::SemanticError(format!(
                                    "Method 'length' is only available on arrays and strings, got: {:?}",
                                    object_type
                                )))
                            }
                        }
                    }
                    _ => {
                        Err(CclError::UndefinedFunctionError {
                            function: format!("method {}", method),
                        })
                    }
                }
            }
            ExpressionNode::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    Ok(TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Custom("unknown".to_string()))))
                } else {
                    let first_type = self.evaluate_expression(&elements[0])?;
                    for element in &elements[1..] {
                        let element_type = self.evaluate_expression(element)?;
                        if !element_type.compatible_with(&first_type) {
                            return Err(CclError::TypeMismatchError {
                                expected: first_type,
                                found: element_type,
                            });
                        }
                    }
                    Ok(TypeAnnotationNode::Array(Box::new(first_type)))
                }
            }
            ExpressionNode::StructLiteral { type_name, fields: _ } => {

                // Validate that the struct type exists
                if !self.struct_table.contains_key(type_name) {
                    return Err(CclError::SemanticError(format!("Unknown struct type: {}", type_name)));
                }
                // TODO: Validate struct fields against definition
                Ok(TypeAnnotationNode::Custom(type_name.clone()))
            }
            ExpressionNode::MemberAccess { object, member } => {
                let object_type = self.evaluate_expression(object)?;
                
                // Check if the object is a struct type and get the member type
                match object_type {
                    TypeAnnotationNode::Custom(struct_name) => {
                        if let Some(struct_type) = self.struct_table.get(&struct_name) {
                            if let Some(member_type) = struct_type.fields.get(member) {
                                Ok(member_type.clone())
                            } else {
                                Err(CclError::SemanticError(format!(
                                    "Struct {} has no member named {}", struct_name, member
                                )))
                            }
                        } else {
                            Err(CclError::SemanticError(format!(
                                "Cannot access member {} on non-struct type", member
                            )))
                        }
                    }
                    _ => {
                        Err(CclError::SemanticError(format!(
                            "Cannot access member {} on type {:?}", member, object_type
                        )))
                    }
                }
            }
            ExpressionNode::IndexAccess { object, index } => {
                let object_type = self.evaluate_expression(object)?;
                let index_type = self.evaluate_expression(index)?;
                
                if !matches!(index_type, TypeAnnotationNode::Integer) {
                    return Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Integer,
                        found: index_type,
                    });
                }
                
                match object_type {
                    TypeAnnotationNode::Array(element_type) => Ok(*element_type),
                    _ => Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Custom("T".to_string()))),
                        found: object_type,
                    }),
                }
            }
            ExpressionNode::Some(inner) => {
                let inner_type = self.evaluate_expression(inner)?;
                Ok(TypeAnnotationNode::Option(Box::new(inner_type)))
            }
            ExpressionNode::None => Ok(TypeAnnotationNode::Option(Box::new(
                TypeAnnotationNode::Custom("unknown".to_string())
            ))),
            ExpressionNode::Ok(inner) => {
                let inner_type = self.evaluate_expression(inner)?;
                Ok(TypeAnnotationNode::Result {
                    ok_type: Box::new(inner_type),
                    err_type: Box::new(TypeAnnotationNode::Custom("unknown".to_string())),
                })
            }
            ExpressionNode::Err(inner) => {
                let inner_type = self.evaluate_expression(inner)?;
                Ok(TypeAnnotationNode::Result {
                    ok_type: Box::new(TypeAnnotationNode::Custom("unknown".to_string())),
                    err_type: Box::new(inner_type),
                })
            }
            // Governance expressions
            ExpressionNode::Transfer { from: _, to: _, amount } => {
                let amount_type = self.evaluate_expression(amount)?;
                if !matches!(amount_type, TypeAnnotationNode::Mana | TypeAnnotationNode::Integer) {
                    return Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Mana,
                        found: amount_type,
                    });
                }
                Ok(TypeAnnotationNode::Bool)
            }
            ExpressionNode::Mint { to: _, amount } => {
                let amount_type = self.evaluate_expression(amount)?;
                if !matches!(amount_type, TypeAnnotationNode::Mana | TypeAnnotationNode::Integer) {
                    return Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Mana,
                        found: amount_type,
                    });
                }
                Ok(TypeAnnotationNode::Bool)
            }
            ExpressionNode::Burn { from: _, amount } => {
                let amount_type = self.evaluate_expression(amount)?;
                if !matches!(amount_type, TypeAnnotationNode::Mana | TypeAnnotationNode::Integer) {
                    return Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Mana,
                        found: amount_type,
                    });
                }
                Ok(TypeAnnotationNode::Bool)
            }
            // Legacy expressions
            ExpressionNode::IntegerLiteral(_) => Ok(TypeAnnotationNode::Integer),
            ExpressionNode::StringLiteral(_) => Ok(TypeAnnotationNode::String),
            ExpressionNode::BooleanLiteral(_) => Ok(TypeAnnotationNode::Bool),
            ExpressionNode::EnumValue { enum_name, variant: _ } => {
                // For now, just return the enum type
                Ok(TypeAnnotationNode::Custom(enum_name.clone()))
            }
            ExpressionNode::ArrayAccess { array, index } => {
                let array_type = self.evaluate_expression(array)?;
                let index_type = self.evaluate_expression(index)?;
                
                if !matches!(index_type, TypeAnnotationNode::Integer) {
                    return Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Integer,
                        found: index_type,
                    });
                }
                
                match array_type {
                    TypeAnnotationNode::Array(element_type) => Ok(*element_type),
                    _ => Err(CclError::TypeMismatchError {
                        expected: TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Custom("T".to_string()))),
                        found: array_type,
                    }),
                }
            }
        }
    }

    fn literal_type(&self, lit: &LiteralNode) -> TypeAnnotationNode {
        match lit {
            LiteralNode::Integer(_) => TypeAnnotationNode::Integer,
            LiteralNode::Float(_) => TypeAnnotationNode::Custom("Float".to_string()),
            LiteralNode::String(_) => TypeAnnotationNode::String,
            LiteralNode::Boolean(_) => TypeAnnotationNode::Bool,
            LiteralNode::Did(_) => TypeAnnotationNode::Did,
            LiteralNode::Timestamp(_) => TypeAnnotationNode::Custom("Timestamp".to_string()),
        }
    }

    /// Check if a binary operation is valid and return the result type
    fn check_binary_op(
        &self,
        left: &TypeAnnotationNode,
        op: &BinaryOperator,
        right: &TypeAnnotationNode,
    ) -> Result<TypeAnnotationNode, CclError> {
        use BinaryOperator::*;
        use TypeAnnotationNode::*;

        match (left, op, right) {
            // Arithmetic operations
            (Integer, Add | Sub | Mul | Div, Integer) => Ok(Integer),
            (Mana, Add | Sub, Mana) => Ok(Mana),
            (Mana, Add | Sub, Integer) => Ok(Mana),
            (Integer, Add | Sub, Mana) => Ok(Mana),
            
            // Comparison operations
            (Integer, Eq | Neq | Lt | Lte | Gt | Gte, Integer) => Ok(Bool),
            (Mana, Eq | Neq | Lt | Lte | Gt | Gte, Mana) => Ok(Bool),
            (String, Eq | Neq, String) => Ok(Bool),
            (Bool, Eq | Neq, Bool) => Ok(Bool),
            
            // Logical operations
            (Bool, And | Or, Bool) => Ok(Bool),
            
            // String concatenation
            (String, Concat, String) => Ok(String),
            (String, Add, String) => Ok(String), // String + String for concatenation
            
            _ => Err(CclError::InvalidBinaryOperationError {
                left_type: left.clone(),
                operator: op.clone(),
                right_type: right.clone(),
            }),
        }
    }

    /// Check if a unary operation is valid and return the result type
    fn check_unary_op(
        &self,
        op: &UnaryOperator,
        operand: &TypeAnnotationNode,
    ) -> Result<TypeAnnotationNode, CclError> {
        use UnaryOperator::*;
        use TypeAnnotationNode::*;

        match (op, operand) {
            (Not, Bool) => Ok(Bool),
            (Neg, Integer) => Ok(Integer),
            (Neg, Mana) => Ok(Mana),
            _ => Err(CclError::InvalidUnaryOperationError {
                operator: op.clone(),
                operand_type: operand.clone(),
            }),
        }
    }

    fn enter_scope(&mut self) {
        self.current_scope_level += 1;
    }

    fn exit_scope(&mut self) {
        // Remove symbols from current scope
        self.symbol_table.retain(|_, symbol| symbol.scope_level < self.current_scope_level);
        self.current_scope_level -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOperator, ExpressionNode, LiteralNode};

    #[test]
    fn test_arithmetic_type_checking() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let expr = ExpressionNode::BinaryOp {
            left: Box::new(ExpressionNode::Literal(LiteralNode::Integer(5))),
            operator: BinaryOperator::Add,
            right: Box::new(ExpressionNode::Literal(LiteralNode::Integer(3))),
        };
        
        let result = analyzer.evaluate_expression(&expr).unwrap();
        assert_eq!(result, TypeAnnotationNode::Integer);
    }

    #[test]
    fn test_type_mismatch_error() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let expr = ExpressionNode::BinaryOp {
            left: Box::new(ExpressionNode::Literal(LiteralNode::Integer(5))),
            operator: BinaryOperator::Add,
            right: Box::new(ExpressionNode::Literal(LiteralNode::String("hello".to_string()))),
        };
        
        let result = analyzer.evaluate_expression(&expr);
        assert!(result.is_err());
        
        if let Err(CclError::InvalidBinaryOperationError { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected InvalidBinaryOperationError");
        }
    }

    #[test]
    fn test_function_type_checking() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let expr = ExpressionNode::FunctionCall {
            name: "get_mana".to_string(),
            args: vec![ExpressionNode::Literal(LiteralNode::Did("did:example:123".to_string()))],
        };
        
        let result = analyzer.evaluate_expression(&expr).unwrap();
        assert_eq!(result, TypeAnnotationNode::Mana);
    }
}
