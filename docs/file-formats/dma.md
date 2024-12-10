# Dma file format

A file containing textures.

Documentation based on [ps2ls2](https://github.com/NatCracken/ps2ls2)'s c# implementation.

## Header

1. 4 bytes: magic value `0x444d4154` (ascii: `DMAT`)
2. u32 LE: version
3. u32 LE: $varA (ps2ls2 calls it `filenameLength`, but that feels wrong)
4. $varA bytes: see $varB
5. u32 LE: material count
6. $material_count material: see material

## $varB

## material

1. u32 (?): name hash
2. u32 LE: data length (meant for fast-read?)
3. u32 (?): material definition hash
4. u32 LE: parameter count
5. $parameter_count parameter: see parameter

## parameter

1. u32 (?): name hash
2. 4 bytes: D3DXParameterClass (<http://msdn.microsoft.com/en-us/library/windows/desktop/bb205378(v=vs.85).aspx>)
3. 4 bytes: D3DXParameterType (<http://msdn.microsoft.com/en-us/library/windows/desktop/bb205380(v=vs.85).aspx>)
4. u32 LE: data length
5. $data_length bytes: data

