use crate::bin_utils::{read_big_x_bytes, read_u32_be, read_x_bytes};
use crate::cli_utils::humanise_bytes;
use std::io::prelude::Seek;
use std::{fs::File, io::SeekFrom};

pub struct Pack1 {
    pub chunks: Vec<Pack1Chunk>,
}

impl Pack1 {
    pub fn load_from_file(br: &mut File) -> std::io::Result<Self> {
        let mut chunks: Vec<Pack1Chunk> = Vec::new();
        loop {
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
            r#"{o}"chunks":[{chunks}]{c}"#,
            o = '{',
            c = '}',
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
}

pub struct Pack1Chunk {
    pub assets: Vec<Pack1Asset>,
}

impl Pack1Chunk {
    fn load_from_br(br: &mut File) -> std::io::Result<Self> {
        let mut assets: Vec<Pack1Asset> = Vec::new();
        let asset_count: u32 = read_u32_be(br)?;
        for _ in 0..asset_count {
            assets.push(Pack1Asset::load_from_br(br)?);
        }
        Ok(Self { assets })
    }

    #[cfg(feature = "json")]
    pub fn as_json(&self) -> String {
        format!(
            r#"{o}"assets":[{a}]{c}"#,
            o = '{',
            c = '}',
            a = self
                .assets
                .iter()
                .map(|i| i.as_json())
                .collect::<Vec<String>>()
                .join(","),
        )
    }
}

pub struct Pack1Asset {
    pub name: String,
    pub offset: u32,
    pub data_length: u32,
    pub file_hash: u32,
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
        })
    }

    pub fn raw_bytes(&self, pack_file_stream: &mut File) -> std::io::Result<Vec<u8>> {
        pack_file_stream.seek(SeekFrom::Start(self.offset as u64))?;
        read_big_x_bytes(pack_file_stream, self.data_length as usize)
    }

    #[cfg(feature = "json")]
    pub fn as_json(&self) -> String {
        use crate::json_utils::escape_string;

        format!(
            r#"{o}"name":{name},"offest":{offset},"data_length":{data_length},"file_hash":{file_hash}{c}"#,
            o = '{',
            c = '}',
            name = escape_string(self.name.as_str()),
            offset = self.offset,
            data_length = self.data_length,
            file_hash = self.file_hash,
        )
    }
}
