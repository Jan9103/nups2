# Pack2 file format

Pack2 is similar to a tar file and contains a bunch of other files.

Pack1 also existed, but planetside2 no longer uses it. AFAIK pack1 had less edit-protection.

Special thanks to [dbg-pack](https://github.com/brhumphe/dbg-pack) for having a non-code explanation.

## Header

1. 4 bytes: magic value `0x50414b01` (ascii: `PAK `)
2. u32 LE: asset count
3. u64 LE: length (useless?)
4. u64 LE: map location in bytes from file-start
5. u32 LE: unknown (probably crc64 hash)
6. 128 bytes: unknown

## Map

The map lists all files contained.

The map conists of a bunch of file-entries. Just repeat the parsing for the following once per asset-count (from header).

### Map-asset

1. u64 LE: name hash (crc64 jones algorhytm. see `src/crc64.rs` for a encoder implementation)
2. u64 LE: data location in bytes from file-start
3. u64 LE: data length in bytes
4. u32 LE: flags for the file. bits:
  * `flags & 0x1 == 0x1`: file is compressed with zlib (see below for more info). also data length has to be non 0.
  * `flags & 0x10 == 0x10`: unknown, but often set
  * other: unknown and i have never seen them set.
5. u32 LE: data hash (probably crc64 jones again, but never tested or verified this)

## data

### uncompressed data

just filedata. just copy the bytes (start location and length) to a new file and youre done with extracting.

### compressed data

1. u32 BE: magick value indicating this beeing compressed (`0xA1B2C3D4`)
2. u32 BE: uncompressed data length
3. after: [zlib][] compressed data (i just tell zlib to read until uncomprssed-data-length is reached)

## special data

### filename-list

**NOTE:** This seems to not be present in any current gamefiles.

**Filename hash:** `0x4137cc65bd97fd30`.  
**Format:** newline seperated list of filenames included in the `pack2` file.


[zlib]: https://en.wikipedia.org/wiki/Zlib

