use std::io::Read;
use std::io::Result;
use std::io::Write;

pub fn read_u32_le(br: &mut dyn Read) -> Result<u32> {
    let mut buf: [u8; 4] = [0; 4];
    br.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_u64_le(br: &mut dyn Read) -> Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    br.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub fn read_u32_be(br: &mut dyn Read) -> Result<u32> {
    let mut buf: [u8; 4] = [0; 4];
    br.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

pub fn read_u64_be(br: &mut dyn Read) -> Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    br.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

pub fn read_u16_le(br: &mut dyn Read) -> Result<u16> {
    let mut buf: [u8; 2] = [0; 2];
    br.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

pub fn read_u16_be(br: &mut dyn Read) -> Result<u16> {
    let mut buf: [u8; 2] = [0; 2];
    br.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

pub fn read_f32_be(br: &mut dyn Read) -> Result<f32> {
    let mut buf: [u8; 4] = [0; 4];
    br.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

/// equivalent ot c# BinaryReader.ReadSingle
pub fn read_f32_le(br: &mut dyn Read) -> Result<f32> {
    let mut buf: [u8; 4] = [0; 4];
    br.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

/// intended for reading a filename, etc - use read_big_x_bytes for entire files
pub fn read_x_bytes(br: &mut dyn Read, byte_count: usize) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(byte_count);
    let mut buffer: [u8; 1] = [0u8];
    for _ in 1..=byte_count {
        br.read_exact(&mut buffer)?;
        out.push(buffer[0]);
    }
    Ok(out)
}

/// read_x_bytes, but optimized for bigger things, such as 3d-models
pub fn read_big_x_bytes(br: &mut dyn Read, byte_count: usize) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(byte_count);
    clone_big_x_bytes(br, &mut out, byte_count)?;
    Ok(out)
}
/// read_x_bytes, but optimized for bigger things, such as 3d-models
pub fn clone_big_x_bytes(
    read_br: &mut dyn Read,
    write_br: &mut dyn Write,
    byte_count: usize,
) -> Result<()> {
    let mut buf: [u8; 1024] = [0; 1024];
    for _ in 1..=(byte_count >> 10) {
        read_br.read_exact(&mut buf)?;
        write_br.write_all(&buf)?;
    }
    let mut buf: [u8; 1] = [0];
    for _ in 1..=(byte_count % 1024) {
        read_br.read_exact(&mut buf)?;
        write_br.write_all(&buf)?;
    }
    Ok(())
}

pub fn write_u32_le(num: u32, bw: &mut dyn Write) -> Result<()> {
    bw.write_all(&num.to_le_bytes())?;
    Ok(())
}
pub fn write_u32_be(num: u32, bw: &mut dyn Write) -> Result<()> {
    bw.write_all(&num.to_be_bytes())?;
    Ok(())
}

#[cfg(any(target_pointer_width = "16", target_pointer_width = "32"))]
compile_error!("Only systems with a pointer width of at least 64bit are supported (not sure how you are installing modern ps2 on a ThinkPad 300 anyway).");
