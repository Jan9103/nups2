use clap::Parser;
use clap::Subcommand;
use std::fs::File;
#[allow(unused_imports)]
// for some reason rustc thinks this is unused, but removing it is and dosn't compile without
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
#[allow(unused_imports)]
// not used with "--no-defualt-features", but id rather always have it in compile scope to avoid forgetting to add a faeture flaag thing
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

use crate::pack2::Pack2;

pub fn cli() -> std::io::Result<()> {
    // dbg!(crate::dma::Dma::read(&mut File::open(
    //     "extracted/assets_x64_0/Weapon_NC_PistolMag_Lod2.dma"
    // )?)?);
    // crate::dme::Dme::read(&mut File::open(
    //     "extracted/assets_x64_0/Amerish_Props_Rock_HighlandFormation04_Lod0.dme", // "extracted/assets_x64_0/Indar_Flora_Shrub_OceanGreen03_Lod0_LODAuto.dme"
    // )?)?;
    // return Ok(());

    let args = Args::parse();
    match args.cmd {
        Commands::Pack2Ls {
            pack2_file,
            filename_list_file,
            #[cfg(feature = "json")]
            json,
            #[cfg(feature = "rainbow_table")]
            rainbow_table_file,
        } => {
            let mut br: File = File::open(pack2_file)?;
            #[allow(unused_mut, unused_variables)]
            let mut pack2: Pack2 = Pack2::load_from_file(&mut br)?;
            #[cfg(feature = "rainbow_table")]
            if let Some(rtf) = rainbow_table_file {
                pack2.crack_names_with_rainbow_table(rtf.as_path())?;
            }
            if let Some(tmp) = filename_list_file {
                pack2.apply_filename_list(&read_file_lines(&tmp)?);
            }
            #[cfg(feature = "json")]
            if json {
                println!("{}", pack2.ls_assets_as_json());
                return Ok(());
            }
            println!("{}", pack2.ls_assets_for_humans());
        }
        Commands::Pack2Extract {
            pack2_file,
            files_to_extract,
            output_dir,
            filename_list_file,
            #[cfg(feature = "rainbow_table")]
            rainbow_table_file,
        } => {
            let mut br: File = File::open(pack2_file)?;
            #[allow(unused_mut)]
            let mut pack2: Pack2 = Pack2::load_from_file(&mut br)?;
            #[cfg(feature = "rainbow_table")]
            if let Some(rtf) = rainbow_table_file {
                pack2.crack_names_with_rainbow_table(rtf.as_path())?;
            }
            if let Some(tmp) = filename_list_file {
                pack2.apply_filename_list(&read_file_lines(&tmp)?);
            }
            for file in files_to_extract {
                pack2.extract_file(&mut br, file, output_dir.as_path())?;
            }
        }

        Commands::Pack2ExtractAll {
            pack2_file,
            exclude_named,
            exclude_unnamed,
            output_dir,
            filename_list_file,
            #[cfg(feature = "rainbow_table")]
            rainbow_table_file,
        } => {
            let mut br: File = File::open(pack2_file)?;
            #[allow(unused_mut)]
            let mut pack2: Pack2 = Pack2::load_from_file(&mut br)?;
            #[cfg(feature = "rainbow_table")]
            if let Some(rtf) = rainbow_table_file {
                pack2.crack_names_with_rainbow_table(rtf.as_path())?;
            }
            if let Some(tmp) = filename_list_file {
                pack2.apply_filename_list(&read_file_lines(&tmp)?);
            }
            if !exclude_named {
                pack2.extract_all_named(&mut br, output_dir.as_path())?;
            }
            if !exclude_unnamed {
                pack2.extract_all_unnamed(&mut br, output_dir.as_path())?;
            }
        }

        #[cfg(feature = "filename_scraper")]
        Commands::Pack2ScrapeFilenames {
            pack2_file,
            output_file,
            filesize_limit,
            scrape_mode,
            #[cfg(feature = "manifests")]
            manifest_from_last_scrape,
        } => {
            if scrape_mode > 4 {
                eprintln!("scrape_mode has to be between 0, 1, 2, 3, or 4");
                exit(1);
            }
            let mut output_file = File::create_new(output_file)?;
            let mut br: File = File::open(pack2_file)?;
            let pack2: Pack2 = Pack2::load_from_file(&mut br)?;
            #[allow(unused_mut)]
            let mut limit_to_files: Option<Vec<u64>> = None;

            #[cfg(feature = "manifests")]
            if let Some(m) = manifest_from_last_scrape {
                use crate::pack2_manifest::*;
                let manifest: Manifest = read_manifest_file(m.as_path())?;
                let diff: ManifestDiff = pack2.diff_with_manifest(&manifest);
                limit_to_files = Some(diff.iter().map(|i| i.name_hash).collect());
            }

            let filenames: Vec<String> = crate::filename_extractor::extract_names(
                &pack2,
                &mut br,
                filesize_limit,
                scrape_mode,
                limit_to_files,
            )?;
            output_file.write_all(filenames.join("\n").as_bytes())?;
        }

        #[cfg(feature = "manifests")]
        Commands::Pack2GenerateManifest {
            pack2_file,
            output_file,
        } => {
            if output_file.exists() {
                eprintln!("ERROR: Output file already exists");
                exit(1);
            }
            let mut br: File = File::open(pack2_file)?;
            let pack2: Pack2 = Pack2::load_from_file(&mut br)?;
            pack2.write_manifest_file(output_file.as_path())?;
        }

        #[cfg(feature = "manifests")]
        Commands::Pack2DiffWithManifest {
            pack2_file,
            filename_list_file,
            manifest_file,
        } => {
            use crate::pack2_manifest::*;
            let filename_list: Vec<String> = if let Some(fnlf) = filename_list_file {
                read_file_lines(&fnlf)?
            } else {
                Vec::new()
            };
            let mut br: File = File::open(pack2_file)?;
            let pack2: Pack2 = Pack2::load_from_file(&mut br)?;
            let manifest: Manifest = read_manifest_file(manifest_file.as_path())?;
            let diff: ManifestDiff = pack2.diff_with_manifest(&manifest);
            println!(
                "{}",
                render_for_humans(
                    &diff,
                    &crate::crc64::filename_list_to_lookup_table(&filename_list),
                )
            );
        }

        #[cfg(feature = "manifests")]
        Commands::ManifestDiffWithAnother {
            new_manifest_file,
            old_manifest_file,
            filename_list_file,
        } => {
            use crate::pack2_manifest::*;
            let filename_list: Vec<String> = if let Some(fnlf) = filename_list_file {
                read_file_lines(&fnlf)?
            } else {
                Vec::new()
            };
            let old_manifest: Manifest = read_manifest_file(old_manifest_file.as_path())?;
            let new_manifest: Manifest = read_manifest_file(new_manifest_file.as_path())?;
            let diff: ManifestDiff = diff_two_manifests(&old_manifest, &new_manifest);
            println!(
                "{}",
                render_for_humans(
                    &diff,
                    &crate::crc64::filename_list_to_lookup_table(&filename_list),
                )
            )
        }

        #[cfg(feature = "pack1")]
        Commands::Pack1Ls {
            pack1_file,
            #[cfg(feature = "json")]
            json,
        } => {
            use crate::pack1::Pack1;
            let mut br: File = File::open(pack1_file)?;
            let pack1: Pack1 = Pack1::load_from_file(&mut br)?;
            #[cfg(feature = "json")]
            if json {
                println!("{}", pack1.as_json());
                return Ok(());
            }
            println!("{}", pack1.ls_for_humans());
        }

        #[cfg(feature = "rainbow_table")]
        Commands::RainbowtableBuild {
            wordlist_file,
            extlist_file,
            output_file,
            max_word_count,
        } => {
            let wordlist: Vec<String> = read_file_lines(&wordlist_file)?;
            let extlist: Vec<String> = read_file_lines(&extlist_file)?;
            crate::rainbow_table::build::build_table(
                &wordlist,
                &extlist,
                max_word_count,
                output_file.as_path(),
            )?;
        }

        #[cfg(feature = "rainbow_table")]
        Commands::RainbowtableFromFilenames {
            filename_list_file,
            output_file,
        } => {
            crate::rainbow_table::build::convert_filename_list_to_rainbow_table_format(
                filename_list_file.as_path(),
                output_file.as_path(),
            )?;
        }
    }

    Ok(())
}

fn read_file(path: &PathBuf) -> std::io::Result<String> {
    let f: File = File::open(path)?;
    let mut b: BufReader<File> = BufReader::new(f);
    let mut s: String = String::new();
    b.read_to_string(&mut s)?;
    Ok(s)
}

fn read_file_lines(path: &PathBuf) -> std::io::Result<Vec<String>> {
    Ok(read_file(path)?
        .lines()
        .map(|i| i.into())
        .collect::<Vec<String>>())
}

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List the files contained in a pack2 file
    Pack2Ls {
        /// The pack2 file you want to inspect
        pack2_file: PathBuf,

        /// Output the data as json for further use by other programs
        #[cfg(feature = "json")]
        #[clap(long, action)]
        json: bool,

        /// Path to a file containing a newline-seperated list of filenames (for example from pack2-scrape-filenames)
        #[clap(long)]
        filename_list_file: Option<PathBuf>,

        /// Rainbow-table file to use for decrypting names
        /// You can generate one via rainbowtable-build
        #[cfg(feature = "rainbow_table")]
        #[clap(long)]
        rainbow_table_file: Option<PathBuf>,
    },
    /// Extract specific files from a pack2 file
    Pack2Extract {
        pack2_file: PathBuf,

        files_to_extract: Vec<String>,

        /// Into which dircetory should the extracted files be put?
        #[clap(long, default_value = ".")]
        output_dir: PathBuf,

        /// Path to a file containing a newline-seperated list of filenames (for example from pack2-scrape-filenames)
        #[clap(long)]
        filename_list_file: Option<PathBuf>,

        /// Rainbow-table file to use for decrypting names
        /// You can generate one via rainbowtable-build
        #[cfg(feature = "rainbow_table")]
        #[clap(long)]
        rainbow_table_file: Option<PathBuf>,
    },
    /// Extract all files from a pack2 file
    Pack2ExtractAll {
        pack2_file: PathBuf,

        /// do not extract files with a known name
        #[clap(long, action)]
        exclude_named: bool,

        /// do not extract files with no known name
        #[clap(long, action)]
        exclude_unnamed: bool,

        /// Into which dircetory should the extracted files be put?
        #[clap(long, default_value = ".")]
        output_dir: PathBuf,

        /// Path to a file containing a newline-seperated list of filenames (for example from pack2-scrape-filenames)
        #[clap(long)]
        filename_list_file: Option<PathBuf>,

        /// Rainbow-table file to use for decrypting names
        /// You can generate one via rainbowtable-build
        #[cfg(feature = "rainbow_table")]
        #[clap(long)]
        rainbow_table_file: Option<PathBuf>,
    },

    #[cfg(feature = "filename_scraper")]
    /// scrape the contents of a pack2 file for things, which look like filenames
    Pack2ScrapeFilenames {
        pack2_file: PathBuf,

        /// where to store the found filenames
        /// the file is a newline-seperated list
        output_file: PathBuf,

        #[clap(long, default_value_t = 256u32 * 1024 * 1024)]
        filesize_limit: u32,

        #[clap(long, default_value_t = 3)]
        scrape_mode: usize,

        #[cfg(feature = "manifests")]
        #[clap(long)]
        manifest_from_last_scrape: Option<PathBuf>,
    },

    #[cfg(feature = "manifests")]
    Pack2GenerateManifest {
        pack2_file: PathBuf,
        output_file: PathBuf,
    },

    #[cfg(feature = "manifests")]
    Pack2DiffWithManifest {
        pack2_file: PathBuf,

        manifest_file: PathBuf,

        #[clap(long)]
        filename_list_file: Option<PathBuf>,
    },

    #[cfg(feature = "manifests")]
    ManifestDiffWithAnother {
        new_manifest_file: PathBuf,
        old_manifest_file: PathBuf,

        #[clap(long)]
        filename_list_file: Option<PathBuf>,
    },

    #[cfg(feature = "pack1")]
    Pack1Ls {
        /// The pack2 file you want to inspect
        pack1_file: PathBuf,

        /// Output the data as json for further use by other programs
        #[cfg(feature = "json")]
        #[clap(long, action)]
        json: bool,
    },

    #[cfg(feature = "rainbow_table")]
    RainbowtableBuild {
        /// Path to file containing a list of words (1 word per line)
        /// NOTE: it is recommended to write the words with capslock on, since daybreaks hashing algorhytm seems to convert names to uppercase per default.
        wordlist_file: PathBuf,

        /// Path to file containing a list of file-extensions (1 per line)
        /// NOTE: it is recommended to write the words with capslock on, since daybreaks hashing algorhytm seems to convert names to uppercase per default.
        /// Extensions im aware of: WAV, PNG, JPG, JPEG, TGA, FBX, OBJ, TXT, XLSX, ZIP, XML
        extlist_file: PathBuf,

        /// Path to file in which the table should get stored
        output_file: PathBuf,

        /// How many tokens/words can 1 filename contain (tokens, not characters)
        /// The longest filename im aware of is 165 characters. the tokencount depends on your wordlist.
        #[clap(long, default_value_t = 10usize)]
        max_word_count: usize,
    },

    #[cfg(feature = "rainbow_table")]
    RainbowtableFromFilenames {
        filename_list_file: PathBuf,
        output_file: PathBuf,
    },
}
