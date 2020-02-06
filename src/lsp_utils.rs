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
use tower_lsp::lsp_types::*;

pub fn to_lsp_position(location: &language::Location) -> Position {
    // position in language server protocol is zero-based
    Position::new((location.line - 1) as u64, (location.column - 1) as u64)
}

pub fn to_lsp_severity(error_level: &language::ErrorLevel) -> DiagnosticSeverity {
    match error_level {
        language::ErrorLevel::Error => DiagnosticSeverity::Error,
        language::ErrorLevel::Warning => DiagnosticSeverity::Warning,
        language::ErrorLevel::Hint => DiagnosticSeverity::Hint,
    }
}

pub fn get_token(text: &str, position: Position) -> Option<String> {
    //file_dbg("get_token_text", text);
    //file_dbg(
    //    "get_token_position",
    //    &format!("{}:{}", position.line, position.character),
    //);

    let lines: Vec<&str> = text.split('\n').collect();

    if let Some(line) = lines.get(position.line as usize) {
        // TODO index check here
        let start_index = match line[..position.character as usize].rfind(is_token_boundary) {
            Some(i) => i + 1,
            None => 0,
        };
        let end_index = line[position.character as usize..]
            .find(is_token_boundary)
            .unwrap_or(0)
            + (position.character as usize);

        //file_dbg("get_token_start_index", &start_index.to_string());
        //file_dbg("get_token_end_index", &end_index.to_string());

        Some(line[start_index..end_index].to_string())
    } else {
        None
    }
}

// naive implementation for detecting tokens. works for our current needs
// TODO eliminate this if we use lexer directly from language trait
fn is_token_boundary(c: char) -> bool {
    // treat : as valid for token for now, since it occurs in module::function pairs
    !(c.is_alphanumeric() || c == ':')
}
