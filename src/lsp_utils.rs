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

use crate::backend::file_dbg;

pub fn to_lsp_position(location: &language::Location) -> Position {
    // position in language server protocol is zero-based
    Position::new((location.line() - 1) as u64, (location.column() - 1) as u64)
}

#[allow(clippy::cast_possible_truncation)]
pub fn to_language_location(position: &Position) -> language::Location {
    // location numbers in our languages starts from one
    language::Location::new(
        (position.line + 1) as usize, // we should never have line positions > 32 bit
        (position.character + 1) as usize, // we should never have character positions > 32 bit
        0,
        0, // absolute byte offset -- we don't use it here so setting to 0
    )
}

pub fn to_lsp_severity(error_level: &language::ErrorLevel) -> DiagnosticSeverity {
    match error_level {
        language::ErrorLevel::Error => DiagnosticSeverity::Error,
        language::ErrorLevel::Warning => DiagnosticSeverity::Warning,
        language::ErrorLevel::Hint => DiagnosticSeverity::Hint,
    }
}

pub fn get_token(tokens: &[language::TokenSpan], position: Position) -> Option<String> {
    let location = to_language_location(&position);

    //file_dbg("get_token_location_line", &location.line.to_string());
    //file_dbg("get_token_location_column", &location.column.to_string());

    let mut token = None;
    for (i, t) in tokens.iter().enumerate() {
        if t.span.end.line() == location.line() && t.span.end.column() > location.column() {
            //file_dbg("get_token_span_end", &token.span.end.line.to_string());
            //file_dbg("get_token_location_end", &location.line.to_string());
            file_dbg("get_token_t_value", &t.value.to_string());

            token = match t.value {
                language::Token::Ident(_, _) => {
                    if language::Token::ColonColon == tokens[i - 1].value {
                        Some(format!(
                            "{}{}{}",
                            tokens[i - 2].value,
                            tokens[i - 1].value,
                            tokens[i].value
                        ))
                    } else if language::Token::ColonColon == tokens[i + 1].value {
                        Some(format!(
                            "{}{}{}",
                            tokens[i].value,
                            tokens[i + 1].value,
                            tokens[i + 2].value,
                        ))
                    } else {
                        None
                    }
                }

                language::Token::ColonColon => Some(format!(
                    "{}{}{}",
                    tokens[i - 1].value,
                    //t.value,
                    tokens[i].value,
                    tokens[i + 1].value
                )),
                _ => None,
            };

            break;
        }
    }
    //file_dbg("get_token_return", &token.clone().unwrap_or_default());
    token
}
