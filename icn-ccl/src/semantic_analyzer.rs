// icn-ccl/src/semantic_analyzer.rs
use crate::ast::{
    ActionNode, AstNode, BinaryOperator, BlockNode, ExpressionNode, StatementNode,
    TypeAnnotationNode,
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
        SemanticAnalyzer {
            symbol_table_stack: vec![HashMap::new()], // Global scope
            current_return_type: None,
        }
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
                self.insert_symbol(name.clone(), Symbol::Variable { type_ann: ty })?;
            }
            StatementNode::ExpressionStatement(expr) => {
                self.evaluate_expression(expr)?;
            }
            StatementNode::Return(expr) => {
                let expr_ty = self.evaluate_expression(expr)?;
                let expected = self.current_return_type.clone().ok_or_else(|| {
                    CclError::InternalCompilerError("Return outside function".to_string())
                })?;
                if expr_ty != expected {
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
            ExpressionNode::FunctionCall { name, arguments } => {
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
                            if &arg_ty != param_ty {
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
            ExpressionNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                let l = self.evaluate_expression(left)?;
                let r = self.evaluate_expression(right)?;
                match operator {
                    BinaryOperator::Add
                    | BinaryOperator::Sub
                    | BinaryOperator::Mul
                    | BinaryOperator::Div => {
                        if l == TypeAnnotationNode::Integer && r == TypeAnnotationNode::Integer {
                            Ok(TypeAnnotationNode::Integer)
                        } else {
                            Err(CclError::TypeError(
                                "Arithmetic operations require Integer operands".to_string(),
                            ))
                        }
                    }
                    BinaryOperator::Eq | BinaryOperator::Neq => {
                        if l == r {
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
                        if l == TypeAnnotationNode::Integer && r == TypeAnnotationNode::Integer {
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
        }
    }
}
