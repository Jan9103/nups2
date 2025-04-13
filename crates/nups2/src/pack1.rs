use flate2::bufread::ZlibDecoder;

use crate::bin_utils::{
    clone_big_x_bytes, read_big_x_bytes, read_u32_be, read_x_bytes, write_u32_be,
};
use crate::cli_utils::humanise_bytes;
use std::fmt::Display;
use std::io::prelude::Seek;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::{fs::File, io::SeekFrom};

#[derive(Debug)]
pub struct Pack1 {
    pub chunks: Vec<Pack1Chunk>,
}

impl Pack1 {
    pub fn load_from_file(br: &mut File) -> std::io::Result<Self> {
        log::debug!("Loading pack1 file..");
        let mut chunks: Vec<Pack1Chunk> = Vec::new();
        loop {
            log::trace!("Loading pack1 Chunk idx {} headers..", chunks.len());
            let next_chunk: u32 = read_u32_be(br)?;
            chunks.push(Pack1Chunk::load_from_br(br)?);
            if next_chunk == 0 {
                break;
            }
            br.seek(SeekFrom::Start(next_chunk as u64))?;
        }
        Ok(Self { chunks })
    }

    #[cfg(feature = "json")]
    pub fn as_json(&self) -> String {
        format!(
            r#"{{"chunks":[{chunks}]}}"#,
            chunks = self
                .chunks
                .iter()
                .map(|i| i.as_json())
                .collect::<Vec<String>>()
                .join(","),
        )
    }

    pub fn ls_for_humans(&self) -> String {
        #[cfg(feature = "use_comfy_table")]
        let mut table = comfy_table::Table::new();
        #[cfg(feature = "use_comfy_table")]
        table.set_header(vec!["Name", "Size", "Chunk"]);
        #[cfg(not(feature = "use_comfy_table"))]
        let mut out: Vec<String> = Vec::with_capacity(self.assets.len());

        for chunk in self.chunks.iter().enumerate() {
            for asset in chunk.1.assets.iter() {
                let columns: Vec<String> = vec![
                    asset.name.clone(),
                    humanise_bytes(asset.data_length.into()),
                    format!("{}", chunk.0),
                ];

                #[cfg(feature = "use_comfy_table")]
                table.add_row(columns);
                #[cfg(not(feature = "use_comfy_table"))]
                out.push(columns.join(" "));
            }
        }

        #[cfg(feature = "use_comfy_table")]
        return table.to_string();
        #[cfg(not(feature = "use_comfy_table"))]
        return out.join("\n");
    }

    pub fn extract_all(
        &self,
        br: &mut File,
        output_directory: &Path,
        subdirs_for_chunks: bool,
    ) -> std::io::Result<()> {
        std::fs::create_dir_all(output_directory)?;

        for (index, chunk) in self.chunks.iter().enumerate() {
            let chunkdir: PathBuf = if subdirs_for_chunks {
                let res = output_directory.join(format!("chunk_{}", index));
                std::fs::create_dir(&res)?;
                res
            } else {
                output_directory.to_path_buf()
            };
            chunk.extract_all(br, &chunkdir)?;
        }
        Ok(())
    }

    pub fn write(
        &self,
        old_pack_file: &mut File,
        target_file: &mut dyn Write,
    ) -> std::io::Result<()> {
        log::debug!("Starting generation of pack1 file");
        if self.chunks.is_empty() || self.chunks.iter().all(|chunk| chunk.assets.is_empty()) {
            log::info!("Written pack1 file is completely empty");
            write_u32_be(0, target_file)?;
            return Ok(());
        }

        log::trace!("Calculating offsets");
        let header_length: u32 = self
            .chunks
            .iter()
            .map(|chunk| {
                8 // offset of next + asset count
                + chunk.assets.iter().map(|asset| -> u32 {asset.header_length()}).sum::<u32>()
            })
            .sum::<u32>()
            + 4u32; // the 0 at the end to indicate "no further chunks"

        let mut chunk_header_offsets: Vec<u32> = Vec::with_capacity(self.chunks.len());
        let mut current_offset: u32 = 1; // 1 for the initial chunk offset
        for chunk in self.chunks.iter() {
            chunk_header_offsets.push(current_offset);
            current_offset += 8 // offset of next + asset count
                + chunk.assets.iter().map(|asset| -> u32 {asset.header_length()}).sum::<u32>();
        }

        let mut asset_offsets: Vec<Vec<u32>> = Vec::with_capacity(self.chunks.len());
        let mut current_offset: u32 = header_length;
        for chunk in self.chunks.iter() {
            let mut offsets_for_chunk: Vec<u32> = Vec::with_capacity(chunk.assets.len());
            for asset in chunk.assets.iter() {
                offsets_for_chunk.push(current_offset);
                current_offset += asset.data_length;
            }
            asset_offsets.push(offsets_for_chunk);
        }

        log::trace!("Writing headers");
        // actually write header
        //write_u32_be(1, target_file)?; // offset of the first chunk header
        for (chunk_id, chunk) in self.chunks.iter().enumerate() {
            // next chunk header offset
            write_u32_be(
                if let Some(next_offset) = chunk_header_offsets.get(chunk_id + 1) {
                    *next_offset
                } else {
                    0
                },
                target_file,
            )?;
            write_u32_be(chunk.assets.len() as u32, target_file)?;
            let offsets_in_chunk: &Vec<u32> = &asset_offsets[chunk_id];
            for (asset_id, asset) in chunk.assets.iter().enumerate() {
                write_u32_be(asset.name.len() as u32, target_file)?;
                target_file.write_all(asset.name.as_bytes())?;
                write_u32_be(offsets_in_chunk[asset_id], target_file)?;
                write_u32_be(asset.data_length, target_file)?;
                write_u32_be(asset.file_hash, target_file)?;
            }
        }
        write_u32_be(0, target_file)?; // no next file

        log::trace!("Writing data");
        // write actual data
        for chunk in self.chunks.iter() {
            for asset in chunk.assets.iter() {
                log::trace!("Asset: {asset}");
                asset.clone_data(old_pack_file, target_file)?;
            }
        }

        Ok(())
    }

    pub fn from_pack2(
        p2: crate::pack2::Pack2,
        unknown_name_handling: &UnknownNameHandling,
    ) -> Result<Self, &'static str> {
        // AFAIK the limit for a pack1 chunk is u32::MAX, which is more than the asset offset can handle -> no problem
        Ok(Self {
            chunks: vec![Pack1Chunk {
                assets: p2
                    .assets
                    .into_iter()
                    .filter(|asset| {
                        *unknown_name_handling != UnknownNameHandling::SkipFile
                            || asset.name.is_some()
                    })
                    .map(|asset| Pack1Asset::from_pack2_asset(asset, unknown_name_handling))
                    .collect::<Result<Vec<Pack1Asset>, &'static str>>()?,
            }],
        })
    }
}

impl Display for Pack1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pack1File [{}]",
            self.chunks
                .iter()
                .map(|i| format!("{i}"))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl TryFrom<crate::pack2::Pack2> for Pack1 {
    type Error = &'static str;

    fn try_from(p2: crate::pack2::Pack2) -> Result<Self, Self::Error> {
        Self::from_pack2(p2, &UnknownNameHandling::ReturnError)
    }
}

#[derive(Debug)]
pub struct Pack1Chunk {
    pub assets: Vec<Pack1Asset>,
}

impl Pack1Chunk {
    fn load_from_br(br: &mut File) -> std::io::Result<Self> {
        let mut assets: Vec<Pack1Asset> = Vec::new();
        let asset_count: u32 = read_u32_be(br)?;
        for asset_idx in 0..asset_count {
            log::trace!("Loading asset idx {asset_idx} header");
            assets.push(Pack1Asset::load_from_br(br)?);
        }
        Ok(Self { assets })
    }

    #[cfg(feature = "json")]
    pub fn as_json(&self) -> String {
        format!(
            r#"{{"assets":[{a}]}}"#,
            a = self
                .assets
                .iter()
                .map(|i| i.as_json())
                .collect::<Vec<String>>()
                .join(","),
        )
    }

    pub fn extract_all(&self, br: &mut File, output_directory: &Path) -> std::io::Result<()> {
        for asset in self.assets.iter() {
            let output_file: PathBuf = output_directory.join(&asset.name);
            let mut output_stream: File = File::create_new(output_file)?;
            asset.clone_data(br, &mut output_stream)?;
        }
        Ok(())
    }
}

impl Display for Pack1Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pack1Chunk [{}]",
            self.assets
                .iter()
                .map(|i| format!("{i}"))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug)]
pub struct Pack1Asset {
    pub name: String,
    pub offset: u32,
    pub data_length: u32,
    pub file_hash: u32,
    /// for pack2 -> pack1 conversion
    stream_is_pack2_zipped: bool,
}

impl Pack1Asset {
    fn load_from_br(br: &mut File) -> std::io::Result<Self> {
        let name_length: u32 = read_u32_be(br)?;
        let name_bytes: Vec<u8> = read_x_bytes(br, name_length as usize)?;
        let name: String = String::from_utf8(name_bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let offset: u32 = read_u32_be(br)?;
        let data_length: u32 = read_u32_be(br)?;
        let file_hash: u32 = read_u32_be(br)?;
        Ok(Self {
            name,
            offset,
            data_length,
            file_hash,
            stream_is_pack2_zipped: false,
        })
    }

    fn header_length(&self) -> u32 {
        16 + (self.name.as_bytes().len() as u32)
    }

    // pub fn raw_bytes(&self, pack_file_stream: &mut File) -> std::io::Result<Vec<u8>> {
    //     pack_file_stream.seek(SeekFrom::Start(self.offset as u64))?;
    //     read_big_x_bytes(pack_file_stream, self.data_length as usize)
    // }

    pub fn clone_data(
        &self,
        pack_file_stream: &mut File,
        output_stream: &mut dyn Write,
    ) -> std::io::Result<()> {
        log::trace!("cloning asset {}", self);

        if self.stream_is_pack2_zipped {
            pack_file_stream.seek(SeekFrom::Start(self.offset as u64 + 8))?;
            let mut d = ZlibDecoder::new(BufReader::new(pack_file_stream));
            clone_big_x_bytes(&mut d, output_stream, self.data_length as usize)?;
        } else {
            pack_file_stream.seek(SeekFrom::Start(self.offset as u64))?;
            clone_big_x_bytes(pack_file_stream, output_stream, self.data_length as usize)?;
        }

        Ok(())
    }

    #[cfg(feature = "json")]
    pub fn as_json(&self) -> String {
        use crate::json_utils::escape_string;

        format!(
            r#"{{"name":{name},"offest":{offset},"data_length":{data_length},"file_hash":{file_hash}}}"#,
            name = escape_string(self.name.as_str()),
            offset = self.offset,
            data_length = self.data_length,
            file_hash = self.file_hash,
        )
    }

    /// WARNING: this does not set the data_hash field
    pub fn as_pack2_asset_fast(&self) -> crate::pack2::Asset {
        crate::pack2::Asset {
            name: Some(self.name.clone()),
            name_hash: crate::crc64::convert_filename(&self.name),
            offset: self.offset as u64,
            data_length: self.data_length as u64,
            is_zipped: false,
            data_hash: 0,       // FIXME
            unzipped_length: 0, // not zipped and then its 0
        }
    }

    pub fn from_pack2_asset(
        p2a: crate::pack2::Asset,
        unknown_name_handling: &UnknownNameHandling,
    ) -> Result<Self, &'static str> {
        log::trace!("Converting pack2 asset to pack1: {}", &p2a);
        let name = if let Some(p2a_name) = p2a.name {
            p2a_name
        } else {
            match unknown_name_handling {
                UnknownNameHandling::ReturnError | UnknownNameHandling::SkipFile => {
                    log::error!("Asset has no name: {}", p2a);
                    return Err("Can't convert a pack2 asset with a unknown name to pack1");
                }
                UnknownNameHandling::GenerateName => {
                    format!("crc_64_{}", p2a.name_hash)
                }
            }
        };
        // as of 2025-04-12 the biggest pack2 file is 330.8MB (Oshur_x64_4.pack2). the u32::MAX is 4.2GB. -> no need to worry any time soon
        if p2a.offset > (u32::MAX as u64) || p2a.data_length > (u32::MAX as u64) {
            return Err(
                "Can't convert a pack2 asset to a pack1 asset due to pack1 limitations (u32::MAX < u64::MAX)"
            );
        }
        let res = Ok(Self {
            name,
            offset: p2a.offset as u32,
            data_length: if p2a.is_zipped {
                p2a.unzipped_length
            } else {
                p2a.data_length as u32
            },
            file_hash: p2a.data_hash, // i presume its the same?
            stream_is_pack2_zipped: p2a.is_zipped,
        });
        res
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum UnknownNameHandling {
    ReturnError,
    GenerateName,
    SkipFile,
}

impl TryFrom<crate::pack2::Asset> for Pack1Asset {
    type Error = &'static str;

    fn try_from(p2a: crate::pack2::Asset) -> Result<Self, Self::Error> {
        Self::from_pack2_asset(p2a, &UnknownNameHandling::ReturnError)
    }
}

impl Display for Pack1Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pack1Asset({name})", name = self.name)
    }
}
