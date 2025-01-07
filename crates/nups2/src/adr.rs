use serde::Deserialize;
use std::io::Read;

use quick_xml::reader::Reader;

#[deprecated(note = "Do not use (not finished)")]
pub struct Adr {}

impl Adr {
    pub fn parse_from_text(text: &str) -> std::io::Result<Self> {
        let mut reader = Reader::from_str(text);
        reader.config_mut().trim_text(true);
        Ok(Self {})
    }
}

#[serde()]
pub struct ActorDefinition {}
