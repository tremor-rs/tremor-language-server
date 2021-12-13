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
use crate::language::script::TremorScript;
use tremor_script::path::ModulePath;
use tremor_script::query::Query;

pub const LANGUAGE_NAME: &str = "tremor-query";
pub const FILE_EXTENSION: &str = "trickle";

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct TremorQuery {
    registry: registry::Registry,
    aggr_registry: registry::Aggr,
    all_function_docs: HashMap<String, FunctionDoc>,
    // tremor-query is built on top of tremor-script
    tremor_script: TremorScript,
}

impl Default for TremorQuery {
    fn default() -> Self {
        Self {
            registry: registry::registry(),
            aggr_registry: registry::aggr(),
            all_function_docs: load_function_docs!("tremor-query"),
            // TODO might want to pass the same registry here
            tremor_script: TremorScript::default(),
        }
    }
}

impl Language for TremorQuery {
    fn parse_errors(&self, uri: &Url, text: &str) -> Option<Vec<Error>> {
        // FIXME .unwrap() should we path in something here?
        let mut m = ModulePath::load();
        let file = uri.as_str().replace("file://", "");
        let p = Path::new(&file);
        m.add(p.ancestors().nth(2).unwrap().to_str().unwrap().to_string());
        let cus = vec![];
        match Query::parse(&m, "<file>", text, cus, &self.registry, &self.aggr_registry) {
            Ok(query) => Some(query.warnings.iter().map(Into::into).collect()),
            Err(ref e) => Some(vec![e.into()]),
        }
    }

    fn functions(&self, uri: &Url, module_name: &str) -> Vec<String> {
        self.aggr_registry.find_module(module_name).map_or_else(
            || self.tremor_script.functions(uri, module_name),
            |module| {
                let mut vec: Vec<String> = module.keys().cloned().collect();
                vec.sort();
                vec
            },
        )
    }

    fn function_doc(&self, uri: &Url, full_function_name: &str) -> Option<&FunctionDoc> {
        self.all_function_docs
            .get(full_function_name)
            .or_else(|| self.tremor_script.function_doc(uri, full_function_name))
    }
}
