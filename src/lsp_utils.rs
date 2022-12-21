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

use crate::language;
use tower_lsp::lsp_types::{DiagnosticSeverity, Position};
use tremor_script::lexer::{Spanned, Token};

use crate::backend::file_dbg;

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn to_lsp_position(location: &language::Location) -> Position {
    // position in language server protocol is zero-based
    Position::new((location.line() - 1) as u32, (location.column() - 1) as u32)
}

pub(crate) fn to_lsp_severity(error_level: &language::ErrorLevel) -> DiagnosticSeverity {
    match error_level {
        language::ErrorLevel::Error => DiagnosticSeverity::ERROR,
        language::ErrorLevel::Warning(_) => DiagnosticSeverity::WARNING,
        language::ErrorLevel::Hint => DiagnosticSeverity::HINT,
    }
}

pub(crate) fn get_token(tokens: &[language::TokenSpan], position: Position) -> Option<String> {
    let line = position.line as usize;
    let column = position.character as usize;

    //file_dbg("get_token_location_line", &location.line.to_string());
    //file_dbg("get_token_location_column", &location.column.to_string());

    let mut token = None;
    for (i, t) in tokens.iter().enumerate() {
        if t.span.end().line() == line && t.span.end().column() > column {
            //file_dbg("get_token_span_end", &token.span.end.line.to_string());
            //file_dbg("get_token_location_end", &location.line.to_string());
            file_dbg("get_token_t_value", &t.value.to_string());

            token = match t.value {
                language::Token::Ident(_, _) => {
                    if let Some(
                        [Spanned { value: v1, .. }, Spanned {
                            value: Token::ColonColon,
                            ..
                        }, Spanned { value: v2, .. }],
                    ) = tokens.get(i - 2..i)
                    {
                        Some(format!("{v1}::{v2}"))
                    } else if let Some(
                        [Spanned { value: v1, .. }, Spanned {
                            value: Token::ColonColon,
                            ..
                        }, Spanned { value: v2, .. }],
                    ) = tokens.get(i..i + 3)
                    {
                        Some(format!("{v1}::{v2}"))
                    } else {
                        None
                    }
                }
                language::Token::ColonColon => {
                    if let Some(
                        [Spanned { value: v1, .. }, Spanned {
                            value: Token::ColonColon,
                            ..
                        }, Spanned { value: v2, .. }],
                    ) = tokens.get(i - 2..i)
                    {
                        Some(format!("{v1}::{v2}"))
                    } else {
                        None
                    }
                }
                _ => None,
            };

            break;
        }
    }
    //file_dbg("get_token_return", &token.clone().unwrap_or_default());
    token
}
