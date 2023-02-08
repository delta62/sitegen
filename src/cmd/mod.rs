use std::fs::{self, File};
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
        log::info!("{} {}", request.method(), request.url());

        let path = &request.url()[1..];
        let path = Path::new(config.out_dir.as_str()).join(path);
        let mut extension = path.extension().map(|e| e.to_string_lossy().to_string());

        let file = fs::metadata(path.as_path()).and_then(|metadata| {
            if metadata.is_dir() {
                let index = Path::new(path.as_path()).join("pages/index.html");
                extension = index.extension().map(|e| e.to_string_lossy().to_string());
                File::open(&index)
            } else {
                File::open(&path)
            }
        });

        match file {
            Ok(file) => {
                let content_type = if let Some(e) = extension {
                    match e.as_str() {
                        "html" => "text/html",
                        "css" => "text/css",
                        _ => "text/plain",
                    }
                } else {
                    "text/html"
                };
                let content_type =
                    Header::from_str(format!("Content-Type: {}", content_type).as_str()).unwrap();
                let res = Response::from_file(file).with_header(content_type);
                request.respond(res).unwrap();
            }
            Err(e) => {
                log::error!("{:?}", e);
                let res = Response::from_string(format!("{:?}", e)).with_status_code(404);
                request.respond(res).unwrap();
            }
        }
    }

    Ok(())
}
