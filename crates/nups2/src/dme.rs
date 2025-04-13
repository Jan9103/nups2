use crate::bin_utils::*;
use crate::dma::Dma;
use std::collections::HashMap;
use std::io::Read;

pub type Vector4 = (f32, f32, f32, f32);
pub type Vector3 = (f32, f32, f32);
pub type Matrix4x4 = (Vector4, Vector4, Vector4, Vector4);

#[derive(Debug)]
pub struct Dme {
    bounding_box: (Vector3, Vector3),
    dma: Dma,
    meshes: Vec<DmeMesh>,
    bone_draw_calls: Vec<DmeBoneDrawCall>,
    internal_bone_map_entries: Vec<DmeBoneMapEntry>,
    bones: Vec<DmeBone>,
}

impl Dme {
    pub fn read(br: &mut dyn Read) -> std::io::Result<Self> {
        log::trace!("[Dme::read] start");
        let magic: u32 = read_u32_be(br)?;
        assert_eq!(magic, 0x444d4f44, "DME file invalid (missing magick value)");
        let version: u32 = read_u32_le(br)?;
        assert_eq!(version, 4, "Unsupported DME version (only v4 is supported)");
        match version {
            4 => Self::internal_read_v4(br),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unsupported DME version ({})", version),
            )),
        }
    }

    /// DO NOT CALL FROM OUTSIDE
    /// called by read if it determines its v4 after changing some things
    fn internal_read_v4(br: &mut dyn Read) -> std::io::Result<Self> {
        let _dma_length: u32 = read_u32_le(br)?;
        let dma: Dma = Dma::read(br)?;
        let bounding_box_min: Vector3 = read_vector3(br)?;
        let bounding_box_max: Vector3 = read_vector3(br)?;

        let mesh_count: u32 = read_u32_le(br)?;
        log::debug!("[Dme::read] reading meshes ({})", mesh_count);
        let mut meshes: Vec<DmeMesh> = Vec::with_capacity(mesh_count as usize);
        for _ in 1..=mesh_count {
            let mesh = DmeMesh::read(br)?;
            meshes.push(mesh);
        }

        let bone_draw_call_count: u32 = read_u32_le(br)?;
        log::debug!(
            "[Dme::read] reading bone draw calls ({})",
            bone_draw_call_count
        );
        let mut bone_draw_calls: Vec<DmeBoneDrawCall> =
            Vec::with_capacity(bone_draw_call_count as usize);
        for _ in 1..=bone_draw_call_count {
            bone_draw_calls.push(DmeBoneDrawCall::read(br)?);
        }

        let bone_map_entry_count: u32 = read_u32_le(br)?;
        log::debug!("[Dme::read] reading bone map ({})", bone_map_entry_count);
        let mut bone_map_entries: Vec<DmeBoneMapEntry> =
            Vec::with_capacity(bone_map_entry_count as usize);
        for _ in 1..=bone_map_entry_count {
            let bone_map_entry: DmeBoneMapEntry = DmeBoneMapEntry::read(br)?;
            bone_map_entries.push(bone_map_entry);
        }

        let bone_count: u32 = read_u32_le(br)?;
        log::debug!(
            "[Dme::read] reading bone inverse_bind_poses ({})",
            bone_count
        );
        let mut bone_inverse_bind_poses: Vec<Matrix4x4> = Vec::with_capacity(bone_count as usize);
        for _ in 1..=bone_count {
            let v1: Vector4 = read_vector3_plus1(br, 0f32)?;
            let v2: Vector4 = read_vector3_plus1(br, 0f32)?;
            let v3: Vector4 = read_vector3_plus1(br, 0f32)?;
            let v4: Vector4 = read_vector3_plus1(br, 1f32)?;
            bone_inverse_bind_poses.push((v1, v2, v3, v4));
        }
        log::debug!("[Dme::read] reading bone min_max");
        let mut bone_min_max: Vec<(Vector3, Vector3)> = Vec::with_capacity(bone_count as usize);
        for _ in 1..=bone_count {
            let min: Vector3 = read_vector3(br)?;
            let max: Vector3 = read_vector3(br)?;
            bone_min_max.push((min, max));
        }
        log::debug!("[Dme::read] reading bone name_hash");
        let mut bones: Vec<DmeBone> = Vec::with_capacity(bone_count as usize);
        for i in 0..(bone_count as usize) {
            let name_hash: u32 = read_u32_le(br)?;
            let mm: (Vector3, Vector3) = bone_min_max[i];
            bones.push(DmeBone {
                inverse_bind_pose: bone_inverse_bind_poses[i],
                min: mm.0,
                max: mm.1,
                name_hash,
            });
        }

        log::debug!("[Dme::read] finished");
        Ok(Self {
            bounding_box: (bounding_box_min, bounding_box_max),
            dma,
            meshes,
            bone_draw_calls,
            internal_bone_map_entries: bone_map_entries,
            bones,
        })
    }

    pub fn get_vertex_count(&self) -> u32 {
        self.meshes.iter().map(|mesh| mesh.vertex_count).sum()
    }

    pub fn get_index_count(&self) -> u32 {
        self.meshes.iter().map(|mesh| mesh.vertex_count).sum()
    }

    pub fn build_bonemaps(&self) -> (HashMap<u16, u16>, HashMap<u16, u16>) {
        let mut bone_map_1: HashMap<u16, u16> =
            HashMap::with_capacity(self.internal_bone_map_entries.len());
        let mut bone_map_2: HashMap<u16, u16> = HashMap::new();
        for bone_map_entry in self.internal_bone_map_entries.iter() {
            if bone_map_1.contains_key(&bone_map_entry.global_index) {
                bone_map_1.insert(bone_map_entry.global_index + 64, bone_map_entry.bone_index);
                bone_map_2.insert(bone_map_entry.global_index, bone_map_entry.bone_index);
            } else {
                bone_map_1.insert(bone_map_entry.global_index, bone_map_entry.bone_index);
            }
        }
        (bone_map_1, bone_map_2)
    }
}

#[derive(Debug)]
pub struct DmeMesh {
    vertex_count: u32,
    index_count: u32,
    draw_call_offset: u32,
    draw_call_count: u32,
    bone_transformation_count: u32,
    index_data: Vec<u8>,
    vertex_streams: Vec<DmeVertexStream>,
}
impl DmeMesh {
    pub fn read(br: &mut dyn Read) -> std::io::Result<Self> {
        let draw_call_offset: u32 = read_u32_le(br)?;
        let draw_call_count: u32 = read_u32_le(br)?;
        let bone_transformation_count: u32 = read_u32_le(br)?;
        let _unknown: u32 = read_u32_le(br)?;
        let vertex_stream_count: u32 = read_u32_le(br)?;
        let index_size: u16 = read_u16_le(br)?; // byte length of each index (2 is u16, 4 is u32)
        let _unknown: u16 = read_u16_le(br)?; // ignore the upper bytes of index_size
        let index_count: u32 = read_u32_le(br)?;
        let vertex_count: u32 = read_u32_le(br)?;

        let mut vertex_streams: Vec<DmeVertexStream> =
            Vec::with_capacity(vertex_stream_count as usize);
        for _ in 1..=vertex_stream_count {
            let vertex_stream: DmeVertexStream = DmeVertexStream::read(br, vertex_count)?;
            vertex_streams.push(vertex_stream);
        }

        let index_byte_length: u32 = (index_size as u32) * index_count;
        let index_data: Vec<u8> = read_x_bytes(br, index_byte_length as usize)?;

        Ok(Self {
            vertex_count,
            index_count,
            draw_call_offset,
            draw_call_count,
            bone_transformation_count,
            index_data,
            vertex_streams,
        })
    }
}

#[derive(Debug)]
pub struct DmeBoneDrawCall {
    bone_start: u32,
    bone_count: u32,
    delta: u32,
    vertex_offset: u32,
    vertex_count: u32,
    index_offset: u32,
    index_count: u32,
}
impl DmeBoneDrawCall {
    pub fn read(br: &mut dyn Read) -> std::io::Result<Self> {
        let _unknown: u32 = read_u32_le(br)?;
        let bone_start: u32 = read_u32_le(br)?;
        let bone_count: u32 = read_u32_le(br)?;
        let delta: u32 = read_u32_le(br)?;
        let _unknown: u32 = read_u32_le(br)?;
        let vertex_offset: u32 = read_u32_le(br)?;
        let vertex_count: u32 = read_u32_le(br)?;
        let index_offset: u32 = read_u32_le(br)?;
        let index_count: u32 = read_u32_le(br)?;
        Ok(Self {
            bone_start,
            bone_count,
            vertex_offset,
            delta,
            vertex_count,
            index_offset,
            index_count,
        })
    }
}

#[derive(Debug)]
pub struct DmeBoneMapEntry {
    bone_index: u16,
    global_index: u16,
}
impl DmeBoneMapEntry {
    pub fn read(br: &mut dyn Read) -> std::io::Result<Self> {
        let bone_index: u16 = read_u16_le(br)?;
        let global_index: u16 = read_u16_le(br)?;
        Ok(Self {
            bone_index,
            global_index,
        })
    }
}

#[derive(Debug)]
pub struct DmeBone {
    inverse_bind_pose: Matrix4x4,
    min: Vector3,
    max: Vector3,
    name_hash: u32,
}
impl DmeBone {}

#[derive(Debug)]
pub struct DmeVertexStream {
    bytes_per_vertex: u32,
    data: Vec<u8>,
}
impl DmeVertexStream {
    pub fn read(br: &mut dyn Read, vertex_count: u32) -> std::io::Result<Self> {
        let bytes_per_vertex: u32 = read_u32_le(br)?;
        let data: Vec<u8> = read_x_bytes(br, (bytes_per_vertex * vertex_count) as usize)?;
        Ok(Self {
            bytes_per_vertex,
            data,
        })
    }
}

fn read_vector3(br: &mut dyn Read) -> std::io::Result<Vector3> {
    let x: f32 = read_f32_le(br)?;
    let y: f32 = read_f32_le(br)?;
    let z: f32 = read_f32_le(br)?;
    Ok((x, y, z))
}

fn read_vector3_plus1(br: &mut dyn Read, value4: f32) -> std::io::Result<Vector4> {
    let x: f32 = read_f32_le(br)?;
    let y: f32 = read_f32_le(br)?;
    let z: f32 = read_f32_le(br)?;
    Ok((x, y, z, value4))
}
