use crate::args::BuildMode;
use crate::compilers::HandlebarsCompiler;
use crate::error::{Error, Result};
use chrono::{DateTime, Local, Utc};
use glob::glob;
use markdown::mdast::{Node, Root, Toml};
use markdown::{Constructs, Options, ParseOptions};
use std::path::Path;
use tokio::fs;

#[derive(serde::Serialize)]
struct PostData {
    body: String,
    description: String,
    is_published: bool,
    title: String,
}

#[derive(Debug, serde::Deserialize)]
struct FrontMatter {
    slug: String,
    title: String,
    template: String,
    description: String,
    published: Option<DateTime<Utc>>,
}

pub struct MarkdownCompiler {
    build_mode: BuildMode,
    options: Options,
}

impl MarkdownCompiler {
    pub fn new(build_mode: BuildMode) -> Self {
        let constructs = Constructs {
            frontmatter: true,
            gfm_footnote_definition: true,
            gfm_label_start_footnote: true,
            ..Default::default()
        };

        let options = Options {
            parse: ParseOptions {
                constructs,
                ..Default::default()
            },
            ..Options::gfm()
        };

        Self {
            build_mode,
            options,
        }
    }

    pub async fn compile<'a, P: AsRef<Path>>(
        &self,
        pattern: &str,
        output_path: P,
        handlebars: &'a HandlebarsCompiler<'a>,
    ) -> Result<()> {
        let posts = glob(pattern).map_err(Error::Pattern)?;

        for post in posts {
            let post = post.map_err(Error::Glob)?;
            let content = fs::read_to_string(post.as_path())
                .await
                .map_err(Error::Io)?;
            let md = markdown::to_html_with_options(content.as_str(), &self.options).unwrap();
            let fm = self.parse_front_matter(content.as_str());

            if self.build_mode == BuildMode::Release && !is_published(&fm) {
                continue;
            }

            let mut path = output_path.as_ref().join(&fm.slug);
            path.set_extension("html");

            log::info!("{:?} -> {:?}", post, path);
            fs::create_dir_all(path.parent().unwrap())
                .await
                .map_err(Error::Io)?;

            handlebars
                .render_to_write(
                    fm.template.as_str(),
                    &PostData {
                        body: md,
                        description: fm.description,
                        is_published: fm.published.is_some(),
                        title: fm.title,
                    },
                    &path,
                )
                .await
                .unwrap();
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

fn is_published(fm: &FrontMatter) -> bool {
    fm.published
        .map(|publish_date| Local::now() > publish_date)
        .unwrap_or_default()
}
