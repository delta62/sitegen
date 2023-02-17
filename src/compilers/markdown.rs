use crate::args::BuildMode;
use crate::compilers::HandlebarsCompiler;
use crate::error::{Error, Result};
use crate::post_cache::{PostCache, PostData, PostRef};
use chrono::{DateTime, Local, Utc};
use glob::glob;
use markdown::mdast::{Node, Root, Toml};
use markdown::{Constructs, Options, ParseOptions};
use std::path::Path;
use tokio::fs;

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
    fm_parse_options: ParseOptions,
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

        let fm_parse_options = ParseOptions {
            constructs: options.parse.constructs.clone(),
            ..Default::default()
        };

        Self {
            build_mode,
            fm_parse_options,
            options,
        }
    }

    pub async fn compile<'a, P: AsRef<Path>>(
        &self,
        pattern: &str,
        output_path: P,
        handlebars: &'a HandlebarsCompiler<'a>,
    ) -> Result<PostCache> {
        let mut post_cache = PostCache::new();
        let posts = glob(pattern).map_err(Error::Pattern)?;
        let output_path = output_path.as_ref();

        for post in posts {
            let post = post.map_err(Error::Glob)?;
            let post = self
                .render_post(&post.as_path(), handlebars, output_path)
                .await?;

            post.map(|p| post_cache.add_ref(p));
        }

        Ok(post_cache)
    }

    async fn render_post(
        &self,
        post: &Path,
        handlebars: &HandlebarsCompiler<'_>,
        output_path: &Path,
    ) -> Result<Option<PostRef>> {
        let content = fs::read_to_string(post).await.map_err(Error::Io)?;
        let md = markdown::to_html_with_options(content.as_str(), &self.options)
            .map_err(Error::MarkdownError)?;
        let fm = self.parse_front_matter(content.as_str())?;

        if self.build_mode.is_release() && !is_published(&fm) {
            return Ok(None);
        }

        let mut path = output_path.join(&fm.slug);
        path.set_extension("html");

        log::debug!("{:?} -> {:?}", post, path);
        fs::create_dir_all(path.parent().unwrap())
            .await
            .map_err(Error::Io)?;

        let post_data = PostData {
            body: md,
            intro: fm.description,
            publish_date: fm.published,
            slug: fm.slug,
            title: fm.title,
        };

        handlebars
            .render_to_write(fm.template.as_str(), &post_data, &path)
            .await?;

        Ok(Some(post_data.into()))
    }

    fn parse_front_matter(&self, content: &str) -> Result<FrontMatter> {
        let ast =
            markdown::to_mdast(content, &self.fm_parse_options).map_err(Error::MarkdownError)?;

        if let Node::Root(Root { children, .. }) = ast {
            if let Some(Node::Toml(Toml { value, .. })) = children.first() {
                toml::from_str(value).map_err(Error::Toml)
            } else {
                Err(Error::MissingFrontMatter)
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
