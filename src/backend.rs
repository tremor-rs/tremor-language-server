// Copyright 2018-2020, Wayfair GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::language;
use futures::future;
use halfbrown::HashMap;
use jsonrpc_core::{BoxFuture, Result};
use serde_json::Value;
use std::fs;
use std::sync::Mutex;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, Printer};

#[derive(Debug, Default)]
struct DocumentState {
    text: String,
    // TODO more fields here based on ast
}
type State = HashMap<Url, DocumentState>;

pub struct Backend {
    language: Box<dyn language::Language>,
    state: Mutex<State>,
}

impl Backend {
    pub fn new(language: Box<dyn language::Language>) -> Self {
        Self {
            language,
            state: Mutex::new(State::new()),
        }
    }

    fn update(&self, uri: Url, text: &str) {
        // TODO implement update as well. also remove unwrap
        self.state.lock().unwrap().insert(
            uri,
            DocumentState {
                text: text.to_string(),
            },
        );
    }

    // LSP helper functions

    fn get_diagnostics(&self, text: &str) -> Vec<Diagnostic> {
        file_dbg("get_diagnostics", text);

        let mut diagnostics = Vec::new();

        if let Some(e) = self.language.parse_err(text) {
            let range = match e.context() {
                (_, Some(language::pos::Range(start, end))) => Range {
                    start: self.to_lsp_position(start),
                    end: self.to_lsp_position(end),
                },
                _ => Range::default(),
            };

            let mut message = e.to_string();
            if let Some(hint) = e.hint() {
                // comma here splits the message into multiple lines
                message = format!("{}, Note: {}", message, hint);
            }

            diagnostics.push(Diagnostic {
                range,
                message,
                severity: Some(DiagnosticSeverity::Error),
                source: Some("tremor-language-server".to_string()),
                code: None,
                related_information: None,
            });
        }

        diagnostics
    }

    fn get_completions(&self, text: &str, position: Position) -> Vec<CompletionItem> {
        file_dbg("get_completions_text", text);
        file_dbg(
            "get_completions_position",
            &format!("{}:{}", position.line, position.character),
        );

        if let Some(partial_line) = self.get_partial_line(text, position) {
            file_dbg("get_completions_partial_line", &partial_line);

            // TODO support partial completions too, after ::
            if partial_line.ends_with("::") {
                // TODO more efficient. also save as constant and take len
                if let Some(module_name) = &partial_line[..partial_line.len() - 2]
                    .split_ascii_whitespace()
                    .last()
                {
                    file_dbg("get_completions_module_name", module_name);

                    return self
                        .language
                        .functions(module_name)
                        .iter()
                        .map(|func| {
                            //let insert_text = format!("{}($1)", func);
                            let insert_text = format!("{}()", func);
                            CompletionItem {
                                label: func.to_string(),
                                kind: Some(CompletionItemKind::Function),
                                insert_text: Some(insert_text),
                                insert_text_format: Some(InsertTextFormat::Snippet),
                                ..CompletionItem::default()
                            }
                        })
                        .collect();
                }
            }
        }

        vec![]
    }

    // utility functions
    // TODO move to utils module?

    fn to_lsp_position(&self, location: language::pos::Location) -> Position {
        // position in language server protocol is zero-based
        Position::new((location.line - 1) as u64, (location.column - 1) as u64)
    }

    fn get_partial_line(&self, text: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = text.split("\n").collect();
        if let Some(line) = lines.get(position.line as usize) {
            // TODO index check here
            Some((&line[..position.character as usize]).to_string())
        } else {
            None
        }
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
                code_action_provider: None,
                code_lens_provider: None, /*Some(CodeLensOptions {
                                              resolve_provider: None,
                                          }),*/
                color_provider: None,
                completion_provider: Some(CompletionOptions {
                    resolve_provider: None,
                    trigger_characters: Some(vec!["::".to_string()]),
                }),
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

    // TODO do we need this?
    fn shutdown(&self) -> Self::ShutdownFuture {
        file_dbg("shutdown", "shutdown");
        Box::new(future::ok(()))
    }

    // TODO do we need this?
    fn symbol(&self, _: WorkspaceSymbolParams) -> Self::SymbolFuture {
        file_dbg("symbol", "symbol");
        Box::new(future::ok(None))
    }

    // TODO do we need this?
    fn execute_command(&self, printer: &Printer, _: ExecuteCommandParams) -> Self::ExecuteFuture {
        file_dbg("execute", "execute");
        printer.log_message(MessageType::Info, "executing command!");
        Box::new(future::ok(None))
    }

    fn completion(&self, params: CompletionParams) -> Self::CompletionFuture {
        file_dbg("completion", "completion");
        // TODO remove unwraps
        let state = self.state.lock().unwrap();
        let doc = state
            .get(&params.text_document_position.text_document.uri)
            .unwrap();

        Box::new(future::ok(Some(CompletionResponse::Array(
            self.get_completions(&doc.text, params.text_document_position.position),
        ))))

        //if let Ok(doc) = state.get(&params.text_document_position.text_document.uri) {
        //    return Box::new(future::ok(Some(CompletionResponse::Array(
        //        self.get_completions(&doc.text, params.text_document_position.position),
        //    ))));
        //}

        //Box::new(future::ok(None))
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

    // TODO do we need this?
    fn document_highlight(&self, _: TextDocumentPositionParams) -> Self::HighlightFuture {
        file_dbg("document_highlight", "document_highlight");
        Box::new(future::ok(None))
    }

    fn did_open(&self, printer: &Printer, params: DidOpenTextDocumentParams) {
        file_dbg("didOpen", "didOpen");
        let uri = params.text_document.uri;
        if let Ok(path) = uri.to_file_path() {
            // TODO pull this from params.text_document.text
            // TODO cleanup
            if let Ok(text) = fs::read_to_string(path) {
                self.update(uri.clone(), &text);
                printer.publish_diagnostics(uri, self.get_diagnostics(&text));
            }
        }
    }

    fn did_change(&self, printer: &Printer, params: DidChangeTextDocumentParams) {
        file_dbg("didChange", "didChange");
        // TODO cleanup
        let uri = params.text_document.uri;
        let text = &params.content_changes[0].text;
        self.update(uri.clone(), text);
        printer.publish_diagnostics(uri, self.get_diagnostics(text));
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
