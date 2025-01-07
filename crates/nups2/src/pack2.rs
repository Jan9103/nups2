use crate::bin_utils::*;
use crate::cli_utils::humanise_bytes;
use crate::crc64;
use flate2::read::ZlibDecoder;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::{prelude::*, Result, SeekFrom};
use std::path::Path;

const ZIPPED_FLAGS: [u32; 2] = [0x01, 0x11];
#[allow(dead_code)]
const UNZIPPED_FLAGS: [u32; 2] = [0x00, 0x10];

#[allow(dead_code)]
#[derive(Debug)]
pub struct Pack2 {
    asset_count: u32,
    length: u64,
    map_offset: u64,
    pub assets: Vec<Asset>,
}

impl Pack2 {
    #[cfg(feature = "json")]
    pub fn to_json(&self) -> String {
        format!(
            "{o}\"asset_count\": {asset_count}, \"length\": {length}, \"map_offset\": {map_offset}, \"assets\": {assets}{c}",
            o = "{",
            c = "}",
            asset_count = self.asset_count.to_string(),
            length = self.length.to_string(),
            map_offset = self.map_offset.to_string(),
            assets = self.ls_assets_as_json(),
        )
    }

    #[cfg(feature = "rainbow_table")]
    pub fn crack_names_with_rainbow_table(&mut self, rainbow_table_file: &Path) -> Result<()> {
        let hashes_to_crack: Vec<u64> = self
            .assets
            .iter()
            .filter(|i| i.name.is_none())
            .map(|i| i.name_hash.clone())
            .collect();
        let words: Vec<String> =
            crate::rainbow_table::search::search_table(rainbow_table_file, &hashes_to_crack)?;
        self.apply_filename_list(&words);
        Ok(())
    }

    pub fn apply_filename_lookup_table(&mut self, filename_lookup_table: &HashMap<u64, String>) {
        for asset in self.assets.iter_mut() {
            match filename_lookup_table.get(&asset.name_hash) {
                Some(name) => asset.name = Some(name.clone()),
                None => (),
            }
        }
    }

    pub fn apply_filename_list(&mut self, filename_list: &Vec<String>) {
        self.apply_filename_lookup_table(&crc64::filename_list_to_lookup_table(filename_list));
    }

    pub fn load_from_file(br: &mut File) -> Result<Self> {
        // let mut br: File = File::open(file_path.as_str())?;
        // let mut br: BufReader<File> = BufReader::new(file);

        // start of header
        let magic: u32 = read_u32_be(br)?;
        assert_eq!(
            magic, 0x50414b01,
            "file is missing pack2 magic value \"PAK\""
        );
        let asset_count: u32 = read_u32_le(br)?;
        let length: u64 = read_u64_le(br)?;
        let map_offset: u64 = read_u64_le(br)?;
        // unknown: u32_le
        // unknown: 128bytes
        // end of header

        br.seek(SeekFrom::Start(map_offset))?;
        let mut assets: Vec<Asset> = Vec::with_capacity(asset_count as usize);
        let mut filenames: Option<Vec<String>> = None;
        for _asset_id in 1..=asset_count {
            let asset = Asset::load_from_br(br)?;
            if asset.name_hash == 0x4137cc65bd97fd30 {
                let old_stream_position: u64 = br.stream_position()?;
                filenames = Some(
                    asset
                        .extract_text(br)?
                        .lines()
                        .map(|i| String::from(i))
                        .collect(),
                );
                br.seek(SeekFrom::Start(old_stream_position))?;
            }
            // if !asset.is_zipped {
            //     let old_stream_position: u64 = br.stream_position()?;
            //     println!("EXTRACTING");
            //     let mut fos: File = File::create_new(format!("EXTRACTED_{}", &asset_id))?;
            //     asset.raw_dump_to_file(&mut br, &mut fos)?;
            //     br.seek(SeekFrom::Start(old_stream_position))?;
            // }
            assets.push(asset);
        }

        let mut result = Self {
            asset_count,
            length,
            map_offset,
            assets,
        };

        if let Some(filenames_unwrapped) = &filenames {
            result.apply_filename_list(filenames_unwrapped);
        }

        Ok(result)
    }

    pub fn find_asset_index_by_name(&self, name: &str) -> Option<usize> {
        let hash: u64 = crc64::convert_filename(name);
        for i in self.assets.iter().enumerate() {
            if i.1.name_hash == hash {
                return Some(i.0);
            }
        }
        None
    }

    pub fn find_asset_index_by_name_hash(&self, hash: u64) -> Option<usize> {
        for i in self.assets.iter().enumerate() {
            if i.1.name_hash == hash {
                return Some(i.0);
            }
        }
        None
    }

    pub fn extract_all_named(&self, br: &mut File, output_directory: &Path) -> Result<()> {
        for asset in self.assets.iter() {
            if asset.name.is_some() {
                let mut fos: File =
                    File::create_new(output_directory.join(asset.name.clone().unwrap()))?;
                // let mut fos: File = File::create_new(
                //     format!("extract/{}", asset.name.clone().unwrap_or("".into())).as_str(),
                // )?;
                asset.extract_to_file(br, &mut fos)?;
            }
        }
        Ok(())
    }

    pub fn extract_all_unnamed(&self, br: &mut File, output_directory: &Path) -> Result<()> {
        for asset in self.assets.iter() {
            if asset.name.is_none() {
                let mut fos: File =
                    File::create_new(output_directory.join(format!("crc64_{}", asset.name_hash)))?;
                asset.extract_to_file(br, &mut fos)?;
            }
        }
        Ok(())
    }

    pub fn extract_file(
        &self,
        br: &mut File,
        file_to_extract: String,
        output_directory: &Path,
    ) -> Result<()> {
        for asset in self.assets.iter() {
            if asset.name.is_none() {
                continue;
            }
            if asset.name.clone().unwrap_or("".into()) == file_to_extract {
                let mut fos: File = File::create_new(output_directory.join(file_to_extract))?;
                asset.extract_to_file(br, &mut fos)?;
                return Ok(());
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Asset not found in pack2",
        ))
    }

    #[cfg(feature = "json")]
    pub fn ls_assets_as_json(&self) -> String {
        format!(
            "[{}]",
            self.assets
                .iter()
                .map(|i| i.to_json())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }

    pub fn ls_assets_for_humans(&self) -> String {
        #[cfg(feature = "use_comfy_table")]
        let mut table = comfy_table::Table::new();
        #[cfg(feature = "use_comfy_table")]
        table.set_header(vec!["Name", "Size"]);

        #[cfg(not(feature = "use_comfy_table"))]
        let mut out: Vec<String> = Vec::with_capacity(self.assets.len());

        for asset in self.assets.iter() {
            let mut columns: Vec<String> = Vec::with_capacity(2);

            if asset.name.is_some() {
                columns.push(asset.name.clone().unwrap());
            } else {
                columns.push(format!("crc64({})", asset.name_hash));
            }

            if asset.is_zipped {
                columns.push(format!(
                    "{} [Z]",
                    humanise_bytes(asset.unzipped_length.into())
                ));
            } else {
                columns.push(humanise_bytes(asset.data_length as f64));
            }

            #[cfg(feature = "use_comfy_table")]
            table.add_row(columns);
            #[cfg(not(feature = "use_comfy_table"))]
            out.push(columns.join(" "));
        }

        #[cfg(feature = "use_comfy_table")]
        return table.to_string();
        #[cfg(not(feature = "use_comfy_table"))]
        return out.join("\n");
    }

    #[cfg(feature = "manifests")]
    pub fn write_manifest_file(&self, manifst_file: &Path) -> Result<()> {
        let mut br: File = File::create_new(manifst_file)?;
        for asset in self.assets.iter() {
            br.write(&asset.name_hash.to_be_bytes())?;
            br.write(&asset.data_hash.to_be_bytes())?;
        }
        br.flush()?;
        Ok(())
    }

    #[cfg(feature = "manifests")]
    pub fn diff_with_manifest(
        &self,
        manifest: &crate::pack2_manifest::Manifest,
    ) -> crate::pack2_manifest::ManifestDiff {
        use crate::pack2_manifest::{ManifestDiff, ManifestDiffEntry};
        let mut result: ManifestDiff = Vec::new();
        for manifest_entry in manifest {
            match self.find_asset_index_by_name_hash(manifest_entry.0) {
                Some(asset_index) => {
                    if manifest_entry.1 != self.assets[asset_index].data_hash {
                        result.push(ManifestDiffEntry::new(
                            manifest_entry.0,
                            Some(manifest_entry.1),
                            Some((&self.assets[asset_index]).data_hash),
                        ));
                    }
                }
                None => result.push(ManifestDiffEntry::new(
                    manifest_entry.0,
                    Some(manifest_entry.1),
                    None,
                )),
            }
        }
        'asset_loop: for asset in self.assets.iter() {
            for manifest_entry in manifest {
                if manifest_entry.0 == asset.name_hash {
                    continue 'asset_loop;
                }
            }
            result.push(ManifestDiffEntry::new(
                asset.name_hash,
                None,
                Some(asset.data_hash),
            ));
        }
        result
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Asset {
    pub name: Option<String>,
    pub name_hash: u64, // of uppercase filename
    offset: u64,
    pub data_length: u64,
    pub is_zipped: bool,
    pub data_hash: u32,
    pub unzipped_length: u32,
}

impl Asset {
    #[cfg(feature = "json")]
    pub fn to_json(&self) -> String {
        format!(
            "{o}\"name\": {name}, \"name_hash\": {name_hash}, \"offset\": {offset}, \"data_length\": {data_length}, \"is_zipped\": {is_zipped}, \"data_hash\": {data_hash}, \"unzipped_length\": {unzipped_length}{c}",
            o = "{",
            c = "}",
            name = if self.name.is_some() {format!("\"{}\"", str::replace(str::replace(self.name.clone().unwrap().as_str(), "\\", "\\\\").as_str(), "\"", "\\\""))} else {String::from("null")},
            name_hash = self.name_hash.to_string(),
            offset = self.offset.to_string(),
            data_length = self.data_length.to_string(),
            data_hash = self.data_hash.to_string(),
            unzipped_length = self.unzipped_length.to_string(),
            is_zipped = if self.is_zipped {"true"} else {"false"},
        )
    }

    pub fn load_from_br(br: &mut File) -> Result<Self> {
        let name_hash: u64 = read_u64_le(br)?;
        let offset: u64 = read_u64_le(br)?;
        let data_length: u64 = read_u64_le(br)?;
        let zipped_flag: u32 = read_u32_le(br)?;
        let data_hash: u32 = read_u32_le(br)?;

        let is_zipped: bool = ZIPPED_FLAGS.contains(&zipped_flag) && data_length > 0;

        let unzipped_length: u32 = if is_zipped {
            let old_stream_position: u64 = br.stream_position()?;
            br.seek(SeekFrom::Start(offset))?;
            let zip_magic: u32 = read_u32_be(br)?;
            assert_eq!(
                zip_magic, 0xA1B2C3D4,
                "Asset compression magic value (prefix) not found (\"A1B2C3D4\")"
            );
            let unzipped_length: u32 = read_u32_be(br)?;
            br.seek(SeekFrom::Start(old_stream_position))?;
            unzipped_length
        } else {
            0
        };

        Ok(Self {
            name: None,
            name_hash,
            offset,
            data_length,
            is_zipped,
            data_hash,
            unzipped_length,
        })
    }

    fn raw_dump_to_file(
        &self,
        pack_file_stream: &mut File,
        output_file_steam: &mut File,
    ) -> Result<()> {
        pack_file_stream.seek(SeekFrom::Start(
            self.offset + if self.is_zipped { 8 } else { 0 },
        ))?;

        let mut buffer: [u8; 1024] = [0; 1024];
        for _ in 1..=(self.data_length >> 10) {
            pack_file_stream.read_exact(&mut buffer)?;
            output_file_steam.write(&buffer)?;
        }
        let mut buffer: [u8; 1] = [0];
        for _ in 1..=(self.data_length % 1024) {
            pack_file_stream.read_exact(&mut buffer)?;
            output_file_steam.write(&buffer)?;
        }

        output_file_steam.flush()?;
        Ok(())
    }

    pub fn extract_bytes(&self, pack_file_stream: &mut File) -> Result<Vec<u8>> {
        let raw_bytes: Vec<u8> = self.raw_bytes(pack_file_stream)?;
        if !self.is_zipped {
            return Ok(raw_bytes);
        }
        let mut d = ZlibDecoder::new(raw_bytes.as_slice());
        let mut buf: Vec<u8> = Vec::with_capacity(self.unzipped_length as usize);
        d.read_to_end(&mut buf)?;
        Ok(buf)
    }

    pub fn extract_to_file(
        &self,
        pack_file_stream: &mut File,
        output_file_steam: &mut File,
    ) -> Result<()> {
        if self.is_zipped {
            self.extract_compressed_to_file(pack_file_stream, output_file_steam)
        } else {
            self.raw_dump_to_file(pack_file_stream, output_file_steam)
        }
    }

    fn extract_compressed_to_file(
        &self,
        pack_file_stream: &mut File,
        output_file_steam: &mut File,
    ) -> Result<()> {
        pack_file_stream.seek(SeekFrom::Start(
            self.offset + if self.is_zipped { 8 } else { 0 },
        ))?;

        let mut d = ZlibDecoder::new(BufReader::new(pack_file_stream));

        let mut buf: [u8; 1024] = [0; 1024];
        for _ in 1..=(self.unzipped_length >> 10) {
            d.read_exact(&mut buf)?;
            output_file_steam.write(&buf)?;
        }
        let mut buf: [u8; 1] = [0];
        for _ in 1..=(self.unzipped_length % 1024) {
            d.read_exact(&mut buf)?;
            output_file_steam.write(&buf)?;
        }
        output_file_steam.flush()?;

        Ok(())
    }

    pub fn raw_bytes(&self, pack_file_stream: &mut File) -> Result<Vec<u8>> {
        pack_file_stream.seek(SeekFrom::Start(
            self.offset + if self.is_zipped { 8 } else { 0 },
        ))?;

        let mut out: Vec<u8> = Vec::with_capacity(self.data_length as usize);
        let mut buf: [u8; 1] = [0; 1];
        for _ in 1..=self.data_length {
            pack_file_stream.read_exact(&mut buf)?;
            out.push(buf[0]);
        }

        Ok(out)
    }

    pub fn extract_text(&self, pack_file_stream: &mut File) -> Result<String> {
        let raw_bytes: Vec<u8> = self.raw_bytes(pack_file_stream)?;
        if self.is_zipped {
            let mut d = ZlibDecoder::new(raw_bytes.as_slice());
            let mut s = String::new();
            d.read_to_string(&mut s)?;
            Ok(s)
        } else {
            match String::from_utf8(raw_bytes) {
                Ok(s) => Ok(s),
                Err(_) => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unable to decode utf-8 asset",
                )),
            }
        }
    }
}
