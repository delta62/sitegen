use crate::error::{Error, Result};
use glob::glob;
use handlebars::Handlebars;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct HandlebarsCompiler<'a> {
    registry: Handlebars<'a>,
}

impl<'a> HandlebarsCompiler<'a> {
    pub fn new() -> Self {
        let registry = Handlebars::new();
        Self { registry }
    }

    pub fn add_partials(&mut self, pattern: &str) -> Result<()> {
        let partials = glob(pattern).map_err(Error::Pattern)?;

        for partial in partials {
            let partial = partial.map_err(Error::Glob)?;
            let name = partial.as_path().file_stem().unwrap().to_str().unwrap();
            let content = fs::read_to_string(&partial).map_err(Error::Io)?;

            log::info!("adding partial {}", name);
            self.registry.register_partial(name, content).unwrap();
        }

        Ok(())
    }

    pub fn compile_all<P: AsRef<Path>>(&self, pattern: &str, output_path: P) -> Result<()> {
        let pages = glob(pattern).map_err(Error::Pattern)?;

        for page in pages {
            let page = page.map_err(Error::Glob)?;
            let file_name = page.file_name().unwrap();
            let mut path = output_path.as_ref().join(file_name);
            path.set_extension("html");

            log::info!("render {:?} -> {:?}", page, path);

            let contents = fs::read_to_string(page).map_err(Error::Io)?;
            let file = File::create(&path).unwrap();
            let file = BufWriter::new(file);
            self.registry
                .render_template_to_write(contents.as_str(), &(), file)
                .unwrap();
        }

        Ok(())
    }

    pub fn render_to_write<S: Serialize, W: Write>(&self, template: &str, data: S, writer: W) {
        self.registry
            .render_to_write(template, &data, writer)
            .unwrap()
    }
}
