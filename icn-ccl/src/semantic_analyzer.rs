// icn-ccl/src/semantic_analyzer.rs
use crate::ast::{
    ActionNode, AstNode, BinaryOperator, BlockNode, ExpressionNode, StatementNode,
    TypeAnnotationNode, UnaryOperator, PolicyStatementNode,
};
use crate::error::CclError;
use std::collections::HashMap;

// Example symbol table entry
#[derive(Debug, Clone)]
pub enum Symbol {
    Variable {
        type_ann: crate::ast::TypeAnnotationNode,
    },
    Function {
        params: Vec<crate::ast::TypeAnnotationNode>,
        return_type: crate::ast::TypeAnnotationNode,
    },
    // ... other symbol types
}

pub struct SemanticAnalyzer {
    symbol_table_stack: Vec<HashMap<String, Symbol>>,
    current_return_type: Option<TypeAnnotationNode>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = SemanticAnalyzer {
            symbol_table_stack: vec![HashMap::new()], // Global scope
            current_return_type: None,
        };
        let _ = analyzer.insert_symbol(
            "host_get_reputation".to_string(),
            Symbol::Function {
                params: Vec::new(),
                return_type: TypeAnnotationNode::Integer,
            },
        );
        // Built-in helpers for array manipulation
        let any = TypeAnnotationNode::Custom("Any".to_string());
        let _ = analyzer.insert_symbol(
            "array_len".to_string(),
            Symbol::Function {
                params: vec![TypeAnnotationNode::Array(Box::new(any.clone()))],
                return_type: TypeAnnotationNode::Integer,
            },
        );
        let _ = analyzer.insert_symbol(
            "array_push".to_string(),
            Symbol::Function {
                params: vec![
                    TypeAnnotationNode::Array(Box::new(any.clone())),
                    any.clone(),
                ],
                return_type: TypeAnnotationNode::Integer,
            },
        );
        let _ = analyzer.insert_symbol(
            "array_pop".to_string(),
            Symbol::Function {
                params: vec![TypeAnnotationNode::Array(Box::new(any.clone()))],
                return_type: any,
            },
        );
        analyzer
    }

    pub fn analyze(&mut self, ast: &AstNode) -> Result<(), CclError> {
        self.visit_node(ast)?;
        Ok(())
    }

    fn push_scope(&mut self) {
        self.symbol_table_stack.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.symbol_table_stack.pop();
    }

    fn insert_symbol(&mut self, name: String, symbol: Symbol) -> Result<(), CclError> {
        let scope = self
            .symbol_table_stack
            .last_mut()
            .expect("scope stack should not be empty");
        if scope.contains_key(&name) {
            return Err(CclError::SemanticError(format!(
                "Symbol `{}` already defined in this scope",
                name
            )));
        }
        scope.insert(name, symbol);
        Ok(())
    }

    fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        for scope in self.symbol_table_stack.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }

    fn lookup_symbol_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for scope in self.symbol_table_stack.iter_mut().rev() {
            if let Some(sym) = scope.get_mut(name) {
                return Some(sym);
            }
        }
        None
    }

    fn visit_node(&mut self, node: &AstNode) -> Result<(), CclError> {
        match node {
            AstNode::Policy(statements) => {
                for stmt in statements {
                    self.visit_policy_statement(stmt)?;
                }
            }
            AstNode::FunctionDefinition {
                name,
                parameters,
                return_type,
                body,
            } => {
                // register function in global scope
                self.insert_symbol(
                    name.clone(),
                    Symbol::Function {
                        params: parameters.iter().map(|p| p.type_ann.clone()).collect(),
                        return_type: return_type.clone(),
                    },
                )?;

                let prev_return = self.current_return_type.clone();
                self.current_return_type = Some(return_type.clone());
                self.push_scope();
                for param in parameters {
                    self.insert_symbol(
                        param.name.clone(),
                        Symbol::Variable {
                            type_ann: param.type_ann.clone(),
                        },
                    )?;
                }
                let mut has_return = false;
                self.visit_block(body, &mut has_return)?;
                if !has_return {
                    return Err(CclError::SemanticError(format!(
                        "Function `{}` is missing return statement",
                        name
                    )));
                }
                self.pop_scope();
                self.current_return_type = prev_return;
            }
            AstNode::StructDefinition { name, fields } => {
                self.insert_symbol(
                    name.clone(),
                    Symbol::Variable {
                        type_ann: TypeAnnotationNode::Custom("struct".to_string()),
                    },
                )?;
                self.push_scope();
                for field in fields {
                    self.insert_symbol(
                        field.name.clone(),
                        Symbol::Variable {
                            type_ann: field.type_ann.clone(),
                        },
                    )?;
                }
                self.pop_scope();
            }
            AstNode::RuleDefinition {
                name,
                condition,
                action,
            } => {
                let cond_ty = self.evaluate_expression(condition)?;
                if cond_ty != TypeAnnotationNode::Bool {
                    return Err(CclError::TypeError(format!(
                        "Condition of rule `{}` must be Bool",
                        name
                    )));
                }
                self.visit_action(action)?;
            }
            AstNode::Block(block) => {
                let mut _has_ret = false;
                self.visit_block(block, &mut _has_ret)?;
            }
        }
        Ok(())
    }

    fn visit_policy_statement(
        &mut self,
        stmt: &crate::ast::PolicyStatementNode,
    ) -> Result<(), CclError> {
        match stmt {
            crate::ast::PolicyStatementNode::FunctionDef(func_def_node) => {
                self.visit_node(func_def_node)?;
            }
            crate::ast::PolicyStatementNode::RuleDef(rule_def_node) => {
                self.visit_node(rule_def_node)?;
            }
            crate::ast::PolicyStatementNode::Import { path, alias } => {
                // For now, simply record the alias as a custom type symbol so later references
                // to the imported name pass basic semantic checks. A full implementation would
                // load and analyze the referenced module.
                self.insert_symbol(
                    alias.clone(),
                    Symbol::Variable {
                        type_ann: TypeAnnotationNode::Custom(format!("import<{path}>",)),
                    },
                )?
            }
            crate::ast::PolicyStatementNode::StructDef(def) => {
                self.visit_node(def)?;
            }
            PolicyStatementNode::ConstDef { name, value, type_ann } => {
                let expr_type = self.evaluate_expression(value)?;
                if !expr_type.compatible_with(type_ann) {
                    return Err(CclError::TypeError(format!(
                        "Constant {} type mismatch: expected {:?}, found {:?}",
                        name, type_ann, expr_type
                    )));
                }
                self.insert_symbol(
                    name.clone(),
                    Symbol::Variable {
                        type_ann: type_ann.clone(),
                    },
                )?;
            }
            PolicyStatementNode::MacroDef { name, .. } => {
                // Macros are compile-time constructs, just register the name
                self.insert_symbol(
                    name.clone(),
                    Symbol::Variable {
                        type_ann: TypeAnnotationNode::Custom("Macro".to_string()),
                    },
                )?;
            }
            // Handle governance DSL statements
            PolicyStatementNode::EventDef { name, fields } => {
                // Register event definition
                self.insert_symbol(
                    name.clone(),
                    Symbol::Variable {
                        type_ann: TypeAnnotationNode::Custom("Event".to_string()),
                    },
                )?;
                // Validate field types - basic validation
                for (_field_name, _field_type) in fields {
                    // TODO: Add proper type validation
                }
            }
            PolicyStatementNode::StateDef { name, type_ann, initial_value } => {
                // Register state variable
                self.insert_symbol(
                    name.clone(),
                    Symbol::Variable {
                        type_ann: type_ann.clone(),
                    },
                )?;
                // Validate initial value if present
                if let Some(init_value) = initial_value {
                    let _value_type = self.evaluate_expression(init_value)?;
                    // TODO: Add type compatibility check
                }
            }
            PolicyStatementNode::TriggerDef { name, condition, action } => {
                // Register trigger
                self.insert_symbol(
                    name.clone(),
                    Symbol::Variable {
                        type_ann: TypeAnnotationNode::Custom("Trigger".to_string()),
                    },
                )?;
                // Validate condition and action
                let _condition_type = self.evaluate_expression(condition)?;
                self.evaluate_expression(action)?;
            }
        }
        Ok(())
    }

    fn visit_block(&mut self, block: &BlockNode, found_return: &mut bool) -> Result<(), CclError> {
        self.push_scope();
        for stmt in &block.statements {
            self.visit_statement(stmt, found_return)?;
        }
        self.pop_scope();
        Ok(())
    }

    fn visit_statement(
        &mut self,
        stmt: &StatementNode,
        found_return: &mut bool,
    ) -> Result<(), CclError> {
        match stmt {
            StatementNode::Let { name, value } => {
                let ty = self.evaluate_expression(value)?;
                if let Some(Symbol::Variable { type_ann }) = self.lookup_symbol_mut(name) {
                    if !ty.compatible_with(type_ann) {
                        return Err(CclError::TypeError(format!(
                            "Assignment type mismatch: expected {:?}, got {:?}",
                            type_ann, ty
                        )));
                    }
                    *type_ann = ty;
                } else {
                    self.insert_symbol(name.clone(), Symbol::Variable { type_ann: ty })?;
                }
            }
            StatementNode::ExpressionStatement(expr) => {
                self.evaluate_expression(expr)?;
            }
            StatementNode::Return(expr) => {
                let expr_ty = self.evaluate_expression(expr)?;
                let expected = self.current_return_type.clone().ok_or_else(|| {
                    CclError::InternalCompilerError("Return outside function".to_string())
                })?;
                if !expr_ty.compatible_with(&expected) {
                    return Err(CclError::TypeError(format!(
                        "Return type mismatch: expected {:?}, got {:?}",
                        expected, expr_ty
                    )));
                }
                *found_return = true;
            }
            StatementNode::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond_ty = self.evaluate_expression(condition)?;
                if cond_ty != TypeAnnotationNode::Bool {
                    return Err(CclError::TypeError("If condition must be Bool".to_string()));
                }
                self.visit_block(then_block, found_return)?;
                if let Some(b) = else_block {
                    self.visit_block(b, found_return)?;
                }
            }
            StatementNode::WhileLoop { condition, body } => {
                let cond_ty = self.evaluate_expression(condition)?;
                if cond_ty != TypeAnnotationNode::Bool {
                    return Err(CclError::TypeError(
                        "While condition must be Bool".to_string(),
                    ));
                }
                self.visit_block(body, found_return)?;
            }
            StatementNode::ForLoop {
                iterator,
                iterable,
                body,
            } => {
                let iter_ty = self.evaluate_expression(iterable)?;
                let elem_ty = match iter_ty {
                    TypeAnnotationNode::Array(inner) => *inner,
                    _ => {
                        return Err(CclError::TypeError(
                            "For loop iterable must be an Array".to_string(),
                        ))
                    }
                };
                self.push_scope();
                self.insert_symbol(iterator.clone(), Symbol::Variable { type_ann: elem_ty })?;
                self.visit_block(body, found_return)?;
                self.pop_scope();
            }
            StatementNode::Break | StatementNode::Continue => {
                // Validity checked during parsing; nothing else to do
            }
        }
        Ok(())
    }

    fn visit_action(&mut self, action: &ActionNode) -> Result<(), CclError> {
        match action {
            ActionNode::Allow | ActionNode::Deny => Ok(()),
            ActionNode::Charge(expr) => {
                let ty = self.evaluate_expression(expr)?;
                if ty != TypeAnnotationNode::Integer && ty != TypeAnnotationNode::Mana {
                    return Err(CclError::TypeError(
                        "Charge amount must be Integer or Mana".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }

    fn evaluate_expression(
        &mut self,
        expr: &ExpressionNode,
    ) -> Result<TypeAnnotationNode, CclError> {
        match expr {
            ExpressionNode::IntegerLiteral(_) => Ok(TypeAnnotationNode::Integer),
            ExpressionNode::BooleanLiteral(_) => Ok(TypeAnnotationNode::Bool),
            ExpressionNode::StringLiteral(_) => Ok(TypeAnnotationNode::String),
            ExpressionNode::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    return Err(CclError::SemanticError(
                        "Empty arrays are not supported".to_string(),
                    ));
                }

                let first_type = self.evaluate_expression(&elements[0])?;
                for element in elements.iter().skip(1) {
                    let elem_type = self.evaluate_expression(element)?;
                    if !elem_type.compatible_with(&first_type) {
                        return Err(CclError::TypeError(format!(
                            "Array elements must be of same type: expected {:?}, got {:?}",
                            first_type, elem_type
                        )));
                    }
                }
                Ok(TypeAnnotationNode::Array(Box::new(first_type)))
            }
            ExpressionNode::ArrayAccess { array, index } => {
                let array_type = self.evaluate_expression(array)?;
                let index_type = self.evaluate_expression(index)?;

                if !index_type.is_numeric() {
                    return Err(CclError::TypeError(
                        "Array index must be numeric".to_string(),
                    ));
                }

                match array_type {
                    TypeAnnotationNode::Array(element_type) => Ok(*element_type),
                    _ => Err(CclError::TypeError(format!(
                        "Cannot index into non-array type: {:?}",
                        array_type
                    ))),
                }
            }
            ExpressionNode::SomeExpr(inner) => {
                let inner_type = self.evaluate_expression(inner)?;
                Ok(TypeAnnotationNode::Option(Box::new(inner_type)))
            }
            ExpressionNode::NoneExpr => Ok(TypeAnnotationNode::Option(Box::new(TypeAnnotationNode::Custom("Unknown".to_string())))),
            ExpressionNode::OkExpr(inner) => {
                let inner_type = self.evaluate_expression(inner)?;
                Ok(TypeAnnotationNode::Result {
                    ok_type: Box::new(inner_type),
                    err_type: Box::new(TypeAnnotationNode::String),
                })
            }
            ExpressionNode::ErrExpr(inner) => {
                let inner_type = self.evaluate_expression(inner)?;
                Ok(TypeAnnotationNode::Result {
                    ok_type: Box::new(TypeAnnotationNode::Custom("Unknown".to_string())),
                    err_type: Box::new(inner_type),
                })
            }
            ExpressionNode::RequireProof(inner) => {
                let ty = self.evaluate_expression(inner)?;
                if ty != TypeAnnotationNode::String {
                    return Err(CclError::TypeError(
                        "require_proof expects String".to_string(),
                    ));
                }
                Ok(TypeAnnotationNode::Bool)
            }
            ExpressionNode::Match { value, arms } => {
                let _ = self.evaluate_expression(value)?;
                let mut branch_ty: Option<TypeAnnotationNode> = None;
                for (_, expr) in arms {
                    let t = self.evaluate_expression(expr)?;
                    if let Some(existing) = &branch_ty {
                        if !existing.compatible_with(&t) {
                            return Err(CclError::TypeError("Match arm type mismatch".to_string()));
                        }
                    } else {
                        branch_ty = Some(t);
                    }
                }
                Ok(branch_ty.unwrap_or(TypeAnnotationNode::Integer))
            }
            ExpressionNode::TryExpr { expr, catch_arm } => {
                let expr_type = self.evaluate_expression(expr)?;
                if let Some(catch_expr) = catch_arm {
                    let catch_type = self.evaluate_expression(catch_expr)?;
                    // For now, return the unified type (or the main expression type)
                    if expr_type.compatible_with(&catch_type) {
                        Ok(expr_type)
                    } else {
                        Ok(expr_type)  // Prefer the main expression type
                    }
                } else {
                    Ok(expr_type)
                }
            }
            ExpressionNode::Identifier(name) => match self.lookup_symbol(name) {
                Some(Symbol::Variable { type_ann }) => Ok(type_ann.clone()),
                Some(Symbol::Function { .. }) => Err(CclError::TypeError(format!(
                    "Function `{}` used without call",
                    name
                ))),
                None => Err(CclError::SemanticError(format!(
                    "Undefined identifier `{}`",
                    name
                ))),
            },
            ExpressionNode::FunctionCall { name, arguments } => match name.as_str() {
                "array_len" => {
                    if arguments.len() != 1 {
                        return Err(CclError::TypeError("array_len expects one argument".into()));
                    }
                    let arr_ty = self.evaluate_expression(&arguments[0])?;
                    match arr_ty {
                        TypeAnnotationNode::Array(_) => Ok(TypeAnnotationNode::Integer),
                        _ => Err(CclError::TypeError("array_len requires array".into())),
                    }
                }
                "array_push" => {
                    if arguments.len() != 2 {
                        return Err(CclError::TypeError(
                            "array_push expects two arguments".into(),
                        ));
                    }
                    let arr_ty = self.evaluate_expression(&arguments[0])?;
                    let val_ty = self.evaluate_expression(&arguments[1])?;
                    match arr_ty {
                        TypeAnnotationNode::Array(elem_ty) => {
                            if !val_ty.compatible_with(&elem_ty) {
                                Err(CclError::TypeError("push type mismatch".into()))
                            } else {
                                Ok(TypeAnnotationNode::Integer)
                            }
                        }
                        _ => Err(CclError::TypeError("array_push requires array".into())),
                    }
                }
                "array_pop" => {
                    if arguments.len() != 1 {
                        return Err(CclError::TypeError("array_pop expects one argument".into()));
                    }
                    let arr_ty = self.evaluate_expression(&arguments[0])?;
                    match arr_ty {
                        TypeAnnotationNode::Array(elem_ty) => Ok(*elem_ty),
                        _ => Err(CclError::TypeError("array_pop requires array".into())),
                    }
                }
                _ => {
                    let symbol = self.lookup_symbol(name).cloned();
                    match symbol {
                        Some(Symbol::Function {
                            params,
                            return_type,
                        }) => {
                            if params.len() != arguments.len() {
                                return Err(CclError::TypeError(format!(
                                    "Function `{}` expects {} arguments, got {}",
                                    name,
                                    params.len(),
                                    arguments.len()
                                )));
                            }
                            for (arg_expr, param_ty) in arguments.iter().zip(params.iter()) {
                                let arg_ty = self.evaluate_expression(arg_expr)?;
                                if !arg_ty.compatible_with(param_ty) {
                                    return Err(CclError::TypeError(format!(
                                        "Argument type mismatch for `{}`: expected {:?}, got {:?}",
                                        name, param_ty, arg_ty
                                    )));
                                }
                            }
                            Ok(return_type.clone())
                        }
                        Some(Symbol::Variable { .. }) => Err(CclError::TypeError(format!(
                            "Variable `{}` used as function",
                            name
                        ))),
                        None => Err(CclError::SemanticError(format!(
                            "Undefined function `{}`",
                            name
                        ))),
                    }
                }
            },
            ExpressionNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                let l = self.evaluate_expression(left)?;
                let r = self.evaluate_expression(right)?;
                match operator {
                    BinaryOperator::Add => {
                        if l.is_numeric() && r.is_numeric() {
                            Ok(TypeAnnotationNode::Integer)
                        } else if l == TypeAnnotationNode::String && r == TypeAnnotationNode::String
                        {
                            Ok(TypeAnnotationNode::String) // String concatenation
                        } else {
                            Err(CclError::TypeError(
                                "Addition requires Integer operands or String concatenation"
                                    .to_string(),
                            ))
                        }
                    }
                    BinaryOperator::Sub | BinaryOperator::Mul | BinaryOperator::Div => {
                        if l.is_numeric() && r.is_numeric() {
                            Ok(TypeAnnotationNode::Integer)
                        } else {
                            Err(CclError::TypeError(
                                "Arithmetic operations require Integer operands".to_string(),
                            ))
                        }
                    }
                    BinaryOperator::Concat => {
                        if l == TypeAnnotationNode::String && r == TypeAnnotationNode::String {
                            Ok(TypeAnnotationNode::String)
                        } else {
                            Err(CclError::TypeError(
                                "String concatenation requires String operands".to_string(),
                            ))
                        }
                    }
                    BinaryOperator::Eq | BinaryOperator::Neq => {
                        if l.compatible_with(&r) {
                            Ok(TypeAnnotationNode::Bool)
                        } else {
                            Err(CclError::TypeError(
                                "Equality operands must be of same type".to_string(),
                            ))
                        }
                    }
                    BinaryOperator::Lt
                    | BinaryOperator::Gt
                    | BinaryOperator::Lte
                    | BinaryOperator::Gte => {
                        if l.is_numeric() && r.is_numeric() {
                            Ok(TypeAnnotationNode::Bool)
                        } else {
                            Err(CclError::TypeError(
                                "Comparison operators require Integer operands".to_string(),
                            ))
                        }
                    }
                    BinaryOperator::And | BinaryOperator::Or => {
                        if l == TypeAnnotationNode::Bool && r == TypeAnnotationNode::Bool {
                            Ok(TypeAnnotationNode::Bool)
                        } else {
                            Err(CclError::TypeError(
                                "Logical operators require Bool operands".to_string(),
                            ))
                        }
                    }
                }
            }
            ExpressionNode::MapLiteral(entries) => {
                if entries.is_empty() {
                    // For empty maps, we need explicit type annotation from context
                    return Err(CclError::TypeError("Empty map literal requires type annotation".to_string()));
                }
                
                // Infer key and value types from first entry
                let (first_key, first_value) = &entries[0];
                let key_type = self.evaluate_expression(first_key)?;
                let value_type = self.evaluate_expression(first_value)?;
                
                // Verify all entries have compatible types
                for (key, value) in entries.iter().skip(1) {
                    let k_type = self.evaluate_expression(key)?;
                    let v_type = self.evaluate_expression(value)?;
                    
                    if !key_type.compatible_with(&k_type) {
                        return Err(CclError::TypeError("Inconsistent key types in map literal".to_string()));
                    }
                    if !value_type.compatible_with(&v_type) {
                        return Err(CclError::TypeError("Inconsistent value types in map literal".to_string()));
                    }
                }
                
                Ok(TypeAnnotationNode::Map {
                    key_type: Box::new(key_type),
                    value_type: Box::new(value_type),
                })
            }
            ExpressionNode::MapAccess { map, key } => {
                let map_type = self.evaluate_expression(map)?;
                let key_type = self.evaluate_expression(key)?;
                
                match map_type {
                    TypeAnnotationNode::Map { key_type: expected_key, value_type } => {
                        if key_type.compatible_with(&expected_key) {
                            Ok(*value_type)
                        } else {
                            Err(CclError::TypeError("Map key type mismatch".to_string()))
                        }
                    }
                    _ => Err(CclError::TypeError("Map access on non-map type".to_string()))
                }
            }
            ExpressionNode::PanicExpr { message } => {
                let msg_type = self.evaluate_expression(message)?;
                if msg_type == TypeAnnotationNode::String {
                    // Panic never returns, but for type checking purposes we use a never type
                    // For simplicity, we'll just return unit type
                    Ok(TypeAnnotationNode::Custom("never".to_string()))
                } else {
                    Err(CclError::TypeError("Panic message must be a string".to_string()))
                }
            }
            // Handle governance DSL expressions
            ExpressionNode::EventEmit { event_name, fields } => {
                // Check if event is defined - simplified check
                if self.lookup_symbol(event_name).is_none() {
                    return Err(CclError::TypeError(format!("Undefined event: {}", event_name)));
                }
                // Type check field values
                for (_, field_expr) in fields {
                    self.evaluate_expression(field_expr)?;
                }
                Ok(TypeAnnotationNode::Custom("Unit".to_string()))
            }
            ExpressionNode::StateRead { state_name } => {
                // Check if state variable exists and return its type
                if let Some(symbol) = self.lookup_symbol(state_name) {
                    if let Symbol::Variable { type_ann } = symbol {
                        Ok(type_ann.clone())
                    } else {
                        Err(CclError::TypeError(format!("{} is not a state variable", state_name)))
                    }
                } else {
                    Err(CclError::TypeError(format!("Undefined state variable: {}", state_name)))
                }
            }
            ExpressionNode::StateWrite { state_name, value } => {
                // Check if state variable exists
                if let Some(symbol) = self.lookup_symbol(state_name) {
                    if let Symbol::Variable { type_ann: _type_ann } = symbol {
                        let _value_type = self.evaluate_expression(value)?;
                        // TODO: Add type compatibility check
                        Ok(TypeAnnotationNode::Custom("Unit".to_string()))
                    } else {
                        Err(CclError::TypeError(format!("{} is not a state variable", state_name)))
                    }
                } else {
                    Err(CclError::TypeError(format!("Undefined state variable: {}", state_name)))
                }
            }
            ExpressionNode::TriggerAction { trigger_name, params } => {
                // Check if trigger exists
                if self.lookup_symbol(trigger_name).is_none() {
                    return Err(CclError::TypeError(format!("Undefined trigger: {}", trigger_name)));
                }
                // Type check parameters
                for param in params {
                    self.evaluate_expression(param)?;
                }
                Ok(TypeAnnotationNode::Custom("Unit".to_string()))
            }
            ExpressionNode::CrossContractCall { contract_address, function_name: _, params } => {
                // Validate contract address
                let addr_type = self.evaluate_expression(contract_address)?;
                if !matches!(addr_type, TypeAnnotationNode::String | TypeAnnotationNode::Did) {
                    return Err(CclError::TypeError("Contract address must be string or DID".to_string()));
                }
                // Type check parameters
                for param in params {
                    self.evaluate_expression(param)?;
                }
                // Cross-contract calls can return any type, use generic for now
                Ok(TypeAnnotationNode::Custom("Any".to_string()))
            }
            ExpressionNode::BreakExpr => {
                Ok(TypeAnnotationNode::Custom("never".to_string()))
            }
            ExpressionNode::ContinueExpr => {
                Ok(TypeAnnotationNode::Custom("never".to_string()))
            }
            ExpressionNode::UnaryOp { operator, operand } => {
                let operand_ty = self.evaluate_expression(operand)?;
                match operator {
                    UnaryOperator::Not => {
                        if operand_ty == TypeAnnotationNode::Bool {
                            Ok(TypeAnnotationNode::Bool)
                        } else {
                            Err(CclError::TypeError(
                                "Logical negation requires Bool operand".to_string(),
                            ))
                        }
                    }
                    UnaryOperator::Neg => {
                        if operand_ty.is_numeric() {
                            Ok(operand_ty) // Preserve the exact numeric type (Integer or Mana)
                        } else {
                            Err(CclError::TypeError(
                                "Arithmetic negation requires numeric operand".to_string(),
                            ))
                        }
                    }
                }
            }
        }
    }
}
