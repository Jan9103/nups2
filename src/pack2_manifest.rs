use std::collections::HashMap;
use std::{fs::File, io::ErrorKind, io::Result, path::Path};

use crate::bin_utils::{read_u32_be, read_u64_be};

/// Manifest file format:
///   12bytes per asset, no seperator:
///     8bits: u64_BE filename_hash (from the pack2 index)
///     4bits: u32_BE data_hash (from the pack2 index)
pub fn read_manifest_file(manifest_file: &Path) -> Result<Manifest> {
    let mut br: File = File::open(manifest_file)?;
    let mut manifest_assets: Manifest = Vec::new();
    'asset_loop: loop {
        match read_u64_be(&mut br) {
            Ok(name_hash) => {
                let data_hash: u32 = read_u32_be(&mut br)?;
                manifest_assets.push((name_hash, data_hash));
            }
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                break 'asset_loop;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(manifest_assets)
}

pub fn render_for_humans(
    manifest_diff: &ManifestDiff,
    name_hash_lookup_table: &HashMap<u64, String>,
) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut already_included: Vec<u64> = Vec::new();
    'diff_entry_loop: for d in manifest_diff {
        if d.new_data_hash.is_none() {
            continue;
        }
        if d.old_data_hash.is_none() {
            for i in manifest_diff {
                if i.old_data_hash.eq(&Some(d.new_data_hash.unwrap())) {
                    already_included.push(i.name_hash);
                    out.push(format!(
                        "Renamed file: 0x{:X} ({}) -> 0x{:X} ({})",
                        i.name_hash,
                        name_hash_lookup_table
                            .get(&i.name_hash)
                            .unwrap_or(&String::from("?")),
                        d.name_hash,
                        name_hash_lookup_table
                            .get(&d.name_hash)
                            .unwrap_or(&String::from("?")),
                    ));
                    continue 'diff_entry_loop;
                }
            }
            out.push(format!(
                "Created file: 0x{:X} ({}) with data_hash 0x{:X}",
                d.name_hash,
                name_hash_lookup_table
                    .get(&d.name_hash)
                    .unwrap_or(&String::from("?")),
                d.new_data_hash.unwrap()
            ));
            continue 'diff_entry_loop;
        }
        out.push(format!(
            "Changed file: 0x{:X} ({}): 0x{:X} -> 0x{:X}",
            d.name_hash,
            name_hash_lookup_table
                .get(&d.name_hash)
                .unwrap_or(&String::from("?")),
            d.old_data_hash.unwrap(),
            d.new_data_hash.unwrap(),
        ));
    }
    for d in manifest_diff {
        if d.new_data_hash.is_some() || already_included.contains(&d.name_hash) {
            continue;
        }
        out.push(format!(
            "Deleted file: 0x{:X} ({}) with data_hash 0x{:X}",
            d.name_hash,
            name_hash_lookup_table
                .get(&d.name_hash)
                .unwrap_or(&String::from("?")),
            d.old_data_hash.unwrap(),
        ));
    }
    out.join("\n")
}

pub type Manifest = Vec<(u64, u32)>;
pub type ManifestDiff = Vec<ManifestDiffEntry>;

pub struct ManifestDiffEntry {
    pub name_hash: u64,
    pub old_data_hash: Option<u32>,
    pub new_data_hash: Option<u32>,
}

impl ManifestDiffEntry {
    pub fn new(name_hash: u64, old_data_hash: Option<u32>, new_data_hash: Option<u32>) -> Self {
        Self {
            name_hash,
            old_data_hash,
            new_data_hash,
        }
    }
}
