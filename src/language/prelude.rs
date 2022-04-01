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

pub(crate) use tower_lsp::lsp_types::Url;
pub(crate) use tremor_script::arena::{self, Arena};
pub(crate) use tremor_script::deploy::Deploy;
pub(crate) use tremor_script::docs::FunctionDoc;
use tremor_script::errors::Result;
pub(crate) use tremor_script::highlighter::Error;
pub(crate) use tremor_script::registry;

pub(crate) use tremor_script::lexer::{Lexer, Token, TokenSpan};

pub(crate) trait Language: Send + Sync {
    fn parse_errors(&self, uri: &Url, text: &str) -> Option<Vec<Error>>;

    fn functions(&self, _uri: &Url, _module_name: &str) -> Vec<String> {
        vec![]
    }

    fn function_doc(&self, _uri: &Url, _full_function_name: &str) -> Option<&FunctionDoc> {
        None
    }

    fn tokenize<'input>(
        &self,
        _uri: &Url,
        text: &'input str,
    ) -> Result<(arena::Index, Vec<TokenSpan<'input>>)> {
        let (aid, text) = Arena::insert(text)?;
        let v = Lexer::new(text, aid).collect::<Result<_>>()?;
        Ok((aid, v))
    }
}
