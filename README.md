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
