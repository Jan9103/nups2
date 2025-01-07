# nups2 (cli/lib)

This is the cli-interface and library part of nups2

## Feature flags

This makes heavy use of cargo feature flags.  
The minimal build for listing and extracting `pack2` file contents can be compiled a lot faster than one with all features.

### Meta feature flags

Feature flag | Description
------------ | -----------
`default`    | Everything needed for a good experience with the cli tool (only `pack2` and nothing else)
`all`        | All features you might want (excludes `debug_logs`)
`debug_logs` | Print debug information to `stderr` constantly (as the name suggests only intended for debugging)

### General

Feature flag | Description
------------ | -----------
`json`       | Add `--json` to the cli and `to_json()` to the library
`use_comfy_table` | Make use of the `comfy_table` library to make the CLI output look good


### File format related

**Pack2**

Feature flag | Description
------------ | -----------
`manifests`  | Add manifest functionality (generating a fingerprint and later showing what has changed)
`filename_scraper` | Scrape filenames from `pack2` contents (both cli and library)
`rainbow_table` | Rainbow table generator for `pack2` filenames (not recommended unless you know what you are doing and think its a good idea)

**extra fileformat support** (library only)

Feature flag | Description
------------ | -----------
`dma`        | `.dma` file support (materials) (mostly untested)
`dme`        | `.dme` file support (meshes and bones) (mostly untested)
`adr`        | `.adr` file support (actor definitions) (DOES NOT WORK)


## Usage as library

I would recommend to `default-features = false` and manually import what you need.

Use `rust-analyzer`. As starting point use `nups2::pack2::Pack2::load_from_file` and `apply_filename_list`.

## Use as cli-tool

The default features should be good enough for 99% of usecases.  
Just run `cargo build --release` and grab your binary from `target/release/nups2`.  
Run `nups2 --help` for a list of sub-commands and then `nups2 SUBCOMMAND --help` for further infos.
