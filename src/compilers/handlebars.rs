use crate::{
    args::BuildMode,
    error::{Error, Result},
    post_cache::PostCache,
};
use glob::glob;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, Renderable,
};
use serde::Serialize;
use std::{fs::read_to_string, path::Path};
use tokio::fs::{self, write};

use super::FrontMatter;

struct DevOnly;

impl HelperDef for DevOnly {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let is_dev_mode = ctx
            .data()
            .get("dev_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if !is_dev_mode {
            return Ok(());
        }

        let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
        let tpl = r.get_template(param).unwrap();

        tpl.render(r, ctx, rc, out)?;

        Ok(())
    }
}

struct InlineSvg;

impl HelperDef for InlineSvg {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _r: &'reg Handlebars<'reg>,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let path = h.param(0).and_then(|v| v.value().as_str()).unwrap();
        let svg = read_to_string(path)?;

        out.write(&svg)?;

        Ok(())
    }
}

pub struct HandlebarsCompiler<'a> {
    build_mode: BuildMode,
    registry: Handlebars<'a>,
}

impl<'a> HandlebarsCompiler<'a> {
    pub fn new(build_mode: BuildMode) -> Self {
        let mut registry = Handlebars::new();
        registry.set_strict_mode(true);
        registry.set_dev_mode(build_mode == BuildMode::Development);

        registry.register_helper("ifdev", Box::new(DevOnly));
        registry.register_helper("svg", Box::new(InlineSvg));

        Self {
            build_mode,
            registry,
        }
    }

    pub async fn add_partials(&mut self, pattern: &str) -> Result<()> {
        let partials = glob(pattern).map_err(Error::Pattern)?;

        for partial in partials {
            let partial = partial.map_err(Error::Glob)?;
            let name = partial.as_path().file_stem().unwrap().to_str().unwrap();
            let content = fs::read_to_string(&partial).await.map_err(Error::Io)?;

            log::debug!("adding partial {}", name);
            self.registry.register_partial(name, content).unwrap();
        }

        Ok(())
    }

    pub async fn compile_all<P: AsRef<Path>>(
        &self,
        pattern: &str,
        output_path: P,
        post_cache: &PostCache,
    ) -> Result<()> {
        let pages = glob(pattern).map_err(Error::Pattern)?;

        for page in pages {
            let page = page.map_err(Error::Glob)?;
            let file_name = page.file_name().unwrap();
            let mut path = output_path.as_ref().join(file_name);
            path.set_extension("html");

            log::debug!("render {:?} -> {:?}", page, path);

            let dev_mode = self.build_mode == BuildMode::Development;
            let posts = post_cache.posts();
            let context = PageContext { dev_mode, posts };
            let contents = fs::read_to_string(page).await.map_err(Error::Io)?;
            let rendered = self
                .registry
                .render_template(contents.as_str(), &context)
                .map_err(Error::HandlebarsError)?;

            write(&path, rendered.as_str()).await.map_err(Error::Io)?;
        }

        Ok(())
    }

    pub async fn render_to_write<S: Serialize, P: AsRef<Path>>(
        &self,
        template: &str,
        data: S,
        path: P,
    ) -> Result<()> {
        let rendered = self
            .registry
            .render(template, &data)
            .map_err(Error::HandlebarsError)?;

        write(path, rendered.as_str()).await.map_err(Error::Io)
    }
}

#[derive(Serialize)]
struct PageContext<'a> {
    dev_mode: bool,
    posts: &'a [FrontMatter],
}
