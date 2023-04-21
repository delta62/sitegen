use crate::args::BuildMode;
use crate::compilers::HandlebarsCompiler;
use crate::error::{Error, Result};
use crate::post_cache::PostCache;
use chrono::{Local, TimeZone};
use glob::glob;
use markdown::mdast::{Node, Root, Toml};
use markdown::{Constructs, Options, ParseOptions};
use std::path::Path;
use tokio::fs;

pub type FrontMatter = toml::Table;

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
    ) -> Result<Option<FrontMatter>> {
        let content = fs::read_to_string(post).await.map_err(Error::Io)?;
        let md = markdown::to_html_with_options(content.as_str(), &self.options)
            .map_err(Error::MarkdownError)?;
        let fm = self.parse_front_matter(content.as_str())?;

        if self.build_mode.is_release() && !is_published(&fm) {
            return Ok(None);
        }

        let template = fm.get("template").and_then(|tpl| tpl.as_str()).unwrap();
        let slug = slug(&fm).unwrap();
        let mut path = output_path.join(slug);
        path.set_extension("html");

        log::debug!("{:?} -> {:?}", post, path);
        fs::create_dir_all(path.parent().unwrap())
            .await
            .map_err(Error::Io)?;

        handlebars.render_to_write(template, &fm, &path).await?;

        Ok(Some(fm))
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

fn slug(fm: &FrontMatter) -> Option<&str> {
    fm.get("slug").and_then(|slug| slug.as_str())
}

fn is_published(fm: &FrontMatter) -> bool {
    fm.get("published")
        .and_then(|published| published.as_datetime())
        .map(|publish_date| {
            let y: i32;
            let m: u32;
            let d: u32;

            match publish_date.date {
                Some(date) => {
                    y = date.year as i32;
                    m = date.month as u32;
                    d = date.day as u32;
                }
                None => {
                    y = 3000;
                    m = 1;
                    d = 1;
                }
            }

            let now = Local::now();
            let pub_date = Local.with_ymd_and_hms(y, m, d, 0, 0, 0).unwrap();

            pub_date < now
        })
        .unwrap_or_default()
}
