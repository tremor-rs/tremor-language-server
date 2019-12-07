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
use tremor_script::query::Query;

pub const LANGUAGE_NAME: &str = "tremor-query";
pub const FILE_EXTENSION: &str = "trickle";

#[derive(Debug)]
pub struct TremorQuery {
    registry: registry::Registry,
    aggr_registry: registry::Aggr,
}

impl Default for TremorQuery {
    fn default() -> Self {
        Self {
            registry: registry::registry(),
            aggr_registry: registry::aggr(),
        }
    }
}

impl Language for TremorQuery {
    fn parse_err(&self, text: &str) -> Option<Error> {
        Query::parse(text, &self.registry, &self.aggr_registry).err()
    }
}
