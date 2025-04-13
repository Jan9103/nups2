# nups2

A decoder for planetside2 files (mainly `pack2` for now):
* As a crablang library (`crates/nups2`)
* As a basic cli (`crates/nups2`)
* As a [nushell][] plugin (`crates/nushell_plugin_nups2`)

**NOTE:** If you just want to explore some files you can get pre-extracted ones on the discord
and would probably have a better time with [ps2ls2][].

Currently the only "upsides" compared to `ps2ls2` are probably that this is not a GUI and that the
filename-scraper finds diffrent results.

## Features

### `nu_plugin_nups2` (nushell plugin)

* `pack2`:
  * list contents
  * extract files
  * scrape filenames
* `hash ps2crc64`

### `nups2` (cli)

* `pack2`:
  * list contents
    * human formatting
    * json
  * extract files
  * scrape filenames
  * manifests (for `diff` between updates, etc)
* `pack1`:
  * list contents
  * extract files
  * convert pack2 to pack1

### `nups2` (library)

* `pack2` files
  * read filelist
  * extract files
  * determine filenames
    * read embedded filename index (if present)
    * scrape `pack2` files to build a filename list
    * use external filename lists
    * generate and use [rainbow-table][]s for filename decoding
  * generate a manifest from `pack2` files and compare those with newer version of the same `pack2` file.
* `dma` files
  * read (note: larger than RAM files are not yet supported)
  * convert to json
* `dme` (v4) files
  * read (note: larger than RAM files are not yet supported)
* `pack1` files
  * read filelist
  * extract files
  * convert pack2 to pack1


## usage

* **Nu plugin:** [crates/nu_plugin_nups2/README.md](crates/nu_plugin_nups2/README.md)
* **CLI:** For most things just run `nups2 --help` and then `nups2 <subcommand> --help`. it should be pretty self-explanatory.

### getting proper filenames

`pack2` files store files without their real name. There is support for embedding a filename list inside
of the files, but most official files do not.

There are multiple options on how to do this:

#### (recommended) builtin filename scraper

Nups has a tool builtin, which looks for things looking like filenames in the contents of a pack2 file.

Filenames are not necesarely contained in the same file as the file it applies to.  
Therefore it is recommended to once build a global index and use that one for everything.
It is not complete (`Sanctuary_x64_0.pack2` is still basically untranslated, etc), but a improvement over no translation.

Be ready to wait for an hour!

```bash
# (cli) build a index from 1 file
nups2 pack2-scrape-filenames data_x64_0.pack2 namelist.txt
# (nu_plugin) build a index from 1 file
pack2 scrape_filenames data_x64_0.pack2 namelist.txt

# (cli) build a index from all files
nu ./scrape_all_for_namelist.nu

# (cli) use a index
nups2 pack2-ls --filename-list-file namelist.txt VR_x64_0.pack2
# (nu_plugin) use a index
pack2 ls --filename_list_file namelist.txt VR_x64_0.pack2
```

The scraper has multiple modes (`--scrape-mode`) (3 is recommended):

mode | results | false-positives | speed     | mode of operation
---- | ------- | --------------- | --------- | -----------------
0    | many    | to many         | slow      | simple-regex, all data
1    | many    | to many         | fast      | simple-regex + float-filter, filtered data
2    | average | few             | ok        | standard-regex, filtered data
3    | more than 2 | balanced    | slow      | standard-regex + ignorecase, filtered data
4    | more than 3 | more than 3 | slowest   | standard-regex + ignorecase, all data

I would recommend (time measured on a `ryzen5 1600x` with all files):
* mode 1 or 2 if you just want to explore a few random files of your favorite game (~30min).
  * You can save some more time by just scraping and extracting the `asset` files.
* mode 3 or 4 if you're in a hurry, but want a fairly complete experience (~1h).
* mode 4 + 0 (2 runs and then use `cat` on the results) if you want as much as possible (~2h).

If you are planning on re-scraping after the next update you can create a manifest for each `pack2` file now.  
After the next game-update you can then pass `--manifest-from-last-scrape manifest_file.bin` to the next scrape
to only scrape the files which changed and then merge the new scrape file with the old one.


#### external filename list

If you have found a up-to-date list of all filenames you can just attach them with the `--filename-list-file your_file.txt`.  
I would still recommend to use the filename scraper and then combind the files into one for better results.


#### builtin rainbow tables

nups2 has a rainbow-table feature builtin.  
However this is extremely inefficient.  
If we generate a wordist based on all the parts of known filenames and then use those to generate
a rainbow-table, which could decode all of them, we end up with a ~2500 zetabyte big cache-file.

If you have a good idea of which words might be used in a specific `pack2` file you can enable the
feature at compiletime with `-F rainbow_table`, generate a table with `nups2 rainbowtable-build`,
and then add `--rainbow-table-file PATH-TO-GENERATED-FILE` to all your `pack2` commands.

#### hashcat

The password-cracking tool [hashcat][] can be used to crack the filenames.  
But this is extremely time-consuming.

```bash
# (cli)
nups2 pack2-ls --json ./example.pack2 | jq ".[].name_hash" > hashes.txt
# (nu_plugin)
pack2 ls ./example.pack2 | get name_hash | str join "\n" | save hashes.txt

hashcat -m 28000 hashes.txt
```

It might also be a good idea to create a file of all hashes of all pack2 files and use that instead
of running hashcat once per pack2 file.

#### forgelight-toolbox filename scraper

The [forgelight-toolbox][] includes a tool for searching for potential filenames in
files included within the `pack2` file.  
It is a diffrent implementation and therefore might yield diffrent results from the builtin scraper.


## Credits

I only implemented this. All the research into the file-formats, etc came from other projects.  
I used the following peoples work for understanding these things:
* [psemu][]
* [NatCracken][]
* [RhettVX][]
* [ryanjsims][]

I used some external libraries in this implementation:
* [clap][] for parsing the CLI arguments.
* [comfy-table][] for generating ASCII-art tables.
* [flate2][] for zlib decompression.
* [rayon][] for multithreading (only for [rainbow-table][] features so far).
* [regex][] as regex engine.
* [quick-xml][] and [serde][] for `.adr` files.
* [nushell][]
* [log][] and [env_logger][] for logging.

## Legal stuff

This project is not affiliated with or endorsed by any company. This includes but is not limited to Sony Online Entertainment, the Rust Foundation, and Microsoft.  
You may only use the content extracted by this tool if you hold a valid license from the respective copyright holders.  
No copyrighted assets are provided by or on behalf of this project. Users are responsible for sourcing such content independently.  
Please be mindful of intellectual property rights and ensure that all usage complies with applicable laws and regulations.



[NatCracken]: https://github.com/NatCracken
[RhettVX]: https://github.com/RhettVX
[clap]: https://crates.io/crates/clap
[comfy-table]: https://crates.io/crates/comfy-table
[flate2]: https://crates.io/crates/flate2
[forgelight-toolbox]: https://github.com/RhettVX/forgelight-toolbox
[hashcat]: https://hashcat.net
[nushell]: https://nushell.sh
[ps2ls2]: https://github.com/NatCracken/ps2ls2
[psemu]: https://github.com/psemu
[quick-xml]: https://crates.io/crates/quick-xml
[rainbow-table]: https://en.wikipedia.org/wiki/Rainbow_table
[rayon]: https://crates.io/crates/rayon
[regex]: https://crates.io/crates/regex
[ryanjsims]: https://github.com/ryanjsims
[serde]: https://crates.io/crates/serde
[log]: https://crates.io/crates/log
[env_logger]: https://crates.io/crates/env_logger
