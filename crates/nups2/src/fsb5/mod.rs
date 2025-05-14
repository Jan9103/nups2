/// This file is based on https://github.com/HearthSim/python-fsb5
/// docs for fsb4, fsb3, and fsb2: https://web.archive.org/web/20160622000928/https://www.fmod.org/questions/question/forum-4928/ (original deleted)
/// .fsb.xen decrypter: https://pastebin.com/BFEz1pYd
/// bsf info: https://github.com/CyberBotX/fsb5_split/blob/master/Program.cs
use std::io::{Read, SeekFrom, Write};
use std::{fs::File, io::Seek};

use crate::{bin_utils, Nups2Error};

mod sound_formats;
pub use sound_formats::SoundFormat;
mod pcm;

fn g_bits64(val: u64, start: usize, len: usize) -> u64 {
    let stop = start + len;
    u64::overflowing_shr(
        val & ((1u64.overflowing_shl(stop as u32).0) - 1),
        start as u32,
    )
    .0
}
fn g_bits32(val: u32, start: usize, len: usize) -> u32 {
    let stop = start + len;
    u32::overflowing_shr(
        val & ((1u32.overflowing_shl(stop as u32).0) - 1),
        start as u32,
    )
    .0
}

const FREQUENCY_VALUES: &[(u8, u32)] = &[
    (1, 8_000),
    (2, 11_000),
    (3, 11_025),
    (4, 16_000),
    (5, 22_050),
    (6, 24_000),
    (7, 32_000),
    (8, 44_100),
    (9, 48_000),
];

#[derive(Debug)]
pub enum Fsb5SampleChunk {
    Channels(u8),
    Frequency(u32),
    Loop(u32, u32),
    Xmaseek(Vec<u8>),
    Dspcoeff(Vec<u8>),
    Xwmadata(Vec<u8>),
    Vorbisdata { crc32: u32, unknown: Vec<u8> },
    Unknown { chunk_type_id: u32, data: Vec<u8> },
}

impl Fsb5SampleChunk {
    pub fn from_stream(
        br: &mut File,
        chunk_size: u32,
        chunk_type: u32,
    ) -> Result<Self, Nups2Error> {
        Ok(match chunk_type {
            1 => {
                if chunk_size != 1 {
                    return Err(Nups2Error::Other(
                        "Fsb5SampleChunk::Channels: chunk_size != 1",
                    ));
                }
                Self::Channels(bin_utils::read_u8_le(br)?)
            }
            2 => {
                if chunk_size != 8 {
                    return Err(Nups2Error::Other(
                        "Fsb5SampleChunk::Frequency: chunk_size != 8",
                    ));
                }
                Self::Frequency(bin_utils::read_u32_le(br)?)
            }
            3 => {
                if chunk_size != 16 {
                    return Err(Nups2Error::Other("Fsb5SampleChunk::Loop: chunk_size != 16"));
                }
                let v1 = bin_utils::read_u32_le(br)?;
                let v2 = bin_utils::read_u32_le(br)?;
                Self::Loop(v1, v2)
            }
            6 => Self::Xmaseek(bin_utils::read_x_bytes(br, chunk_size as usize)?),
            7 => Self::Dspcoeff(bin_utils::read_x_bytes(br, chunk_size as usize)?),
            10 => Self::Xwmadata(bin_utils::read_x_bytes(br, chunk_size as usize)?),
            11 => {
                let crc32 = bin_utils::read_u32_le(br)?; // why is it 64 bits for crc32?!
                let unknown = bin_utils::read_x_bytes(br, (chunk_size as usize) - 4)?;
                Self::Vorbisdata { crc32, unknown }
            }
            o => Self::Unknown {
                chunk_type_id: o,
                data: bin_utils::read_x_bytes(br, chunk_size as usize)?,
            },
        })
    }
}

#[derive(Debug)]
pub struct Fsb5Sample {
    pub name: String,
    pub frequency: u32,
    pub channels: u8,
    pub data_offset: u64,
    pub data_end: u64,
    pub samples: u32,
    pub chunks: Vec<Fsb5SampleChunk>,
}

impl Fsb5Sample {
    pub fn read(&self, br: &mut File) -> Result<Vec<u8>, Nups2Error> {
        br.seek(SeekFrom::Start(self.data_offset))?;
        let raw_bin = bin_utils::read_big_x_bytes(br, (self.data_end - self.data_offset) as usize)?;
        Ok(raw_bin)
    }

    pub fn from_stream(br: &mut File, name: String) -> Result<Self, Nups2Error> {
        // FIXME: not sure if u64 or u32
        // upstream used "I", which is defined as u64, but has been u32 in most other places
        // with u32 something in the chunk failes
        let raw = bin_utils::read_u64_le(br)?; // unknown - "raw"
        #[cfg(debug_assertions)]
        dbg!(&raw);
        let mut next_chunk: bool = g_bits64(raw, 0, 1) == 1;
        #[cfg(debug_assertions)]
        dbg!(&next_chunk);
        let frequency_number = g_bits64(raw, 1, 4) as u8;
        #[cfg(debug_assertions)]
        dbg!(&frequency_number);
        let channels: u8 = (g_bits64(raw, 1 + 4, 1) + 1) as u8;
        #[cfg(debug_assertions)]
        dbg!(&channels);
        // FIXME: 0
        // either i have to add some other number to it (header-length or something?)
        // or something is wrong
        let data_offset = g_bits64(raw, 1 + 4 + 1, 28) * 16;
        #[cfg(debug_assertions)]
        dbg!(&data_offset);
        let samples = g_bits64(raw, 1 + 4 + 1 + 28, 30) as u32;
        #[cfg(debug_assertions)]
        dbg!(&samples);

        let mut chunks: Vec<Fsb5SampleChunk> = Vec::new();
        while next_chunk {
            let raw = bin_utils::read_u32_le(br)?;
            dbg!(&raw);
            next_chunk = g_bits32(raw, 0, 1) == 1;
            dbg!(&next_chunk);
            let chunk_size = g_bits32(raw, 1, 24);
            let chunk_type = g_bits32(raw, 1 + 24, 7);
            dbg!(&chunk_size);
            dbg!(&chunk_type);

            chunks.push(Fsb5SampleChunk::from_stream(br, chunk_size, chunk_type)?);
        }

        let frequency = match chunks
            .iter()
            .find_map(|i| match i {
                Fsb5SampleChunk::Frequency(v) => Some(*v),
                _ => None,
            })
            .or_else(|| -> Option<u32> {
                FREQUENCY_VALUES.iter().find_map(|i| {
                    if i.0 == frequency_number {
                        Some(i.1)
                    } else {
                        None
                    }
                })
            }) {
            Some(v) => v,
            None => {
                return Err(Nups2Error::Other(
                    "Fsb5Sample: unable to figure out frequency",
                ));
            }
        };

        Ok(Self {
            name,
            frequency,
            channels,
            data_offset: data_offset as u64,
            data_end: 0,
            samples,
            chunks,
        })
    }

    pub fn rebuild(
        &self,
        br: &mut File,
        target: &mut dyn Write,
        sound_format: &'static SoundFormat,
    ) -> Result<(), Nups2Error> {
        match sound_format.name {
            "vorbis" => {
                todo!()
            }
            "pcm8" | "pcm16" | "pcm32" => pcm::rebuild(self, br, target, sound_format),
            "mpeg" => {
                bin_utils::clone_big_x_bytes(
                    br,
                    target,
                    (self.data_end - self.data_offset) as usize,
                )?;
                Ok(())
            }
            // FIXME: imaadpcm -- this seems to be used by ps2
            // FIXME: vorbis -- according to aezay.dk used in ps2
            _ => Err(Nups2Error::Other("Fsb5: Unsupported sound_format")),
        }
    }
}

fn read_string(br: &mut File) -> Result<String, Nups2Error> {
    let mut res: Vec<u8> = Vec::new();
    let mut buf: [u8; 1] = [0];
    loop {
        br.read_exact(&mut buf)?;
        match buf {
            [0x0] => {
                break;
            }
            _ => {
                res.push(buf[0]);
            }
        }
    }

    Ok(String::from_utf8(res)?)
}

#[derive(Debug)]
pub struct Fsb5 {
    pub samples: Vec<Fsb5Sample>,
    pub sound_format: &'static SoundFormat,
}

impl Fsb5 {
    pub fn new(br: &mut File) -> Result<Self, Nups2Error> {
        // 4s
        if !matches!(
            bin_utils::read_x_bytes(br, 4)?[..],
            [0x46, 0x53, 0x42, 0x35]
        ) {
            return Err(Nups2Error::Other(
                "FSB5 file is missing its magic value/header: FSB5",
            ));
        }
        // I
        let version = bin_utils::read_u32_le(br)?;
        #[cfg(debug_assertions)]
        dbg!(&version);
        // I
        let num_samples = bin_utils::read_u32_le(br)?;
        #[cfg(debug_assertions)]
        dbg!(&num_samples);
        // I
        let sample_header_size = bin_utils::read_u32_le(br)?;
        #[cfg(debug_assertions)]
        dbg!(&sample_header_size);
        // I
        let name_table_size = bin_utils::read_u32_le(br)?;
        #[cfg(debug_assertions)]
        dbg!(&name_table_size);
        // I
        let data_size = bin_utils::read_u32_le(br)?;
        #[cfg(debug_assertions)]
        dbg!(&data_size);
        // I
        let mode = bin_utils::read_u32_le(br)?;
        #[cfg(debug_assertions)]
        dbg!(&mode);
        // 8s
        let _zero = bin_utils::read_x_bytes(br, 8)?;
        #[cfg(debug_assertions)]
        dbg!(&_zero);
        // 16s
        let _hash = bin_utils::read_x_bytes(br, 16)?;
        #[cfg(debug_assertions)]
        dbg!(&_hash);
        // 8s
        let _dummy = bin_utils::read_x_bytes(br, 8)?;
        #[cfg(debug_assertions)]
        dbg!(&_dummy);

        if version == 0 {
            // unknown
            bin_utils::read_u32_le(br)?;
        }

        let header_length: u64 = br.stream_position()?;
        #[cfg(debug_assertions)]
        dbg!(&header_length);

        let sound_format: &'static SoundFormat = match SoundFormat::get_by_id(mode as u8) {
            Some(v) => v,
            None => {
                return Err(Nups2Error::Other(
                    "Fsb5: unknown/unsupported sound-format/mode id",
                ));
            }
        };
        #[cfg(debug_assertions)]
        dbg!(&sound_format);

        let mut samples: Vec<Fsb5Sample> = (0..num_samples)
            .map(|i| Fsb5Sample::from_stream(br, format!("{i}")))
            .collect::<Result<Vec<Fsb5Sample>, Nups2Error>>()?;
        #[cfg(debug_assertions)]
        dbg!(&samples);

        if name_table_size != 0 {
            let nametable_start = br.stream_position()?;
            #[cfg(debug_assertions)]
            dbg!(&nametable_start);
            let samplename_offsets: Vec<u32> = (0..num_samples)
                .map(|_| bin_utils::read_u32_le(br))
                .collect::<Result<Vec<u32>, std::io::Error>>()?;
            for i in 0..num_samples {
                br.seek(SeekFrom::Start(
                    nametable_start + (*(samplename_offsets.get(i as usize).expect("")) as u64),
                ))?;
                let sample: &mut Fsb5Sample = samples.get_mut(i as usize).expect("");
                sample.name = read_string(br)?;
            }
        }

        br.seek(SeekFrom::Start(
            header_length + (sample_header_size as u64) + (name_table_size as u64),
        ))?;
        for i in 0..num_samples {
            let data_start = samples.get(i as usize).expect("").data_offset;
            let data_end = if i == num_samples - 1 {
                data_start + (data_size as u64)
            } else {
                samples.get((i + 1) as usize).unwrap().data_offset
            };
            let sample = samples.get_mut(i as usize).expect("");
            sample.data_end = data_end;
        }

        Ok(Self {
            samples,
            sound_format,
        })
    }
}
