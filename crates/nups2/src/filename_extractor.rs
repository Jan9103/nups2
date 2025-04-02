use crate::pack2::Pack2;
use regex::Regex;
use std::collections::HashSet;
use std::{fs::File, io::Result};

const INTERRESTING_BYTES: &[u8] = &[
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, // A-Z
    97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, 120, 121, 122, // a-z
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, // 0-9
    0x2d, 0x5f, 0x2e, 0x3c, 0x3e, // - _ . < >
];

const SKIP_SIGNATURES_IN_FAST_MODE: &[&[u8]] = &[
    &[0x89, 0x50, 0x4E, 0x47],                   // png
    &[0x44, 0x44, 0x53, 0x20, 0x7c, 0x00],       // dds file (textures)
    &[0x43, 0x44, 0x54, 0x41, 0x01], // cdt file (use unknown, but seems to be filename-free)
    &[0x44, 0x4d, 0x4f, 0x44, 0x44, 0x04, 0x00], // dmv file (use unknown, but seems to be filename-free)
];

const FILENAME_REGEX_STRINGS: [&str; 5] = [
    r#"[A-Za-z0-9<>._-]+\.[a-zA-Z0-9]{2,5}"#,
    r#"[A-Za-z0-9<>._-]+\.[a-zA-Z0-9]{2,5}"#,
    // extensions, which might be present, but yield no results: cfx
    r#"[A-Za-z0-9<>._-]+\.(adr|agr|ags|apb|apx|bat|bmp|bin|cdt|cnk0|cnk1|cnk2|cnk3|cnk4|cnk5|crc|crt|cso|cur|dat|db|dds|def|dir|dll|dma|dme|dmv|dsk|dx11efb|dx11rsb|dx11ssb|eco|efb|exe|fbx|fsb|fx|fxh|fxd|fxo|gr2|gfx|gnf|i64|ind|ini|jpg|lst|lua|mrn|nsa|pak|pem|playerstudio|png|prsb|psd|pssb|swf|tga|thm|tome|ttf|txt|vnfo|wav|xlsx|xmd|xml|xrsb|xssb|zone)"#,
    r#"[A-Za-z0-9<>._-]+\.(?i)(adr|agr|ags|apb|apx|bat|bmp|bin|cdt|cnk0|cnk1|cnk2|cnk3|cnk4|cnk5|crc|crt|cso|cur|dat|db|dds|def|dir|dll|dma|dme|dmv|dsk|dx11efb|dx11rsb|dx11ssb|eco|efb|exe|fbx|fsb|fx|fxh|fxd|fxo|gr2|gfx|gnf|i64|ind|ini|jpg|lst|lua|mrn|nsa|pak|pem|playerstudio|png|prsb|psd|pssb|swf|tga|thm|tome|ttf|txt|vnfo|wav|xlsx|xmd|xml|xrsb|xssb|zone)"#,
    r#"[A-Za-z0-9<>._-]+\.(?i)(adr|agr|ags|apb|apx|bat|bmp|bin|cdt|cnk0|cnk1|cnk2|cnk3|cnk4|cnk5|crc|crt|cso|cur|dat|db|dds|def|dir|dll|dma|dme|dmv|dsk|dx11efb|dx11rsb|dx11ssb|eco|efb|exe|fbx|fsb|fx|fxh|fxd|fxo|gr2|gfx|gnf|i64|ind|ini|jpg|lst|lua|mrn|nsa|pak|pem|playerstudio|png|prsb|psd|pssb|swf|tga|thm|tome|ttf|txt|vnfo|wav|xlsx|xmd|xml|xrsb|xssb|zone)"#,
];

/// search_mode:
///   0: short-regex, all-files => many false positives
///   1: short-regex + anti-float, file-filter => many false positives
///   2: (192288) standard-regex, file-filter
///   3: (192320) standard-regex + ignorecase, file-filter => slower
///   4: standard-regex + ignorecase, all-files => slowest
pub fn extract_names(
    pack: &Pack2,
    br: &mut File,
    filesize_limit: u32,
    search_mode: usize,
    limit_to_files: Option<Vec<u64>>,
) -> Result<Vec<String>> {
    // TODO: it misses most / all .fsb files
    let filename_regex: Regex = Regex::new(FILENAME_REGEX_STRINGS[search_mode])
        .expect("Failed to compile filename_extractor filename_regex");

    let try_skip_binaries: bool = [1, 2, 3].contains(&search_mode);

    let mut output: Vec<String> = Vec::new();

    'asset_loop: for asset in pack.assets.iter() {
        if limit_to_files.is_some() && !limit_to_files.as_ref().unwrap().contains(&asset.name_hash)
        {
            continue 'asset_loop;
        }
        if asset.unzipped_length > filesize_limit || asset.data_length > (filesize_limit as u64) {
            continue 'asset_loop;
        }
        let asset_data: Vec<u8> = asset.extract_bytes(br)?;
        if try_skip_binaries {
            if [0x00, 0xFF].contains(asset_data.first().unwrap_or(&0x00u8)) {
                continue 'asset_loop;
            }
            for sig in SKIP_SIGNATURES_IN_FAST_MODE {
                if asset_data.starts_with(sig) {
                    continue 'asset_loop;
                }
            }
        }
        let texts: Vec<String> = find_text_patches(&asset_data);
        'text_loop: for text in texts {
            if text.len() < 5 || !text.contains(".") {
                continue 'text_loop;
            }
            output.append(
                &mut filename_regex
                    .find_iter(text.as_str())
                    .map(|i| -> String {
                        let s: &str = i.as_str();
                        String::from(s.strip_prefix(">").unwrap_or(s))
                    })
                    .flat_map(|s| {
                        if s.contains("<gender>") {
                            vec![
                                s.replace("<gender>", "Female"),
                                s.replace("<gender>", "Male"),
                            ]
                        } else {
                            vec![s]
                        }
                    })
                    .flat_map(|s| match s.to_lowercase().split(".").last() {
                        Some("efb") => vec![format!("{}dx11efb", s.split_at(s.len() - 3).0), s],
                        Some("nsa") => {
                            let tds = s.split_at(s.len() - 3).0;
                            let mut t: Vec<String> = Vec::new();
                            t.push(s.clone());
                            if let Some((name, hash)) = tds.rsplit_once("_") {
                                t.push(format!("{}.nsa", name));
                                let l = name.len();
                                if (l % 2) == 1 {
                                    let sn = name.split_at(l / 2).0;
                                    t.push(format!("{}.nsa", &sn));
                                    t.push(format!("{}_{}.nsa", &sn, &hash));
                                }
                            }
                            t
                        }
                        Some("dma") => {
                            if let Some(a) = s.strip_suffix("_Lod0.dma") {
                                vec![format!("{a}.adr"), s]
                            } else {
                                vec![s]
                            }
                        }
                        Some("cdt") => vec![format!("{}adr", s.split_at(s.len() - 3).0), s],
                        _ => vec![s],
                    })
                    .collect::<Vec<String>>(),
            );
        }
    }

    if search_mode == 1 {
        // remove floats (example: 1.234)
        let float_regex: Regex =
            Regex::new("^[0-9.]+$").expect("Failed to compile filename_extractor float_regex");
        output.retain(|i| !float_regex.is_match(i.as_str()));
    }

    Ok(output
        .into_iter()
        .collect::<HashSet<String>>()
        .into_iter()
        .collect::<Vec<String>>())
}

/// find text patches in binary data (code contains strings, 3d-models have metadata, etc)
fn find_text_patches(binary: &Vec<u8>) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();
    let mut buffer: String = String::new();

    for b in binary {
        if INTERRESTING_BYTES.contains(b) {
            buffer.push(*b as char);
        } else if !buffer.is_empty() {
            output.push(buffer);
            buffer = String::new();
        }
    }

    if !buffer.is_empty() {
        output.push(buffer);
    }
    output
}
