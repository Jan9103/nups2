pub mod search {
    use regex::Regex;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::{io::Result, path::Path};

    pub fn search_table(rainbow_table_file: &Path, hashes: &Vec<u64>) -> Result<Vec<String>> {
        let file: File = File::open(rainbow_table_file)?;
        let br: BufReader<File> = BufReader::new(file);

        let mut searched: Vec<Regex> = hashes
            .iter()
            .map(|i| Regex::new(format!("^{i} ").as_str()).unwrap())
            .collect();
        let mut results: Vec<String> = Vec::new();

        for line in br.lines() {
            let tmp: String = line?;
            let line: &str = tmp.as_str();
            let mut i: usize = 0;
            while i < searched.len() {
                let hash: &Regex = &searched[i];
                if hash.is_match(line) {
                    results.push(line.split_once(" ").unwrap().1.into());
                    searched.remove(i);
                } else {
                    i += 1;
                }
            }
        }

        Ok(results)
    }
}

pub mod build {
    use crate::crc64;
    use rayon::prelude::*;
    use std::io::{BufRead, BufReader, Result, Write};
    use std::{fs::File, path::Path};

    pub fn build_table(
        rainbow_table_words: &Vec<String>,
        file_extensions: &Vec<String>,
        max_words: usize,
        save_file: &Path,
    ) -> Result<()> {
        assert!(max_words >= 1, "Can't generate table for <= 0 words.");

        let mut br: File = File::create_new(save_file)?;
        for word_count in 1..=max_words {
            recursive_loop_build(
                &String::new(),
                rainbow_table_words,
                file_extensions,
                word_count,
                &mut br,
            )?;
            log::info!("Finished generating for word_count={}", word_count);
        }
        Ok(())
    }

    fn recursive_loop_build(
        v: &String,
        rainbow_table_words: &Vec<String>,
        file_extensions: &Vec<String>,
        subloops: usize,
        br: &mut File,
    ) -> Result<()> {
        let wordlist_length = rainbow_table_words.len();
        for i in 0..wordlist_length {
            let v2: String = (String::new() + v) + rainbow_table_words[i].as_str();
            if subloops == 0 {
                recursive_loop_build_ext(&v2, file_extensions, br)?;
            } else {
                recursive_loop_build(&v2, rainbow_table_words, file_extensions, subloops - 1, br)?;
            }
        }
        Ok(())
    }

    fn recursive_loop_build_ext(
        v: &String,
        file_extensions: &Vec<String>,
        br: &mut File,
    ) -> Result<()> {
        let result: String = file_extensions
            .par_iter()
            .map(|i| -> String {
                let v2: String = (String::new() + v) + "." + i.as_str();
                let hash: u64 = crc64::hash(v2.as_bytes());
                format!("{hash} {v2}\n")
            })
            .collect::<Vec<String>>()
            .join("");
        br.write(result.as_bytes())?;
        Ok(())
    }

    pub fn convert_filename_list_to_rainbow_table_format(
        filename_list_file: &Path,
        save_file: &Path,
    ) -> Result<()> {
        let filenames_file: File = File::open(filename_list_file)?;
        let filenames_br: BufReader<File> = BufReader::new(filenames_file);
        let mut br: File = File::create_new(save_file)?;
        for file_name in filenames_br.lines() {
            let file_name: String = file_name?;
            br.write(
                format!("{} {}\n", crc64::convert_filename(&file_name), &file_name).as_bytes(),
            )?;
        }
        Ok(())
    }
}
