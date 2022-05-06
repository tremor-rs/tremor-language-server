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

use crate::language::prelude::*;

pub(crate) const LANGUAGE_NAME: &str = "tremor-deploy";
pub(crate) const FILE_EXTENSION: &str = "troy";

#[derive(Debug)]
pub(crate) struct TremorDeploy {
    registry: registry::Registry,
    aggr_registry: registry::Aggr,
}

impl Default for TremorDeploy {
    fn default() -> Self {
        Self {
            registry: registry::registry(),
            aggr_registry: registry::aggr(),
        }
    }
}

impl Language for TremorDeploy {
    fn parse_errors(&self, _uri: &Url, text: &str) -> Option<Vec<Error>> {
        // FIXME .unwrap() should we path in something here?

        match Deploy::parse_with_aid(text, &self.registry, &self.aggr_registry) {
            Ok(deploy) => {
                let r = Some(deploy.warnings.iter().map(Into::into).collect());
                unsafe { deploy.consume_and_free().unwrap() };
                r
            }
            Err(tremor_script::errors::ErrorWithIndex(aid, e)) => {
                let r = Some(vec![(&e).into()]);
                unsafe { Arena::delte_index_this_is_really_unsafe_dont_use_it(aid).unwrap() };
                r
            }
        }
    }
}
