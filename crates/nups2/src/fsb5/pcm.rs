/// based on https://en.wikipedia.org/wiki/WAV
/// >4GB wav files: https://github.com/0xAA55-rs/rustwav/blob/main/packages/rustwav-core/src/wavwriter.rs
use std::{fs::File, io::Write};

use crate::{bin_utils, Nups2Error};

use super::{Fsb5Sample, SoundFormat};

pub fn rebuild(
    sample: &Fsb5Sample,
    br: &mut File,
    target: &mut dyn Write,
    sound_format: &'static SoundFormat,
) -> Result<(), Nups2Error> {
    let (width, audio_format): (u16, u16) = match sound_format.name {
        "pcm8" => (1, AUDIO_FORMAT_PCM_INTEGER),
        "pcm16" => (2, AUDIO_FORMAT_PCM_INTEGER),
        "pcm32" => (4, AUDIO_FORMAT_PCM_INTEGER),
        // TODO: figure out the width of "pcmfloat"
        _ => {
            return Err(Nups2Error::Other(
                "fsb5::pcm: unknown pcm width for sound_format",
            ));
        }
    };

    let data_start = sample.data_offset;
    let data_end = u64::min(
        sample.data_end,
        data_start + ((sample.samples as u64) * (width as u64)),
    );

    let sample_data: Vec<u8> = bin_utils::read_big_x_bytes(br, (data_end - data_start) as usize)?;
    write_wav(
        target,
        sample.channels as u16,
        sample.frequency,
        sample_data,
        width,
        audio_format,
    )?;

    Ok(())
}

const AUDIO_FORMAT_PCM_INTEGER: u16 = 1;
#[allow(dead_code)]
const AUDIO_FORMAT_IEEE_754_FLOAT: u16 = 3;

fn write_wav(
    target: &mut dyn Write,
    channel_count: u16,
    frequency: u32,
    sampled_data: Vec<u8>,
    sampwidth: u16,
    audio_format: u16,
) -> Result<(), Nups2Error> {
    const HEADER_SIZE: u32 = 44;
    let data_size: u32 = sampled_data.len() as u32;
    let total_size: u32 = HEADER_SIZE + data_size;

    let bytes_per_block: u16 = channel_count * sampwidth;

    // master RIFF chunk
    target.write_all(&[0x52, 0x49, 0x46, 0x46])?; // RIFF
    bin_utils::write_u32_le(total_size - 8, target)?;
    target.write_all(&[0x57, 0x41, 0x56, 0x45])?; // WAVE

    // data format description
    target.write_all(&[0x66, 0x5D, 0x74, 0x20])?; // fmt_
    bin_utils::write_u32_le(16, target)?; // chunk size
    bin_utils::write_u16_le(audio_format, target)?; // audio_format=pcm
    bin_utils::write_u16_le(channel_count, target)?;
    bin_utils::write_u32_le(frequency, target)?;
    bin_utils::write_u32_be(frequency * (bytes_per_block as u32), target)?;
    bin_utils::write_u16_le(bytes_per_block, target)?;
    bin_utils::write_u16_le(sampwidth * 8, target)?;

    // sampled data
    target.write_all(&[0x64, 0x61, 0x74, 0x61])?; // data
    bin_utils::write_u32_le(data_size, target)?;
    target.write_all(&sampled_data)?;

    Ok(())
}
