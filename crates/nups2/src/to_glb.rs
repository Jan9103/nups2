use std::collections::HashMap;
use std::io::Write;
use std::option::Iter;

use crate::bin_utils::write_u32_le;
use crate::{dma::Dma, dme::Dme};

// https://docs.fileformat.com/3d/glb/

const GLTF_CONTAINER_FORMAT_VERSION: u32 = 1;

pub struct Glb {
    json_head: GlbChunk,
    bin_head: GlbChunk,
}

impl Glb {
    pub fn get_total_length_in_bytes(&self) -> u32 {
        const HEADER_LENGTH: u32 = 12;
        HEADER_LENGTH
    }

    pub fn write(&self, fos: &mut dyn Write) -> Result<(), std::io::Error> {
        let total_length_in_bytes: u32 = 0; // FIXME

        write_u32_le(0x46546C67u32, fos)?; // magic
        write_u32_le(GLTF_CONTAINER_FORMAT_VERSION, fos)?;
        write_u32_le(total_length_in_bytes, fos)?;
        Ok(())
    }
}

trait GlbChunk {
    fn length_in_bytes(&self) -> u32;

    /// This destroys this object in order to reduce ram-usage (yes this is very purpose-build)
    fn into_binary(mut self) -> Vec<u8>;

    fn get_chunk_type_id() -> u32;
    fn get_padding_byte(&self) -> u8;
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GlbBinChunk {
    content: Vec<u8>,
}

fn calc_chunk_padding_amount(length_in_bytes: u32) -> u32 {
    let m = length_in_bytes % 4;
    if m == 0 {
        0
    } else {
        4 - m
    }
}

impl GlbChunk for GlbBinChunk {
    fn length_in_bytes(&self) -> u32 {
        const HEADER_LENGTH: u32 = 4 + 4;
        let length_in_bytes = (self.content.len() as u32) + HEADER_LENGTH;
        length_in_bytes + calc_chunk_padding_amount(length_in_bytes)
    }

    /// This destroys this object in order to reduce ram-usage (yes this is very purpose-build)
    fn into_binary(mut self) -> Vec<u8> {
        let l: u32 = self.length_in_bytes();
        let mut res: Vec<u8> = Vec::new();
        res.extend_from_slice(&l.to_le_bytes());
        res.extend_from_slice(&Self::get_chunk_type_id().to_le_bytes());
        res.append(&mut self.content); // this is a move, not a copy

        let p: u8 = self.get_padding_byte();
        for _ in 0..calc_chunk_padding_amount(l) {
            res.push(p);
        }
        res
    }

    fn get_chunk_type_id() -> u32 {
        0x004E4942
    }

    fn get_padding_byte(&self) -> u8 {
        0x00
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum GlbChunkType {
    JSON,
    BIN,
}

impl GlbChunkType {
    pub fn from_u32(num: u32) -> Option<Self> {
        match num {
            0x4E4F534A => Some(GlbChunkType::JSON),
            0x004E4942 => Some(GlbChunkType::BIN),
            _ => None,
        }
    }
    pub fn get_u32_id(&self) -> u32 {
        match self {
            GlbChunkType::JSON => 0x4E4F534A,
            GlbChunkType::BIN => 0x004E4942,
        }
    }
    pub fn get_padding_byte(&self) -> u8 {
        match self {
            GlbChunkType::JSON => 0x20,
            GlbChunkType::BIN => 0x00,
        }
    }
}

/////////////// JSON GENERATION HELPER FUNCTIONS //////////////////

enum GlbJsonObject {
    Record(HashMap<String, GlbJsonObject>),
    List(Vec<GlbJsonObject>),
    Text(String),
    Number(i64),
}

enum GlbJsonPathNode {
    Index(usize),
    Key(String),
}

impl GlbJsonObject {
    pub fn get_path(
        &self,
        mut path: Iter<GlbJsonPathNode>,
    ) -> Result<&GlbJsonObject, &'static str> {
        if let Some(i) = path.next() {
            match self {
                Self::Record(hm) => match i {
                    GlbJsonPathNode::Index(_) => Err("HashMap cant be int-indexed"),
                    GlbJsonPathNode::Key(k) => {
                        if let Some(e) = hm.get(k) {
                            e.get_path(path)
                        } else {
                            Err("Key not found in HashMap")
                        }
                    }
                },
                Self::List(l) => match i {
                    GlbJsonPathNode::Index(idx) => {
                        if let Some(e) = l.get(*idx) {
                            e.get_path(path)
                        } else {
                            Err("Index out of bounds for List")
                        }
                    }
                    GlbJsonPathNode::Key(_) => Err("Can't index list with a string"),
                },
                Self::Text(_) => Err("Text cant be indexed"),
                Self::Number(_) => Err("Number cant be indexed"),
            }
        } else {
            Ok(self)
        }
    }
    pub fn record_put(&mut self, key: String, value: GlbJsonObject) -> Result<(), ()> {
        match self {
            Self::Record(hm) => {
                hm.insert(key, value);
                Ok(())
            }
            _ => Err(()),
        }
    }

    pub fn list_push(&mut self, value: GlbJsonObject) -> Result<(), ()> {
        match self {
            Self::List(l) => {
                l.push(value);
                Ok(())
            }
            _ => Err(()),
        }
    }
}
