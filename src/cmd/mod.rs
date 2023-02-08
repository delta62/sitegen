use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use tiny_http::{Header, Response, Server};

use crate::compilers::{CompilerOptions, HandlebarsCompiler, MarkdownCompiler, SassCompiler};
use crate::config::Config;
use crate::{
    args::Args,
    error::{Error, Result},
};

pub fn clean(_args: &Args, config: &Config) -> Result<()> {
    std::fs::remove_dir_all(config.out_dir.as_str()).map_err(Error::IoError)
}

pub fn build(_args: &Args, config: &Config) -> Result<()> {
    let sass_opts = CompilerOptions {
        input_pattern: config.style_pattern.as_str(),
        output_path: config.out_dir.as_str(),
    };
    SassCompiler::compile(&sass_opts).unwrap();

    let mut handlebars = HandlebarsCompiler::new();
    handlebars.add_partials(config.partials_pattern.as_str())?;
    handlebars.compile_all(config.page_pattern.as_str(), config.out_dir.as_str())?;

    let markdown = MarkdownCompiler::new();
    markdown.compile(
        config.post_pattern.as_str(),
        config.out_dir.as_str(),
        &handlebars,
    )
}

pub fn serve(args: &Args, config: &Config) -> Result<()> {
    build(args, config)?;

    let server = Server::http("localhost:8080").unwrap();

    for request in server.incoming_requests() {
        let path = &request.url()[1..];
        let path = Path::new(config.out_dir.as_str()).join(path);
        log::info!("{} {:?}", request.method(), path);

        match File::open(path) {
            Ok(file) => {
                let response = Response::from_file(file)
                    .with_header(Header::from_str("Content-Type: text/html").unwrap())
                    .with_status_code(200);

                request.respond(response).unwrap();
            }
            Err(e) => {
                log::error!("{:?}", e);
                let response = Response::from_string(format!("{:?}", e)).with_status_code(500);

                request.respond(response).unwrap();
            }
        }
    }

    Ok(())
}
