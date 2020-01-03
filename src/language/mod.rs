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

#[macro_use]
mod prelude;
mod query;
mod script;

pub use tremor_script::pos;

pub const NAMES: &[&str] = &[
    script::LANGUAGE_NAME,
    query::LANGUAGE_NAME,
    // alternate names for above
    script::FILE_EXTENSION,
    query::FILE_EXTENSION,
];

pub const DEFAULT_NAME: &str = script::LANGUAGE_NAME;

pub trait Language: Send + Sync {
    fn parse_err(&self, text: &str) -> Option<prelude::Error>;

    fn functions(&self, _module_name: &str) -> Vec<String> {
        vec![]
    }

    fn function_doc(&self, _full_function_name: &str) -> Option<&prelude::FunctionDoc> {
        None
    }
}

pub fn lookup(language_name: &str) -> Option<Box<dyn Language>> {
    match language_name {
        script::LANGUAGE_NAME | script::FILE_EXTENSION => {
            Some(Box::new(script::TremorScript::default()))
        }
        query::LANGUAGE_NAME | query::FILE_EXTENSION => {
            Some(Box::new(query::TremorQuery::default()))
        }
        _ => None,
    }
}
