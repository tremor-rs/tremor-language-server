mod backend;
mod language;

use backend::Backend;
use clap::{App, Arg};
use tower_lsp::{LspService, Server};

fn main() {
    let matches = App::new("tremor-language-server")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Tremor language server")
        .arg(
            Arg::with_name("language")
                .help("Tremor language to support")
                .short("l")
                .long("language")
                .takes_value(true),
        )
        .get_matches();

    // if not set, defaults to supporting tremor-script
    let language_name = matches
        .value_of("language")
        .unwrap_or(language::TREMOR_SCRIPT);

    match language::lookup(language_name) {
        Some(language) => {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, messages) = LspService::new(Backend::new(language));
            let handle = service.close_handle();
            let server = Server::new(stdin, stdout)
                .interleave(messages)
                .serve(service);

            tokio::run(handle.run_until_exit(server));
        }
        None => {
            eprintln!("Error: unknown tremor language {}", language_name);
            std::process::exit(1)
        }
    }
}
