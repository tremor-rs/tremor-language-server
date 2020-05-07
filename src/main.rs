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

mod backend;
mod language;
mod lsp_utils;

use backend::Backend;
use clap::{App, Arg};
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    backend::file_dbg("main", "main");

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("path")
                .help("TREMOR_PATH to set")
                .short("p")
                .long("path")
                .takes_value(true)
                .default_value("")
        )
        .arg(
            Arg::with_name("language")
                .help("Tremor language to support")
                .short("l")
                .long("language")
                .takes_value(true)
                .possible_values(language::LANGUAGE_NAMES)
                .default_value(language::DEFAULT_LANGUAGE_NAME),
        )
        .get_matches();

    let path = matches
        .value_of("path")
        // this is safe because we provide a default value for this arg above
        .unwrap_or_else(|| unreachable!());

    let language_name = matches
        .value_of("language")
        // this is safe because we provide a default value for this arg above
        .unwrap_or_else(|| unreachable!());

    std::env::set_var("TREMOR_PATH", path);

    match language::lookup(language_name) {
        Some(language) => {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, messages) = LspService::new(Backend::new(language));
            Server::new(stdin, stdout)
                .interleave(messages)
                .serve(service)
                .await;
        }
        None => {
            eprintln!("Error: unknown tremor language {}", language_name);
            std::process::exit(1)
        }
    }
}
