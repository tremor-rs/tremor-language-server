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
use tremor_script::{
    arena::Index,
    errors::ErrorWithIndex,
    module::{Id, Module},
};

pub(crate) const LANGUAGE_NAME: &str = "tremor-script";
pub(crate) const FILE_EXTENSION: &str = "tremor";

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default)]
pub(crate) struct TremorScript {}

fn parse_with_aid(src: &str) -> Result<(Module, Index), ErrorWithIndex> {
    let (aid, src) = Arena::insert(src).unwrap();
    let mut ids = Vec::new();
    let id = Id::from(src.as_bytes());
    Module::load(id, &mut ids, aid, src)
        .map_err(|e| ErrorWithIndex(aid, e))
        .map(|m| (m, aid))
}
impl Language for TremorScript {
    fn parse_errors(&self, _uri: &Url, text: &str) -> Option<Vec<Error>> {
        // FIXME .unwrap() should we path in something here?

        match parse_with_aid(text) {
            Ok((module, aid)) => {
                drop(module);
                unsafe { Arena::delte_index_this_is_really_unsafe_dont_use_it(aid).unwrap() };
                None
            }
            Err(ErrorWithIndex(aid, e)) => {
                let r = Some(vec![(&e).into()]);
                unsafe { Arena::delte_index_this_is_really_unsafe_dont_use_it(aid).unwrap() };
                r
            }
        }
    }
}
