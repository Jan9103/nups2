# Pack1 (aka pack)

Planetside now only uses [pack2](./pack2.md), but some tools and other games still operate on pack1.

Pack1 similar to a tar file contains multiple other files.

## Pack1 layout

* repeat:
  * u32 BE: offset (from filestart) of where the next chunk starts
    * if `== 0` the end of the pack file has been reached
  * chunk
  * (potentially garbage data - use the offset)

## Chunk layout

* u32 BE: asset count
* repeat:
  * asset

## Asset layout

* u32 BE: name length
* `$name_length` bytes: name bytes (utf-8?)
* u32 BE: offset (from filestart) of where the actual data is located
* u32 BE: data length in bytes
* u32 BE: file hash (presumably crc?)
