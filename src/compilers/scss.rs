use crate::error::{Error, Result};
use glob::glob;
use grass::Options;
use std::path::Path;
use tokio::fs::write;

pub struct CompilerOptions<'a> {
    pub input_pattern: &'a str,
    pub output_path: &'a str,
}

pub struct SassCompiler<'a> {
    compiler_options: Options<'a>,
    options: CompilerOptions<'a>,
}

impl<'a> SassCompiler<'a> {
    pub fn new(options: CompilerOptions<'a>) -> Self {
        let compiler_options = Default::default();
        Self {
            compiler_options,
            options,
        }
    }

    pub async fn compile(&self) -> Result<()> {
        let stylesheets = glob(self.options.input_pattern).map_err(Error::Pattern)?;
        for stylesheet in stylesheets {
            let stylesheet = stylesheet.map_err(Error::Glob)?;
            let rendered = grass::from_path(stylesheet.as_path(), &self.compiler_options)
                .map_err(|e| Error::Compiler(Box::new(e)))?;

            let file_name = stylesheet.file_name().unwrap();
            let mut path = Path::new(self.options.output_path).join(file_name);
            path.set_extension("css");

            write(&path, rendered.as_bytes()).await.map_err(Error::Io)?;
        }

        Ok(())
    }
}
