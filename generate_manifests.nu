def main [
  --planetside-directory: path = "~/.var/app/com.valvesoftware.Steam/.steam/steam/steamapps/common/PlanetSide 2"
  --nups2-bin: path = "./crates/nups2/target/release/nups2"
  --manifest-dir: path = "./manifests"
] {
  let nups2_bin: path = ($nups2_bin | path expand)
  let manifest_dir: path = ($manifest_dir | path join (date now | format date "%F_%H-%M") | path expand)
  mkdir $manifest_dir

  cd $planetside_directory
  let pack_files: list<string> = (ls **/*.pack2 | each {|file| $file.name | path expand})

  let thread_count: int = (([5, (sys cpu | length)] | math max) - 4)
  $pack_files
  | par-each --threads $thread_count {|pack_file|
    print $"Scraping ($pack_file | path basename)"
    let output_file: path = ($manifest_dir | path join $'($pack_file | path basename).manifest')
    ^$nups2_bin pack2-generate-manifest $pack_file $output_file
    null
  }
  null
}
