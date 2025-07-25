// icn-ccl/src/lsp_server.rs
//! Language Server Protocol (LSP) implementation for CCL (Cooperative Contract Language)
//!
//! This module provides IDE support for CCL including:
//! - Syntax highlighting
//! - Autocompletion
//! - Go-to-definition
//! - Inline documentation
//! - Error diagnostics

use crate::{parser, semantic_analyzer, stdlib::StdLibrary, CclError};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// CCL Language Server implementation
pub struct CclLanguageServer {
    client: Client,
    documents: Arc<DashMap<Url, TextDocumentItem>>,
    stdlib: Arc<StdLibrary>,
}

impl CclLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
            stdlib: Arc::new(StdLibrary::new()),
        }
    }

    /// Parse CCL document and return diagnostics
    async fn analyze_document(&self, _uri: &Url, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Parse the CCL source
        match parser::parse_ccl_source(text) {
            Ok(ast) => {
                // Run semantic analysis
                let mut analyzer = semantic_analyzer::SemanticAnalyzer::new();
                if let Err(errors) = analyzer.analyze(&ast) {
                    for error in errors {
                        let diagnostic = self.ccl_error_to_diagnostic(&error);
                        diagnostics.push(diagnostic);
                    }
                }
            }
            Err(error) => {
                let diagnostic = self.ccl_error_to_diagnostic(&error);
                diagnostics.push(diagnostic);
            }
        }

        diagnostics
    }

    /// Convert CCL error to LSP diagnostic
    fn ccl_error_to_diagnostic(&self, error: &CclError) -> Diagnostic {
        let message = match error {
            CclError::ParsingError(msg) => format!("Parse error: {}", msg),
            CclError::SemanticError(msg) => format!("Semantic error: {}", msg),
            CclError::CompilationError(msg) => format!("Compilation error: {}", msg),
            CclError::OptimizationError(msg) => format!("Optimization error: {}", msg),
            CclError::IoError(err) => format!("IO error: {}", err),
            CclError::SerializationError(msg) => format!("Serialization error: {}", msg),
            _ => format!("Error: {}", error),
        };

        Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("ccl".to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        }
    }

    /// Get completions for CCL keywords and stdlib functions
    async fn get_completions(&self, _params: &CompletionParams) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Add CCL keywords
        let keywords = [
            "struct", "fn", "if", "else", "while", "for", "let", "const", "return", "true",
            "false", "proposal", "vote", "delegate", "budget", "transfer", "mint", "burn",
        ];

        for keyword in &keywords {
            completions.push(CompletionItem {
                label: keyword.to_string(),
                label_details: None,
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("CCL keyword: {}", keyword)),
                documentation: None,
                deprecated: Some(false),
                preselect: Some(false),
                sort_text: Some(format!("0{}", keyword)),
                filter_text: Some(keyword.to_string()),
                insert_text: Some(keyword.to_string()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                insert_text_mode: None,
                text_edit: None,
                additional_text_edits: None,
                command: None,
                commit_characters: None,
                data: None,
                tags: None,
            });
        }

        // Add governance-specific functions
        let governance_functions = [
            ("create_proposal", "Create a new governance proposal"),
            ("cast_vote", "Cast a vote on a proposal"),
            ("delegate_to", "Delegate voting power to another member"),
            ("execute_proposal", "Execute an approved proposal"),
            ("get_proposal_status", "Get the status of a proposal"),
            ("get_member_voting_power", "Get voting power of a member"),
        ];

        for (func_name, description) in &governance_functions {
            completions.push(CompletionItem {
                label: func_name.to_string(),
                label_details: None,
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Governance function".to_string()),
                documentation: Some(Documentation::String(description.to_string())),
                deprecated: Some(false),
                preselect: Some(false),
                sort_text: Some(format!("1{}", func_name)),
                filter_text: Some(func_name.to_string()),
                insert_text: Some(format!("{}()", func_name)),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                insert_text_mode: None,
                text_edit: None,
                additional_text_edits: None,
                command: None,
                commit_characters: None,
                data: None,
                tags: None,
            });
        }

        // Add economic functions
        let economic_functions = [
            ("budget_allocate", "Allocate budget for a specific purpose"),
            ("dividend_distribute", "Distribute dividends to members"),
            ("token_mint", "Mint new tokens"),
            ("token_transfer", "Transfer tokens between accounts"),
            ("calculate_voting_weight", "Calculate quadratic voting weight"),
            ("get_account_balance", "Get account balance"),
        ];

        for (func_name, description) in &economic_functions {
            completions.push(CompletionItem {
                label: func_name.to_string(),
                label_details: None,
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Economic function".to_string()),
                documentation: Some(Documentation::String(description.to_string())),
                deprecated: Some(false),
                preselect: Some(false),
                sort_text: Some(format!("2{}", func_name)),
                filter_text: Some(func_name.to_string()),
                insert_text: Some(format!("{}()", func_name)),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                insert_text_mode: None,
                text_edit: None,
                additional_text_edits: None,
                command: None,
                commit_characters: None,
                data: None,
                tags: None,
            });
        }

        completions
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for CclLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "CCL Language Server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("ccl".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "CCL Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();

        // Store the document
        self.documents.insert(uri.clone(), params.text_document);

        // Analyze and send diagnostics
        let diagnostics = self.analyze_document(&uri, &text).await;
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        // Update document content
        if let Some(mut doc) = self.documents.get_mut(&uri) {
            for change in params.content_changes {
                doc.text = change.text;
            }
            let text = doc.text.clone();
            drop(doc);

            // Re-analyze and send updated diagnostics
            let diagnostics = self.analyze_document(&uri, &text).await;
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "CCL file saved!")
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = self.get_completions(&params).await;
        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        // Simple hover implementation - could be enhanced with symbol information
        let uri = &params.text_document_position_params.text_document.uri;

        if let Some(_doc) = self.documents.get(uri) {
            let hover_text = "CCL (Cooperative Contract Language) - Hover information available";
            return Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(hover_text.to_string())),
                range: None,
            }));
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        // Basic go-to-definition implementation
        // In a full implementation, this would analyze the AST and find symbol definitions
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // For now, return the current position (placeholder implementation)
        Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range: Range {
                start: position,
                end: position,
            },
        })))
    }
}

/// Start the CCL Language Server
pub async fn start_lsp_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| CclLanguageServer::new(client));

    Server::new(stdin, stdout, socket).serve(service).await;
}

/// Create a new CCL LSP service for embedding in other applications
pub fn create_lsp_service() -> (LspService<CclLanguageServer>, tower_lsp::ClientSocket) {
    LspService::new(|client| CclLanguageServer::new(client))
}