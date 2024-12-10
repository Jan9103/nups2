# DME file format

**Credits:** My understanding is based on [ps2ls2][]'s implementation.

## Header

1. 4bytes: magic value (u32 BE: `0x444d4f44`)
2. u32 le: version

## Version 4

1. (header)
2. u32 le: DMA length (depending on the implementation part of the DMA-parser instead)
3. 1x [DMA](./dma.md) (probably DMA-length bytes long, unconfirmed)
4. 24 bytes: bounding box: 2x (3x (f32 le))
5. u32 le: mesh count
6. meshes (see below)
7. u32 le: bone draw call count
8. bone draw calls (see below)
9. u32 le: bone map entry count
10. (`$bone_map_entry_count * 32`) bytes: bone map entries (see below)
11. u32 le: bone count
12. bone parsing (see below, quite special parsing needed)

### meshes

1. u32 le: draw_call_offset
2. u32 le: draw_call_count
3. u32 le: bone_transformation_count
4. 4 bytes: unknown
5. u32 le: vertex_stream_count
6. u32 le: index_size
7. 4 bytes: unknown
8. u32 le: index_count
9. u32 le: vertex_count
10. vertex streams:
  1. u32 le: bytes_per_vertex
  2. (`bytes_per_vertex * vertex_count`) bytes: data
11. (`$index_size * $index_count`) bytes: index_data

### bone draw calls

1. 4 bytes: unknown
2. u32 le: bone_start
3. u32 le: bone_count
4. u32 le: delta
5. 4 bytes: unknown
6. u32 le: vertex_offset
7. u32 le: vertex_count
8. u32 le: index_offset
9. u32 le: index_count

### bone map entries

1. u16 le: bone_index
2. u16 le: global_index

### bone parsing

**IMPORTANT:** The bones are mixed. you don't parse one by one, but all at once.  
Therefore run the following once and not once per bone-count.

1. inverse_bind_poses: once per bone: 4x4 f32 matrix:
  * `[[read, read, read, 0f32], [read, read, read, 0f32], [read, read, read, 0f32], [read, read, read, 1f32]]` (read in this order)
2. min and max values: once per bone:
  1. 3x f32 le: min value
  2. 3x f32 le: max value
3. name hashes: once per bone: u32 le



[ps2ls2]: https://github.com/NatCracken/ps2ls2
