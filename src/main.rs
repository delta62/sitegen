mod args;
mod config;
mod error;

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::args::Args;
use crate::config::Config;
use crate::error::{Error, Result};
use clap::Parser;
use handlebars::Handlebars;
use markdown::mdast::{Node, Root, Toml};
use markdown::{Constructs, Options, ParseOptions};
use toml::value::Datetime;

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let cwd = args.cwd()?;
    let cfg_path = cwd.join("config.toml");
    let config = Config::load(cfg_path).expect("Unable to read config file");
    let mut reg = Handlebars::new();
    log::info!("{:?}", config);

    let partials_path = config.partials_path(&cwd);
    let partials = fs::read_dir(&partials_path).map_err(Error::IoError)?;
    for partial in partials {
        let partial = partial.map_err(Error::IoError)?;
        let path = partial.path();
        let name = path.as_path().file_stem().unwrap().to_str().unwrap();
        let content = fs::read_to_string(partial.path()).map_err(Error::IoError)?;

        log::info!("adding partial {}", name);
        reg.register_partial(name, content).unwrap();
    }

    let page_path = config.page_path(&cwd);
    let pages = fs::read_dir(&page_path).map_err(Error::IoError)?;
    for page in pages {
        let page = page.map_err(Error::IoError)?;
        let path = page.path();
        let path = path.as_path().strip_prefix(&page_path).unwrap();
        let mut path = config.output_path(&cwd, path);
        path.set_extension("html");

        log::info!("render {:?} -> {:?}", page.path(), path);

        fs::create_dir_all(path.parent().unwrap()).map_err(Error::IoError)?;
        let contents = fs::read_to_string(page.path()).unwrap();
        let file = File::create(&path).unwrap();
        let file = BufWriter::new(file);
        reg.render_template_to_write(contents.as_str(), &(), file)
            .unwrap();
    }

    let constructs = Constructs {
        frontmatter: true,
        ..Default::default()
    };

    let opts = Options {
        parse: ParseOptions {
            constructs: constructs.clone(),
            ..Default::default()
        },
        ..Default::default()
    };

    let post_path = config.post_path(&cwd);
    let posts = fs::read_dir(&post_path).map_err(Error::IoError)?;
    for post in posts {
        let post = post.map_err(Error::IoError)?;
        let content = fs::read_to_string(&post.path()).map_err(Error::IoError)?;
        let md = markdown::to_html_with_options(content.as_str(), &opts).unwrap();

        let a = markdown::to_mdast(
            content.as_str(),
            &ParseOptions {
                constructs: constructs.clone(),
                ..Default::default()
            },
        )
        .unwrap();

        if let Node::Root(Root { children, .. }) = a {
            if let Some(Node::Toml(Toml { value, .. })) = children.first() {
                let fm: FrontMatter = toml::from_str(value).unwrap();
                let path = config.output_path(&cwd, &fm.slug);

                log::info!("{:?} -> {:?}", post.path(), path);
                fs::create_dir_all(path.parent().unwrap()).map_err(Error::IoError)?;
                let file = File::create(&path).unwrap();
                let file = BufWriter::new(file);
                reg.render_to_write(fm.template.as_str(), &PostData { body: md }, file)
                    .unwrap();
            } else {
                panic!("no front matter");
            }
        } else {
            unreachable!();
        }
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct PostData {
    body: String,
}

#[derive(Debug, serde::Deserialize)]
struct FrontMatter {
    slug: String,
    created: Datetime,
    template: String,
}
