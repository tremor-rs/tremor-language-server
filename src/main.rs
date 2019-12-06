mod backend;

use backend::Backend;
use clap::{App, Arg};
use tower_lsp::{LspService, Server};

fn main() {
    let matches = App::new("tremor-language-server")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Tremor language server")
        .arg(
            Arg::with_name("backend")
                .help("Language backend to use")
                .short("b")
                .long("backend")
                .takes_value(true),
        )
        .get_matches();

    // defaults to supporting tremor file type (i.e. tremor-script)
    let language_name = matches.value_of("backend").unwrap_or("tremor");

    // TODO rename this to language module
    match backend::lookup(language_name) {
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
            eprintln!("Error: unknown backend {}", language_name);
            std::process::exit(1)
        }
    }
}
