use icn_ccl::{compile_ccl_source_to_wasm, CclError, StandardLibrary};
use serde_json::Value;
use std::collections::HashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub struct CclLanguageServer {
    client: Client,
    stdlib: StandardLibrary,
    documents: std::sync::RwLock<HashMap<Url, String>>,
}

impl CclLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            stdlib: StandardLibrary::new(),
            documents: std::sync::RwLock::new(HashMap::new()),
        }
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
                    trigger_characters: Some(vec![".".to_string(), "(".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("ccl".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            ..Default::default()
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
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        
        // Store document content
        if let Ok(mut docs) = self.documents.write() {
            docs.insert(uri.clone(), text.clone());
        }
        
        // Validate and send diagnostics
        self.validate_document(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;
            
            // Update stored document
            if let Ok(mut docs) = self.documents.write() {
                docs.insert(uri.clone(), text.clone());
            }
            
            // Re-validate
            self.validate_document(&uri, &text).await;
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        if let Ok(docs) = self.documents.read() {
            if let Some(text) = docs.get(uri) {
                let completions = crate::completion::get_completions(&self.stdlib, text, position);
                return Ok(Some(CompletionResponse::Array(completions)));
            }
        }
        
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        if let Ok(docs) = self.documents.read() {
            if let Some(text) = docs.get(uri) {
                return Ok(crate::hover::get_hover_info(&self.stdlib, text, position));
            }
        }
        
        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        if let Ok(docs) = self.documents.read() {
            if let Some(text) = docs.get(uri) {
                if let Some(location) = self.find_definition(text, position, uri.clone()) {
                    return Ok(Some(GotoDefinitionResponse::Scalar(location)));
                }
            }
        }
        
        Ok(None)
    }
}

impl CclLanguageServer {
    async fn validate_document(&self, uri: &Url, text: &str) {
        let diagnostics = crate::diagnostics::validate_ccl_source(text);
        
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
    
    fn find_definition(&self, text: &str, position: Position, current_uri: Url) -> Option<Location> {
        // Extract word at position
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }
        
        let line = lines[position.line as usize];
        let word = self.extract_word_at_position(line, position.character as usize)?;
        
        // Check if it's a stdlib function
        if self.stdlib.get_function(&word).is_some() {
            // For stdlib functions, we can't provide a real location,
            // but in a full implementation we'd have documentation locations
            return Some(Location {
                uri: current_uri.clone(),
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: word.len() as u32 },
                },
            });
        }
        
        // Look for function definitions in the current file
        for (line_num, line_text) in lines.iter().enumerate() {
            if line_text.contains(&format!("fn {}", word)) {
                if let Some(start_pos) = line_text.find(&word) {
                    return Some(Location {
                        uri: current_uri.clone(),
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: start_pos as u32,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: (start_pos + word.len()) as u32,
                            },
                        },
                    });
                }
            }
        }
        
        None
    }
    
    fn extract_word_at_position(&self, line: &str, character: usize) -> Option<String> {
        if character >= line.len() {
            return None;
        }
        
        let chars: Vec<char> = line.chars().collect();
        
        // Find word boundaries
        let mut start = character;
        let mut end = character;
        
        // Move start backward to find beginning of word
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        
        // Move end forward to find end of word
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        
        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }
}

pub async fn start_lsp_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| CclLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}