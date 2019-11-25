mod backend;

use crate::backend::{Backend, Language};
use clap::{App, Arg};
use tower_lsp::{LspService, Server};

fn main() {
    let matches = App::new("tremor-language-server")
        .version("0.1.0")
        .about("Tremor language server")
        .arg(
            Arg::with_name("trickle")
                .help("Support tremor query language instead of tremor script")
                // TODO support for long options with flags? or just make this an arg
                .short("q")
                .takes_value(false),
        )
        .get_matches();

    let tremor_lang = if matches.is_present("trickle") {
        Language::TremorQuery
    } else {
        Language::TremorScript
    };

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, messages) = LspService::new(Backend::new(tremor_lang));
    let handle = service.close_handle();
    let server = Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service);

    tokio::run(handle.run_until_exit(server));
}
