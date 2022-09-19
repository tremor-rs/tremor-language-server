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

#![deny(warnings)]
#![deny(clippy::all, clippy::pedantic)]

mod backend;
mod language;
mod lsp_utils;

use backend::Backend;
use clap::{Arg, Command};
use tower_lsp::{LspService, Server};

#[async_std::main]
async fn main() {
    backend::file_dbg("main", "main");

    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::new("language")
                .help("Tremor language to support")
                .short('l')
                .long("language")
                .takes_value(true)
                .possible_values(language::LANGUAGE_NAMES)
                .default_value(language::DEFAULT_LANGUAGE_NAME),
        )
        .arg(
            Arg::new("path")
                .help("TREMOR_PATH to set")
                .short('p')
                .long("path")
                .takes_value(true)
                .default_value(""),
        )
        .get_matches();

    let language_name = matches
        .value_of("language")
        // this is safe because we provide a default value for this arg above
        .unwrap_or_else(|| unreachable!());

    let path = matches
        .value_of("path")
        // this is safe because we provide a default value for this arg above
        .unwrap_or_else(|| unreachable!());

    if !path.is_empty() {
        std::env::set_var(
            "TREMOR_PATH",
            match std::env::var("TREMOR_PATH") {
                // append to existing path if it's already set
                Ok(p) => format!("{}:{}", p, path),
                Err(_) => path.to_string(),
            },
        );
    }

    if let Some(language) = language::lookup(language_name) {
        let (stdin, stdout) = (async_std::io::stdin(), async_std::io::stdout());
        let (service, socket) = LspService::new(|client| Backend::new(client, language));
        Server::new(stdin, stdout, socket).serve(service).await;
    } else {
        eprintln!("Error: unknown tremor language {}", language_name);
        std::process::exit(1)
    }
}
