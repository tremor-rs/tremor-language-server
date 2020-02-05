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

pub use super::Language;

pub use std::collections::HashMap;
pub use tremor_script::docs::FunctionDoc;
pub use tremor_script::highlighter::Error;

pub use tremor_script::registry;

macro_rules! load_function_docs {
    ($language_name:expr) => {{
        let bytes = include_bytes!(concat!(
            env!("OUT_DIR"),
            "/function_docs.",
            $language_name,
            ".bin"
        ));

        match bincode::deserialize::<HashMap<String, FunctionDoc>>(bytes) {
            Ok(function_docs) => {
                //println!("{:?}", function_docs.get("stats::min").unwrap().signature);
                function_docs
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                HashMap::new()
                //std::process::exit(1)
            }
        }
    }};
}
