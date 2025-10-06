const TILE_RESOLUTION: int = 256

def main [
  continent: string = "Esamir"
  output_file: path = "continent_map.avif"
] {
  let continent = ($continent | str pascal-case)

  let dds_files: list<path> = (
    ^fd $"^\(?i\)($continent)_Tile_[0-9-]{3}_[0-9-]{3}_LOD0.dds$"
    | lines
    | each {|file_path|
      let p = ($file_path | parse --regex '_(?P<c1>[0-9-]{3})_(?P<c2>[0-9-]{3})_.*0.dds$' | into int c1 c2).0
      {
        "path": ($file_path | path expand)
        "c1": $p.c1
        "c2": $p.c2
      }
    }
  )
  let c1 = ($dds_files.c1 | uniq | sort)
  let c2 = ($dds_files.c2 | uniq | sort)

  let output_size: int = $TILE_RESOLUTION * ($c1 | length)

  let args = [
    '-tile' $'($c1 | length)x0'
    '-geometry' '+0+0'
    '-border' '0'
    '-define' 'image-quality=100'
    '-quality' '100'
    ...($dds_files | sort-by c2 c1).path
    $output_file
  ]

  print ($args | to json)

  ^montage ...$args
}
