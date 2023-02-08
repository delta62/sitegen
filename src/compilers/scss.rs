use crate::error::{Error, Result};
use glob::glob;
use std::fs::{self, File};
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

        let stylesheets = glob(opts.input_pattern).map_err(Error::PatternError)?;
        for stylesheet in stylesheets {
            let stylesheet = stylesheet.map_err(Error::GlobError)?;
            let rendered = grass::from_path(stylesheet.as_path(), &compiler_opts)
                .map_err(|e| Error::CompilerError(Box::new(e)))?;

            let mut path = Path::new(opts.output_path).join(stylesheet.as_path());
            path.set_extension("css");
            fs::create_dir_all(path.parent().unwrap()).map_err(Error::IoError)?;

            let file = File::create(&path).map_err(Error::IoError)?;
            let mut file = BufWriter::new(file);

            file.write_all(rendered.as_bytes())
                .map_err(Error::IoError)?;
        }

        Ok(())
    }
}
