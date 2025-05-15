use crate::bin_utils::*;
use std::io::Read;

#[derive(Debug)]
pub struct Dma {
    version: u32,
    materials: Vec<DmaMaterial>,
}

#[derive(Debug)]
pub struct DmaMaterial {
    name_hash: u32,
    material_definition_hash: u32,
    parameters: Vec<DmaParameter>,
}

#[derive(Debug)]
pub struct DmaParameter {
    name_hash: u32,
    d3dx_parameter_class: D3dxparameterClass,
    d3dx_parameter_type: D3dxparameterType,
    data: Vec<u8>,
}

impl Dma {
    pub fn read(br: &mut dyn Read) -> std::io::Result<Self> {
        log::trace!("started reading dma");
        let magic_value: u32 = read_u32_be(br)?;
        assert_eq!(magic_value, 0x444d4154u32, "Magick value of DMA is wrong");
        let version: u32 = read_u32_le(br)?;

        let var_a_length: u32 = read_u32_le(br)?;
        let mut var_a: Vec<u8> = Vec::with_capacity(var_a_length as usize);
        let mut buf: [u8; 1] = [0u8];
        for _ in 1..=var_a_length {
            br.read_exact(&mut buf)?;
            var_a.push(buf[0]);
        }
        // maybe filename? usually empty

        let material_count: u32 = read_u32_le(br)?;
        let mut materials: Vec<DmaMaterial> = Vec::with_capacity(material_count as usize);
        for _ in 1..=material_count {
            materials.push(DmaMaterial::read(br)?);
        }
        log::trace!("finished reading dma");
        Ok(Self { version, materials })
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> String {
        format!(
            "{o}\"version\": {version}, \"materials\": [{materials}]{c}",
            o = "{",
            c = "}",
            version = self.version,
            materials = self
                .materials
                .iter()
                .map(|i| i.to_json())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

impl DmaMaterial {
    pub fn read(br: &mut dyn Read) -> std::io::Result<Self> {
        let name_hash: u32 = read_u32_le(br)?;
        let _data_length: u32 = read_u32_le(br)?;
        let material_definition_hash: u32 = read_u32_le(br)?;

        let parameter_count: u32 = read_u32_le(br)?;
        let mut parameters: Vec<DmaParameter> = Vec::with_capacity(parameter_count as usize);
        for _ in 1..=parameter_count {
            parameters.push(DmaParameter::read(br)?);
        }

        Ok(Self {
            name_hash,
            material_definition_hash,
            parameters,
        })
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> String {
        format!(
            "{o}\"name_hash\": {name_hash}, \"material_definition_hash\": {material_definition_hash}, \"parameters\": [{parameters}]{c}",
            o = "{",
            c = "}",
            name_hash = self.name_hash,
            material_definition_hash = self.material_definition_hash,
            parameters = self.parameters.iter().map(|i| i.to_json()).collect::<Vec<String>>().join(", "),
        )
    }
}

impl DmaParameter {
    pub fn read(br: &mut dyn Read) -> std::io::Result<Self> {
        let name_hash: u32 = read_u32_le(br)?;
        let d3dx_parameter_class: D3dxparameterClass =
            D3dxparameterClass::try_from(read_u32_le(br)?).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid D3dxparameterClass in dma",
                )
            })?;
        let d3dx_parameter_type: D3dxparameterType = D3dxparameterType::try_from(read_u32_le(br)?)
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid D3dxparameterType in dma",
                )
            })?;

        let data_length: u32 = read_u32_le(br)?;
        let mut data: Vec<u8> = Vec::with_capacity(data_length as usize);
        let mut buf: [u8; 1] = [0u8];
        for _ in 1..=data_length {
            br.read_exact(&mut buf)?;
            data.push(buf[0]);
        }

        Ok(Self {
            name_hash,
            d3dx_parameter_class,
            d3dx_parameter_type,
            data,
        })
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> String {
        format!(
            "{o}\"name_hash\": {name_hash}, \"d3dx_parameter_class\": \"{d3dx_parameter_class:?}\", \"d3dx_parameter_type\": \"{d3dx_parameter_type:?}\", \"data\": [{data}]{c}",
            o = "{",
            c = "}",
            name_hash = self.name_hash,
            d3dx_parameter_class = self.d3dx_parameter_class,
            d3dx_parameter_type = self.d3dx_parameter_type,
            data = self.data.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(", "),
        )
    }
}

#[derive(Debug)]
pub enum D3dxparameterClass {
    Scalar = 0,
    Vector = 1,
    MatrixRows = 2,
    MatrixColumns = 3,
    Object = 4,
    Struct = 5,
    ForceDword = 0x7fffffff,
}

impl TryFrom<u32> for D3dxparameterClass {
    type Error = ();
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        use D3dxparameterClass::*;
        Ok(match v {
            0 => Scalar,
            1 => Vector,
            2 => MatrixRows,
            3 => Object,
            4 => Struct,
            0x7fffffff => ForceDword,
            _ => return Err(()),
        })
    }
}

#[derive(Debug)]
pub enum D3dxparameterType {
    Void = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
    String = 4,
    Texture = 5,
    Texture1d = 6,
    Texture2d = 7,
    Texture3d = 8,
    Texturecube = 9,
    Sampler = 10,
    Sampler1d = 11,
    Sampler2d = 12,
    Sampler3d = 13,
    Samplercube = 14,
    Pixelshader = 15,
    Vertexshader = 16,
    Pixelfragment = 17,
    Vertexfragment = 18,
    Unsupported = 19,
    ForceDword = 0x7fffffff,
}

impl TryFrom<u32> for D3dxparameterType {
    type Error = ();
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        use D3dxparameterType::*;
        Ok(match v {
            0 => Void,
            1 => Bool,
            2 => Int,
            3 => Float,
            4 => String,
            5 => Texture,
            6 => Texture1d,
            7 => Texture2d,
            8 => Texture3d,
            9 => Texturecube,
            10 => Sampler,
            11 => Sampler1d,
            12 => Sampler2d,
            13 => Sampler3d,
            14 => Samplercube,
            15 => Pixelshader,
            16 => Vertexshader,
            17 => Pixelfragment,
            18 => Vertexfragment,
            19 => Unsupported,
            0x7fffffff => ForceDword,
            _ => return Err(()),
        })
    }
}
