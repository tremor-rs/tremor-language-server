use futures::future;
use jsonrpc_core::{BoxFuture, Result};
use serde_json::Value;
use std::fs;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, Printer};
use tremor_script::{errors, pos};

mod tremor; // tremor-script
mod trickle; // tremor-query

pub fn lookup(name: &str) -> Option<Box<dyn Backend>> {
    match name {
        "tremor" => Some(Box::new(tremor::TremorScript::default())),
        "trickle" => Some(Box::new(trickle::TremorQuery::default())),
        _ => None,
    }
}

pub trait Backend: Send + Sync {
    fn parse_err(&self, text: &str) -> Option<errors::Error>;

    fn get_diagnostics(&self, text: &str) -> Vec<Diagnostic> {
        file_dbg("get_diagnostics", text);

        let mut diagnostics = Vec::new();

        if let Some(e) = self.parse_err(text) {
            let range = match e.context() {
                (_, Some(pos::Range(start, end))) => Range {
                    start: self.to_lsp_position(start),
                    end: self.to_lsp_position(end),
                },
                _ => Range::default(),
            };
            diagnostics.push(Diagnostic {
                range,
                severity: None,
                code: None,
                source: None,
                message: format!("{:?}", &e.to_string()),
                related_information: None,
            });
        }

        diagnostics
    }

    fn to_lsp_position(&self, location: pos::Location) -> Position {
        // position in language server protocol is zero-based
        Position::new((location.line - 1) as u64, (location.column - 1) as u64)
    }
}

impl LanguageServer for dyn Backend {
    type ShutdownFuture = BoxFuture<()>;
    type SymbolFuture = BoxFuture<Option<Vec<SymbolInformation>>>;
    type ExecuteFuture = BoxFuture<Option<Value>>;
    type CompletionFuture = BoxFuture<Option<CompletionResponse>>;
    type HoverFuture = BoxFuture<Option<Hover>>;
    type HighlightFuture = BoxFuture<Option<Vec<DocumentHighlight>>>;

    fn initialize(&self, _: &Printer, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                code_action_provider: None,
                code_lens_provider: None, /*Some(CodeLensOptions {
                                              resolve_provider: None,
                                          }),*/
                color_provider: None,
                completion_provider: None,
                definition_provider: None,
                document_formatting_provider: None,
                document_highlight_provider: None,
                document_link_provider: None,
                document_on_type_formatting_provider: None,
                document_range_formatting_provider: None,
                document_symbol_provider: None,
                execute_command_provider: None,
                folding_range_provider: None,
                hover_provider: Some(true),
                implementation_provider: None,
                references_provider: None,
                rename_provider: None,
                signature_help_provider: None,
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::Full,
                )),
                type_definition_provider: None,
                workspace_symbol_provider: None,
                workspace: Some(WorkspaceCapability {
                    workspace_folders: Some(WorkspaceFolderCapability {
                        supported: Some(true),
                        change_notifications: Some(
                            WorkspaceFolderCapabilityChangeNotifications::Bool(true),
                        ),
                    }),
                }),
            },
        })
    }

    fn initialized(&self, printer: &Printer, _: InitializedParams) {
        file_dbg("initialized", "initialized");
        // TODO check this from clients
        //printer.show_message(MessageType::Info, "server initialized!");
        printer.log_message(MessageType::Info, "server initialized!");
    }

    fn shutdown(&self) -> Self::ShutdownFuture {
        file_dbg("shutdown", "shutdown");
        Box::new(future::ok(()))
    }

    fn symbol(&self, _: WorkspaceSymbolParams) -> Self::SymbolFuture {
        file_dbg("symbol", "symbol");
        Box::new(future::ok(None))
    }

    fn execute_command(&self, printer: &Printer, _: ExecuteCommandParams) -> Self::ExecuteFuture {
        file_dbg("execute", "execute");
        printer.log_message(MessageType::Info, "executing command!");
        Box::new(future::ok(None))
    }

    fn completion(&self, _: CompletionParams) -> Self::CompletionFuture {
        file_dbg("completion", "completion");
        Box::new(future::ok(None))
    }

    fn hover(&self, params: TextDocumentPositionParams) -> Self::HoverFuture {
        file_dbg("hover", "hover");
        // TODO remove. just for test right now
        let result = Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                params.position.character.to_string(),
            )),
            range: None,
        };
        Box::new(future::ok(Some(result)))
    }

    fn document_highlight(&self, _: TextDocumentPositionParams) -> Self::HighlightFuture {
        Box::new(future::ok(None))
    }

    fn did_open(&self, printer: &Printer, params: DidOpenTextDocumentParams) {
        file_dbg("didOpen", "didOpen");
        let uri = params.text_document.uri;
        if let Ok(path) = uri.to_file_path() {
            if let Ok(text) = fs::read_to_string(path) {
                printer.publish_diagnostics(uri, self.get_diagnostics(&text));
            }
        }
    }

    fn did_change(&self, printer: &Printer, params: DidChangeTextDocumentParams) {
        file_dbg("didChange", "didChange");
        printer.publish_diagnostics(
            params.text_document.uri,
            self.get_diagnostics(&params.content_changes[0].text),
        );
    }

    // TODO make this run and handle local state here
    fn did_close(&self, printer: &Printer, params: DidCloseTextDocumentParams) {
        file_dbg("didClose", "didClose");
        printer.publish_diagnostics(params.text_document.uri, vec![]);
    }
}

// TODO remove. just for test right now
pub fn file_dbg(name: &str, content: &str) {
    use std::fs::File;
    use std::io::Write;

    let path = format!("/tmp/tremor_{}", name);

    let mut output = File::create(path).unwrap();
    write!(output, "{}", content);
}
