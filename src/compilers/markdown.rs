use crate::compilers::HandlebarsCompiler;
use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use glob::glob;
use markdown::mdast::{Node, Root, Toml};
use markdown::{Constructs, Options, ParseOptions};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

#[derive(serde::Serialize)]
struct PostData {
    body: String,
    title: String,
    is_published: bool,
}

#[derive(Debug, serde::Deserialize)]
struct FrontMatter {
    slug: String,
    title: String,
    template: String,
    published: Option<DateTime<Utc>>,
}

pub struct MarkdownCompiler {
    options: Options,
}

impl MarkdownCompiler {
    pub fn new() -> Self {
        let constructs = Constructs {
            frontmatter: true,
            ..Default::default()
        };

        let options = Options {
            parse: ParseOptions {
                constructs,
                ..Default::default()
            },
            ..Default::default()
        };

        Self { options }
    }

    pub fn compile<P: AsRef<Path>>(
        &self,
        pattern: &str,
        output_path: P,
        handlebars: &HandlebarsCompiler,
    ) -> Result<()> {
        let posts = glob(pattern).map_err(Error::Pattern)?;

        for post in posts {
            let post = post.map_err(Error::Glob)?;
            let content = fs::read_to_string(post.as_path()).map_err(Error::Io)?;
            let md = markdown::to_html_with_options(content.as_str(), &self.options).unwrap();
            let fm = self.parse_front_matter(content.as_str());

            let mut path = output_path.as_ref().join(&fm.slug);
            path.set_extension("html");

            log::info!("{:?} -> {:?}", post, path);
            fs::create_dir_all(path.parent().unwrap()).map_err(Error::Io)?;

            let file = File::create(&path).unwrap();
            let file = BufWriter::new(file);

            handlebars.render_to_write(
                fm.template.as_str(),
                &PostData {
                    body: md,
                    title: fm.title,
                    is_published: fm.published.is_some(),
                },
                file,
            )
        }

        Ok(())
    }

    fn parse_front_matter(&self, content: &str) -> FrontMatter {
        let ast = markdown::to_mdast(
            content,
            &ParseOptions {
                constructs: self.options.parse.constructs.clone(),
                ..Default::default()
            },
        )
        .unwrap();

        if let Node::Root(Root { children, .. }) = ast {
            if let Some(Node::Toml(Toml { value, .. })) = children.first() {
                toml::from_str(value).unwrap()
            } else {
                panic!("No front matter");
            }
        } else {
            unreachable!();
        }
    }
}
