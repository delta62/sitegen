# sitegen

Generate static website content

There are many like it, but this one is mine

## Usage

```
Usage: sitegen [OPTIONS] <COMMAND>

Arguments:
  <COMMAND>
          Possible values:
          - build: Build the site
          - clean: Remove previously built artifacts
          - serve: Start a development server and host the site locally

Options:
  -m, --mode <MODE>
          The mode to build the site in

          [default: development]

          Possible values:
          - development: Non-optimized build with devtools support
          - release:     Optimized build without any extra functionality

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Toolchain

This generator uses a pre-defined group of libraries to build content:

- Styles are compiled via [Sass](https://sass-lang.com/)
- HTML content is generated via [Handlebars](https://handlebarsjs.com/)
- Posts are parsed from [Markdown](https://www.markdownguide.org/)

JavaScript isn't currently supported.

## Configuration

Most options are controlled via a configuration file. Create a file named
`config.toml` in the root directory of your static site. The full config file is
documented here. All options are required unless specified otherwise. Globbing
is done by the [glob crate](https://docs.rs/glob/0.3.1/glob/); see there for
options.

```toml
[build]
# The directory where built files should be placed
out_dir = "dist"
# Where to search for page definitions
page_pattern = "pages/**/*.handlebars"
# Where to search for post content
post_pattern = "posts/**/*.md"
# Where to search for Handlebar partial files
partials_pattern = "partials/**/*.handlebars"
# Where to search for scss files
style_pattern = "styles/**/*.scss"
# Optional. Additional patterns to copy into the output directory
copy = [ "images/*.png", "fonts" ]

[http]
# The command to use for starting a static web server
command = "http-server"
# Optional. Arguments to pass to `command`.
args = [ "--cwd", "dist", "--port", "8080" ]

[watch]
# Paths to watch for changes when running `serve`
paths = [ "posts", "templates" ]
```

## CI/CD

This tool can built your site in either development or release mode. This can be
helpful for skipping drafts when publishing, or enabling extra features for
local development builds that you don't want published.

Since the same `build.out_dir` is used regardless of build mode, an extra file
is output named `sitegen_meta.toml`. It includes a `mode` key which will either
be set to `development` or `release`. You can use this in your deployment
scripts to avoid accidentally publishing a dev build.
