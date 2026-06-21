mod actions;
mod diagnostics;
mod document;
mod features;
mod workspace;

pub use features::completion_items;
pub use scaffold_docs as docs;

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    path::PathBuf,
    sync::Arc,
};

use document::Document;
use scaffold_docs::{DocIndex, WorkspaceDocIndex};
use tokio::sync::Mutex;
use tower_lsp::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result as LspResult,
    lsp_types::{
        CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability, CompletionOptions,
        CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
        DidChangeWatchedFilesParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DocumentFormattingParams, DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse,
        GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams, InitializeParams,
        InitializeResult, InitializedParams, InlayHint, InlayHintOptions, InlayHintParams,
        InlayHintServerCapabilities, Location, MessageType, OneOf, ReferenceParams,
        SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensParams,
        SemanticTokensResult, SemanticTokensServerCapabilities, ServerCapabilities, SignatureHelp,
        SignatureHelpOptions, SignatureHelpParams, SymbolInformation, TextDocumentSyncCapability,
        TextDocumentSyncKind, TextEdit, Url, WorkDoneProgressOptions, WorkspaceSymbolParams,
    },
};

pub fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(run_server());
    Ok(())
}

async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let documents = Arc::new(Mutex::new(HashMap::new()));
    let base_docs = DocIndex::scaffold();
    let workspace_docs = Arc::new(Mutex::new(WorkspaceDocIndex::empty()));
    let workspace_roots = Arc::new(Mutex::new(Vec::new()));
    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents,
        base_docs,
        workspace_docs,
        workspace_roots,
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, Document>>>,
    base_docs: DocIndex,
    workspace_docs: Arc<Mutex<WorkspaceDocIndex>>,
    workspace_roots: Arc<Mutex<Vec<PathBuf>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        let roots = workspace::workspace_roots(&params);
        let workspace_docs = workspace::workspace_doc_index_from_roots(&roots);
        *self.workspace_roots.lock().await = roots;
        *self.workspace_docs.lock().await = workspace_docs;
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                hover_provider: Some(tower_lsp::lsp_types::HoverProviderCapability::Simple(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec![" ".to_owned(), "(".to_owned()]),
                    retrigger_characters: Some(vec![" ".to_owned()]),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                            legend: features::semantic_tokens_legend(),
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                    ),
                ),
                document_formatting_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                inlay_hint_provider: Some(OneOf::Right(InlayHintServerCapabilities::Options(
                    InlayHintOptions {
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                        resolve_provider: Some(false),
                    },
                ))),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Scaffold language server initialized")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let document = Document::new(params.text_document.text);
        let _previous = self
            .documents
            .lock()
            .await
            .insert(uri.clone(), document.clone());
        self.publish_syntax_diagnostics(uri, &document).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let Some(change) = params.content_changes.into_iter().next() else {
            return;
        };
        let document = Document::new(change.text);
        let _previous = self
            .documents
            .lock()
            .await
            .insert(uri.clone(), document.clone());
        self.publish_syntax_diagnostics(uri, &document).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        let _previous = self.documents.lock().await.remove(&uri);
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        if !params
            .changes
            .iter()
            .any(|change| change.uri.path().ends_with(".scm"))
        {
            return;
        }
        self.reindex_workspace_docs().await;
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let documents = self.documents.lock().await;
        let index = if let Some(document) = documents.get(&uri) {
            self.index_for_document(uri.as_str(), document).await
        } else {
            self.index_without_document().await
        };
        Ok(Some(CompletionResponse::Array(completion_items(&index))))
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let documents = self.documents.lock().await;
        let Some(document) = documents.get(&uri) else {
            return Ok(None);
        };
        let index = self.index_for_document(uri.as_str(), document).await;
        Ok(document
            .word_at(position)
            .and_then(|symbol| features::hover_for_symbol(&index, &symbol)))
    }

    async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> LspResult<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let documents = self.documents.lock().await;
        let Some(document) = documents.get(&uri) else {
            return Ok(None);
        };
        let index = self.index_for_document(uri.as_str(), document).await;
        Ok(document.form_context_before(position).and_then(|context| {
            features::signature_help_for_symbol(&index, &context.head, context.active_argument)
        }))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> LspResult<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let documents = self.documents.lock().await;
        let Some(document) = documents.get(&uri) else {
            return Ok(None);
        };
        let index = self.index_for_document(uri.as_str(), document).await;
        Ok(Some(SemanticTokensResult::Tokens(
            features::semantic_tokens(&index, document.text()),
        )))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let documents = self.documents.lock().await;
        let Some(document) = documents.get(&uri) else {
            return Ok(None);
        };
        let Some(symbol) = document.word_at(position) else {
            return Ok(None);
        };
        let index = self.index_for_document(uri.as_str(), document).await;
        Ok(index
            .get(&symbol)
            .and_then(features::location)
            .map(GotoDefinitionResponse::Scalar))
    }

    async fn references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let documents = self.documents.lock().await;
        let Some(document) = documents.get(&uri) else {
            return Ok(None);
        };
        let Some(symbol) = document.word_at(position) else {
            return Ok(None);
        };

        let open_uris = documents.keys().cloned().collect::<HashSet<_>>();
        let mut reference_documents = documents
            .iter()
            .map(|(document_uri, document)| (document_uri.to_string(), document.text().to_owned()))
            .collect::<Vec<_>>();
        drop(documents);

        let roots = self.workspace_roots.lock().await.clone();
        for root in roots {
            for path in scaffold_context::workspace_scheme_paths(&root) {
                let Ok(file_uri) = Url::from_file_path(&path) else {
                    continue;
                };
                if open_uris.contains(&file_uri) {
                    continue;
                }
                let Ok(source) = std::fs::read_to_string(&path) else {
                    continue;
                };
                reference_documents.push((file_uri.to_string(), source));
            }
        }
        Ok(Some(reference_locations(reference_documents, &symbol)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let documents = self.documents.lock().await;
        let Some(document) = documents.get(&uri) else {
            return Ok(None);
        };
        let index = self.index_for_document(uri.as_str(), document).await;
        let symbols = index
            .entries_in_source(uri.as_str())
            .filter_map(|entry| {
                let range = features::lsp_range(entry.range?);
                #[allow(deprecated)]
                let symbol = DocumentSymbol {
                    name: entry.name.clone(),
                    detail: entry.signature.clone(),
                    kind: features::symbol_kind(entry),
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: None,
                };
                Some(symbol)
            })
            .collect::<Vec<_>>();
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> LspResult<Option<Vec<SymbolInformation>>> {
        let mut index = self.index_without_document().await;
        let documents = self.documents.lock().await;
        for (uri, document) in documents.iter() {
            index.extend_editor_source(uri.as_str(), document.text());
        }
        Ok(Some(features::workspace_symbols(&index, &params.query)))
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> LspResult<Option<Vec<CodeActionOrCommand>>> {
        let uri = params.text_document.uri;
        let documents = self.documents.lock().await;
        let Some(document) = documents.get(&uri) else {
            return Ok(None);
        };
        let actions = params
            .context
            .diagnostics
            .iter()
            .filter_map(|diagnostic| actions::missing_doc_action(&uri, document, diagnostic))
            .map(CodeActionOrCommand::CodeAction)
            .collect::<Vec<_>>();
        Ok(Some(actions))
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> LspResult<Option<Vec<InlayHint>>> {
        let uri = params.text_document.uri;
        let documents = self.documents.lock().await;
        let Some(document) = documents.get(&uri) else {
            return Ok(None);
        };
        let index = self.index_for_document(uri.as_str(), document).await;
        Ok(Some(features::inlay_hints(
            &index,
            document.text(),
            params.range,
        )))
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> LspResult<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let documents = self.documents.lock().await;
        let Some(document) = documents.get(&uri) else {
            return Ok(None);
        };
        let Ok(formatted) = scaffold_fmt::format_text(document.text()) else {
            return Ok(None);
        };
        if formatted == document.text() {
            return Ok(Some(Vec::new()));
        }
        Ok(Some(vec![TextEdit {
            range: document.full_range(),
            new_text: formatted,
        }]))
    }
}

impl Backend {
    async fn index_without_document(&self) -> DocIndex {
        let mut index = self.base_docs.clone();
        let workspace_docs = self.workspace_docs.lock().await;
        index.extend_index(workspace_docs.all());
        index
    }

    async fn index_for_document(&self, source_name: &str, document: &Document) -> DocIndex {
        let mut index = self.base_docs.clone();
        let workspace_docs = self.workspace_docs.lock().await;
        workspace_docs.extend_imported_docs(&mut index, document.text());
        index.extend_editor_source(source_name, document.text());
        index
    }

    async fn publish_syntax_diagnostics(&self, uri: Url, document: &Document) {
        let diagnostics = diagnostics::document_diagnostics(uri.as_str(), document.text());
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn reindex_workspace_docs(&self) {
        let roots = self.workspace_roots.lock().await.clone();
        let workspace_docs = workspace::workspace_doc_index_from_roots(&roots);
        *self.workspace_docs.lock().await = workspace_docs;
    }
}

fn reference_locations(documents: Vec<(String, String)>, symbol: &str) -> Vec<Location> {
    scaffold_editor::symbols::reference_locations(
        documents
            .iter()
            .map(|(uri, text)| (uri.as_str(), text.as_str())),
        symbol,
    )
    .into_iter()
    .filter_map(|location| {
        Some(Location::new(
            Url::parse(&location.uri).ok()?,
            tower_lsp::lsp_types::Range::new(
                tower_lsp::lsp_types::Position::new(location.line, location.start),
                tower_lsp::lsp_types::Position::new(
                    location.line,
                    location.start + location.length,
                ),
            ),
        ))
    })
    .collect()
}
