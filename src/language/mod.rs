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

mod prelude;
mod query;
mod script;

pub use tremor_script::pos;

// language names
pub const TREMOR_SCRIPT: &str = "tremor-script";
pub const TREMOR_QUERY: &str = "tremor-query";

// file extensions
pub const TREMOR_SCRIPT_FILE_EXT: &str = "tremor";
pub const TREMOR_QUERY_FILE_EXT: &str = "trickle";

pub trait Language: Send + Sync {
    fn parse_err(&self, text: &str) -> Option<prelude::Error>;

    fn functions(&self, _module_name: &str) -> Vec<String> {
        vec![]
    }
}

pub fn lookup(name: &str) -> Option<Box<dyn Language>> {
    match name {
        TREMOR_SCRIPT | TREMOR_SCRIPT_FILE_EXT => Some(Box::new(script::TremorScript::default())),
        TREMOR_QUERY | TREMOR_QUERY_FILE_EXT => Some(Box::new(query::TremorQuery::default())),
        _ => None,
    }
}
