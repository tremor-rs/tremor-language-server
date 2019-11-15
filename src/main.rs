use futures::future;
use jsonrpc_core::{BoxFuture, Result};
use serde_json::Value;
use std::fs;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService, Printer, Server};

#[derive(Debug, Default)]
struct Backend;

impl Backend {
    fn run_checks(&self, text: &str) -> Vec<Diagnostic> {
        file_dbg("run_checks", text);
        let mut diagnostics = Vec::new();
        // TODO add real tremor script errors as diagnostics
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 0),
            },
            severity: None,
            code: None,
            source: None,
            message: format!("{:?}", "testing testing"),
            related_information: None,
        });
        diagnostics
    }
}

impl LanguageServer for Backend {
    type ShutdownFuture = BoxFuture<()>;
    type SymbolFuture = BoxFuture<Option<Vec<SymbolInformation>>>;
    type ExecuteFuture = BoxFuture<Option<Value>>;
    type CompletionFuture = BoxFuture<Option<CompletionResponse>>;
    type HoverFuture = BoxFuture<Option<Hover>>;
    type HighlightFuture = BoxFuture<Option<Vec<DocumentHighlight>>>;

    fn initialize(&self, _: &Printer, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(true),
                ..ServerCapabilities::default()
            },
        })
    }

    fn initialized(&self, printer: &Printer, _: InitializedParams) {
        file_dbg("initialized", "initialized");
        // TODO see this from clients
        //printer.show_message(MessageType::Info, "server initialized!");
        printer.log_message(MessageType::Info, "server initialized!");
    }

    fn shutdown(&self) -> Self::ShutdownFuture {
        Box::new(future::ok(()))
    }

    fn symbol(&self, _: WorkspaceSymbolParams) -> Self::SymbolFuture {
        Box::new(future::ok(None))
    }

    fn execute_command(&self, _: &Printer, _: ExecuteCommandParams) -> Self::ExecuteFuture {
        Box::new(future::ok(None))
    }

    fn completion(&self, _: CompletionParams) -> Self::CompletionFuture {
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
                printer.publish_diagnostics(uri, self.run_checks(&text));
            }
        }
    }

    // TODO make this run
    fn did_change(&self, printer: &Printer, params: DidChangeTextDocumentParams) {
        file_dbg("didChange", "didChange");
        printer.publish_diagnostics(
            params.text_document.uri,
            self.run_checks(&params.content_changes[0].text),
        );
    }

    // TODO make this run
    fn did_close(&self, printer: &Printer, params: DidCloseTextDocumentParams) {
        file_dbg("didClose", "didClose");
        // TODO handle local state
        printer.publish_diagnostics(params.text_document.uri, vec![]);
    }
}

// TODO remove. just for test right now
fn file_dbg(name: &str, content: &str) {
    use std::fs::File;
    use std::io::Write;

    let path = format!("/tmp/tremor/{}", name);

    let mut output = File::create(path).unwrap();
    write!(output, "{}", content);
}

fn main() {
    file_dbg("main", "Hello world!");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, messages) = LspService::new(Backend::default());
    let handle = service.close_handle();
    let server = Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service);

    tokio::run(handle.run_until_exit(server));
}
