// Copyright 2020-2021, The Tremor Team
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

use crate::{language, lsp_utils};
use async_std::sync::Mutex;
use halfbrown::HashMap;
use serde_json::Value;
use std::fs;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentHighlight, DocumentHighlightParams, Documentation, ExecuteCommandParams, Hover,
    HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult,
    InitializedParams, InsertTextFormat, MarkupContent, MarkupKind, MessageType, OneOf, Position,
    Range, ServerCapabilities, ServerInfo, SymbolInformation, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url, WorkDoneProgressOptions, WorkspaceFoldersServerCapabilities,
    WorkspaceSymbolParams,
};
use tower_lsp::{jsonrpc::Result, lsp_types::WorkspaceServerCapabilities};
use tower_lsp::{Client, LanguageServer};
use tremor_script::arena::Arena;
use tremor_script::highlighter::ErrorLevel;

// stores the latest state of the document as it changes (on edits)
// TODO can add more fields here based on ast parsing
#[derive(Debug, Default)]
struct DocumentState {
    text: String,
}

// mapping of file uri to its server document state
type State = HashMap<Url, DocumentState>;

pub(crate) struct Backend {
    client: Client,
    language: Box<dyn language::Language>,
    state: Mutex<State>,
}

impl Backend {
    pub(crate) fn new(client: Client, language: Box<dyn language::Language>) -> Self {
        Self {
            client,
            language,
            state: Mutex::new(State::new()),
        }
    }

    async fn update(&self, uri: Url, text: &str) {
        // TODO implement update as well. also remove unwraps
        self.state.lock().await.insert(
            uri,
            DocumentState {
                text: text.to_string(),
            },
        );
    }

    // LSP helper functions

    fn get_diagnostics(&self, uri: &Url, text: &str) -> Vec<Diagnostic> {
        file_dbg("get_diagnostics", text);

        let mut diagnostics = Vec::new();

        if let Some(errors) = self.language.parse_errors(uri, text) {
            for e in &errors {
                let range = Range {
                    start: lsp_utils::to_lsp_position(&e.start()),
                    end: lsp_utils::to_lsp_position(&e.end()),
                };

                let mut message = e.callout().to_string();
                if let Some(hint) = &e.hint() {
                    // comma here splits the message into multiple lines
                    message = format!("{message}, Note: {hint}");
                }

                if let ErrorLevel::Warning(class) = e.level() {
                    message = format!("{class}: {message}");
                }

                diagnostics.push(Diagnostic {
                    range,
                    message,
                    severity: Some(lsp_utils::to_lsp_severity(*e.level())),
                    source: Some("tremor-language-server".to_string()),
                    ..Diagnostic::default()
                });
            }
        }

        diagnostics
    }

    fn get_completions(&self, uri: &Url, text: &str, position: Position) -> Vec<CompletionItem> {
        let pre_position = Position {
            line: position.line,
            character: position.character - 1,
        };

        if let Ok((aid, tokens)) = self.language.tokenize(uri, text) {
            if let Some(token) = lsp_utils::get_token(&tokens, pre_position) {
                file_dbg("get_completions_token", &token);
                // TODO eliminate the need for this by improving get_token()
                let module_parts: Vec<&str> = token.rsplitn(2, "::").collect();

                if let Some(module_name) = module_parts.get(1) {
                    file_dbg("get_completions_module_name", module_name);
                    let res = self
                        .language
                        .functions(uri, module_name)
                        .iter()
                        .map(|function_name| {
                            let mut detail = None;
                            let mut documentation = None;
                            let mut insert_text = None;
                            if let Some(function_doc) = self
                                .language
                                .function_doc(uri, &format!("{module_name}::{function_name}"))
                            {
                                file_dbg("get_completions_function_doc", &function_doc.description);
                                detail = Some(function_doc.signature.to_string());
                                documentation = Some(Documentation::MarkupContent(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: function_doc.description.clone(),
                                }));
                                let args_snippet = function_doc
                                    .signature
                                    .args
                                    .iter()
                                    .enumerate()
                                    // produces snippet text like ${1:arg} (where arg is the placeholder text)
                                    // https://microsoft.github.io/language-server-protocol/specifications/specification-3-14/#snippet-syntax
                                    .map(|(i, arg)| format!("${{{}:{}}}", i + 1, arg))
                                    .collect::<Vec<String>>()
                                    .join(", ");
                                insert_text = Some(format!("{function_name}({args_snippet})"));
                            };
                            CompletionItem {
                                label: function_name.to_string(),
                                kind: Some(CompletionItemKind::FUNCTION),
                                detail,
                                documentation,
                                insert_text,
                                insert_text_format: Some(InsertTextFormat::SNIPPET),
                                ..CompletionItem::default()
                            }
                        })
                        .collect();
                    unsafe { Arena::delte_index_this_is_really_unsafe_dont_use_it(aid).unwrap() };
                    return res;
                }
            }
            unsafe { Arena::delte_index_this_is_really_unsafe_dont_use_it(aid).unwrap() }
        }

        vec![]
    }

    fn get_hover_content(
        &self,
        uri: &Url,
        text: &str,
        position: Position,
    ) -> Option<MarkupContent> {
        // TODO merge the repeated tokenize operation with get_completions()?
        if let Ok((aid, tokens)) = self.language.tokenize(uri, text) {
            if let Some(token) = lsp_utils::get_token(&tokens, position) {
                file_dbg("get_hover_content_token", &token);
                if let Some(function_doc) = self.language.function_doc(uri, &token) {
                    file_dbg("get_hover_content_function_doc", &function_doc.description);
                    let res = Some(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: function_doc.to_string(),
                    });
                    unsafe { Arena::delte_index_this_is_really_unsafe_dont_use_it(aid).unwrap() }
                    return res;
                }
            }
            unsafe { Arena::delte_index_this_is_really_unsafe_dont_use_it(aid).unwrap() }
        }
        None
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "tremor-language-server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![":".to_string()]),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    ..CompletionOptions::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..WorkspaceServerCapabilities::default()
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        file_dbg("initialized", "initialized");

        // TODO check this from clients
        //self.client.show_message(MessageType::Info, "Initialized Trill!").await;
        self.client
            .log_message(MessageType::INFO, "Initialized Trill!")
            .await;
    }

    // TODO do more here (as appropriate). manadatory implementations for the trait

    async fn shutdown(&self) -> Result<()> {
        file_dbg("shutdown", "shutdown");
        Ok(())
    }

    async fn symbol(&self, _: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        file_dbg("symbol", "symbol");
        Ok(None)
    }

    async fn document_highlight(
        &self,
        _: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        file_dbg("document_highlight", "document_highlight");
        Ok(None)
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        file_dbg("execute", "execute");
        self.client
            .log_message(MessageType::INFO, "executing command!")
            .await;
        Ok(None)
    }

    // backend state updates on text edits and reporting of diagnostics

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        file_dbg("didOpen", "didOpen");
        file_dbg("didOpen_language", &params.text_document.language_id);

        let uri = params.text_document.uri;
        if let Ok(path) = uri.to_file_path() {
            // TODO pull this from params.text_document.text
            // TODO cleanup
            if let Ok(text) = fs::read_to_string(path) {
                self.update(uri.clone(), &text).await;
                let d = self.get_diagnostics(&uri, &text);
                self.client.publish_diagnostics(uri, d, None).await;
            }
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        file_dbg("didChange", "didChange");
        // TODO cleanup
        let uri = params.text_document.uri;
        let text = &params.content_changes[0].text;
        self.update(uri.clone(), text).await;
        self.client
            .publish_diagnostics(uri.clone(), self.get_diagnostics(&uri, text), None)
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        file_dbg("didClose", "didClose");
        // TODO can cleanup backend state here
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    // other lsp features
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        file_dbg("completion", "completion");

        // TODO remove unwraps
        let state = self.state.lock().await;
        let doc = state
            .get(&params.text_document_position.text_document.uri)
            .unwrap();
        let uri = params.text_document_position.text_document.uri;

        Ok(Some(CompletionResponse::Array(self.get_completions(
            &uri,
            &doc.text,
            params.text_document_position.position,
        ))))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        file_dbg("hover", "hover");
        // TODO remove unwraps
        // TODO bake state lookup in self
        let state = self.state.lock().await;
        let uri = params.text_document_position_params.text_document.uri;
        let doc = state.get(&uri).unwrap();

        let result = self
            .get_hover_content(
                &uri,
                &doc.text,
                params.text_document_position_params.position,
            )
            .map(|hover_content| Hover {
                contents: HoverContents::Markup(hover_content),
                range: None,
            });

        Ok(result)
    }
}

// TODO remove. just for testing right now
pub(crate) fn file_dbg(name: &str, content: &str) {
    use std::env::temp_dir;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    let mut path = PathBuf::new();
    path.push(temp_dir());
    path.push(format!("tremor_{name}"));

    let mut output = File::create(path).unwrap();
    write!(output, "{content}").unwrap();
}

#[cfg(test)]
mod tests {
    use async_std::prelude::{FutureExt, StreamExt};
    use serde_json::json;
    use tower::Service;
    use tower_lsp::jsonrpc::{Id, Request};
    use tower_lsp::LspService;

    use super::*;

    const VERSION: &str = env!("CARGO_PKG_VERSION");

    #[async_std::test]
    async fn backend() -> Result<()> {
        let lang = language::lookup("tremor-deploy").unwrap();
        let (mut service, _socket) = LspService::new(|client| Backend::new(client, lang));
        let req = Request::build("initialize")
            .params(json!({"capabilities":{}}))
            .id(1)
            .finish();
        let res = service
            .call(req)
            .await
            .expect("Expect request to be executed")
            .expect("Expect response");
        assert_eq!(res.id(), &Id::Number(1));
        assert!(res.is_ok());
        assert_eq!(
            &json!({
                "capabilities": {
                    "completionProvider": {
                        "triggerCharacters": [":"],
                    },
                    "textDocumentSync": 1,
                    "workspace": {
                        "workspaceFolders": {
                            "changeNotifications": true,
                            "supported": true,
                        }
                    },
                    "hoverProvider": true
                },
                "serverInfo": {
                    "name": "tremor-language-server",
                    "version": VERSION
                }
            }),
            res.result().unwrap()
        );
        Ok(())
    }

    #[async_std::test]
    async fn warning_class() {
        tracing_subscriber::fmt::init();

        let lang = language::lookup("tremor-deploy").unwrap();
        let (mut service, mut socket) = LspService::new(|client| Backend::new(client, lang));

        let join_handle = async_std::task::spawn(async move {
            while let Some(x) = socket.next().await {
                if x.method() == "textDocument/publishDiagnostics" {
                    let params = x.params();

                    if let Some(params) = params {
                        let message = params.get("diagnostics").unwrap().as_array().unwrap()[0]
                            .get("message")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string();

                        return Some(message);
                    }
                }
            }

            None
        });

        let initialize_req = Request::build("initialize")
            .params(json!({"capabilities":{
            "textDocument": {
                "synchronization": {
                    "dynamicRegistration": true,
                }
            }}}))
            .id(1)
            .finish();

        let _initialize_res = service
            .call(initialize_req)
            .await
            .expect("Expect request to be executed");

        let initialized_req = Request::build("initialized").params(json!({})).finish();
        let _initialized_res = service
            .call(initialized_req)
            .await
            .expect("Expect request to be executed");

        let req = Request::build("textDocument/didOpen")
            .params(json!({"textDocument": {
                "uri": format!("file://{}/{}", env!("CARGO_MANIFEST_DIR"), "tests/warning_class.tremor"),
                "languageId": "tremor-deploy",
                "version": 1,
                "text": ""
            }}))
            .finish();
        let _res = service
            .call(req)
            .await
            .expect("Expect request to be executed");

        let result = join_handle.timeout(std::time::Duration::from_secs(5)).await;

        assert_eq!(
            result,
            Ok(Some(
                "consistency: const's are canonically written in UPPER_CASE".to_string()
            ))
        );
    }
}
