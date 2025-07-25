// icn-ccl/src/lsp/server.rs
//! Main LSP server implementation for CCL

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use url::Url;

use crate::{
    ast::AstNode,
    error::CclError,
    parser::{parse_ccl_source, CclParser},
    semantic_analyzer::SemanticAnalyzer,
};

use super::{completion, diagnostics, hover, navigation};

/// Document state stored for each open CCL file
#[derive(Debug, Clone)]
pub struct DocumentState {
    pub uri: Url,
    pub text: String,
    pub version: i32,
    pub ast: Option<AstNode>,
    pub semantic_errors: Vec<CclError>,
    pub parse_errors: Vec<CclError>,
}

/// Main CCL Language Server
pub struct CclLanguageServer {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, DocumentState>>>,
}

impl CclLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Parse and analyze a CCL document, updating its state
    async fn analyze_document(&self, uri: &Url, text: &str, version: i32) {
        let mut documents = self.documents.write().await;
        
        let mut doc_state = DocumentState {
            uri: uri.clone(),
            text: text.to_string(),
            version,
            ast: None,
            semantic_errors: Vec::new(),
            parse_errors: Vec::new(),
        };

        // Parse the document
        match parse_ccl_source(text) {
            Ok(ast) => {
                // Perform semantic analysis
                let mut analyzer = SemanticAnalyzer::new();
                match analyzer.analyze(&ast) {
                    Ok(()) => {
                        doc_state.ast = Some(ast);
                    }
                    Err(errors) => {
                        doc_state.ast = Some(ast);
                        doc_state.semantic_errors = errors;
                    }
                }
            }
            Err(parse_error) => {
                doc_state.parse_errors.push(parse_error);
            }
        }

        documents.insert(uri.clone(), doc_state.clone());

        // Send diagnostics to the client
        let diagnostics = diagnostics::generate_diagnostics(&doc_state);
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, Some(version))
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for CclLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "CCL Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        self.analyze_document(&uri, &text, version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        
        // Get the new text content (assuming full document sync)
        if let Some(change) = params.content_changes.first() {
            self.analyze_document(&uri, &change.text, version).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut documents = self.documents.write().await;
        documents.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        let documents = self.documents.read().await;
        if let Some(doc_state) = documents.get(uri) {
            let completions = completion::provide_completions(doc_state, position);
            return Ok(Some(CompletionResponse::Array(completions)));
        }
        
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let documents = self.documents.read().await;
        if let Some(doc_state) = documents.get(uri) {
            return Ok(hover::provide_hover(doc_state, position));
        }
        
        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let documents = self.documents.read().await;
        if let Some(doc_state) = documents.get(uri) {
            if let Some(location) = navigation::goto_definition(doc_state, position) {
                return Ok(Some(GotoDefinitionResponse::Scalar(location)));
            }
        }
        
        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        let documents = self.documents.read().await;
        if let Some(doc_state) = documents.get(uri) {
            let references = navigation::find_references(doc_state, position);
            if !references.is_empty() {
                return Ok(Some(references));
            }
        }
        
        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        
        let documents = self.documents.read().await;
        if let Some(doc_state) = documents.get(uri) {
            // For now, return no formatting changes
            // TODO: Implement CCL code formatting
            return Ok(Some(Vec::new()));
        }
        
        Ok(None)
    }
}