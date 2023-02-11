use crate::error::{Error, Result};
use glob::glob;
use handlebars::Handlebars;
use serde::Serialize;
use std::path::Path;
use tokio::fs::{self, write};

pub struct HandlebarsCompiler<'a> {
    registry: Handlebars<'a>,
}

impl<'a> HandlebarsCompiler<'a> {
    pub fn new() -> Self {
        let registry = Handlebars::new();
        Self { registry }
    }

    pub async fn add_partials(&mut self, pattern: &str) -> Result<()> {
        let partials = glob(pattern).map_err(Error::Pattern)?;

        for partial in partials {
            let partial = partial.map_err(Error::Glob)?;
            let name = partial.as_path().file_stem().unwrap().to_str().unwrap();
            let content = fs::read_to_string(&partial).await.map_err(Error::Io)?;

            log::info!("adding partial {}", name);
            self.registry.register_partial(name, content).unwrap();
        }

        Ok(())
    }

    pub async fn compile_all<P: AsRef<Path>>(&self, pattern: &str, output_path: P) -> Result<()> {
        let pages = glob(pattern).map_err(Error::Pattern)?;

        for page in pages {
            let page = page.map_err(Error::Glob)?;
            let file_name = page.file_name().unwrap();
            let mut path = output_path.as_ref().join(file_name);
            path.set_extension("html");

            log::info!("render {:?} -> {:?}", page, path);

            let contents = fs::read_to_string(page).await.map_err(Error::Io)?;
            let rendered = self
                .registry
                .render_template(contents.as_str(), &())
                .unwrap();

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
        let rendered = self.registry.render(template, &data).unwrap();

        write(path, rendered.as_str()).await.map_err(Error::Io)
    }
}
