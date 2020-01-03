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

use crate::language::prelude::*;
use tremor_script::script::Script;

pub const LANGUAGE_NAME: &str = "tremor-script";
pub const FILE_EXTENSION: &str = "tremor";

#[derive(Debug)]
pub struct TremorScript {
    registry: registry::Registry,
    all_function_docs: HashMap<String, FunctionDoc>,
}

impl Default for TremorScript {
    fn default() -> Self {
        Self {
            registry: registry::registry(),
            all_function_docs: load_function_docs!("tremor-script"),
        }
    }
}

impl Language for TremorScript {
    fn parse_err(&self, text: &str) -> Option<Error> {
        Script::parse(text, &self.registry).err()
    }

    fn functions(&self, module_name: &str) -> Vec<String> {
        if let Some(module) = self.registry.functions.get(module_name) {
            let mut vec: Vec<String> = module.keys().cloned().collect();
            vec.sort();
            vec
        } else {
            vec![]
        }
    }

    fn function_doc(&self, full_function_name: &str) -> Option<&FunctionDoc> {
        self.all_function_docs.get(full_function_name)
    }
}
