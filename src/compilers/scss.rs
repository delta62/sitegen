use crate::error::{Error, Result};
use glob::glob;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct CompilerOptions<'a> {
    pub input_pattern: &'a str,
    pub output_path: &'a str,
}

pub struct SassCompiler;

impl SassCompiler {
    pub fn compile(opts: &CompilerOptions) -> Result<()> {
        let compiler_opts = Default::default();

        let stylesheets = glob(opts.input_pattern).map_err(Error::Pattern)?;
        for stylesheet in stylesheets {
            let stylesheet = stylesheet.map_err(Error::Glob)?;
            let rendered = grass::from_path(stylesheet.as_path(), &compiler_opts)
                .map_err(|e| Error::Compiler(Box::new(e)))?;

            let file_name = stylesheet.file_name().unwrap();
            let mut path = Path::new(opts.output_path).join(file_name);
            path.set_extension("css");

            let file = File::create(&path).map_err(Error::Io)?;
            let mut file = BufWriter::new(file);

            file.write_all(rendered.as_bytes()).map_err(Error::Io)?;
        }

        Ok(())
    }
}
