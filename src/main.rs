mod backend;

use clap::{App, Arg};
use tower_lsp::{LspService, Server};

fn main() {
    let matches = App::new("tremor-language-server")
        .version("0.1.0")
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
    let backend_name = matches.value_of("backend").unwrap_or("tremor");

    match backend::lookup(backend_name) {
        Some(backend) => {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, messages) = LspService::new(backend);
            let handle = service.close_handle();
            let server = Server::new(stdin, stdout)
                .interleave(messages)
                .serve(service);

            tokio::run(handle.run_until_exit(server));
        }
        None => {
            eprintln!("Error: unknown backend {}", backend_name);
            std::process::exit(1)
        }
    }
}
