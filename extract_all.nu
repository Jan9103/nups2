def main [
  --planetside-directory: path = "~/.var/app/com.valvesoftware.Steam/.steam/steam/steamapps/common/PlanetSide 2"
  --nups2-bin: path = "./crates/nups2/target/release/nups2"
  --extract-dir: path = "./extracted"
  --thread-count: int
  --namelist: path = "./ultimate_namelist.txt"
]: nothing -> nothing {
  let nups2_bin: path = ($nups2_bin | path expand)
  let extract_dir: path = ($extract_dir | path expand)
  let namelist: path = ($namelist | path expand)
  mkdir $extract_dir

  cd $planetside_directory
  let pack_files: list<string> = (ls **/*.pack2 | each {|file| $file.name | path expand})

  let thread_count: int = ($thread_count | default (([5, (sys cpu | length)] | math max) - 4))
  $pack_files
  | par-each --threads $thread_count {|pack_file|
    print $"Unpacking ($pack_file | path basename)"
    let output_dir: path = ($extract_dir | path join ($pack_file | path parse | get stem))
    mkdir $output_dir
    print $'^$nups2_bin pack2-extract-all --output-dir ($output_dir) --filename-list-file ($namelist) ($pack_file)'
    ^$nups2_bin pack2-extract-all --output-dir $output_dir --filename-list-file $namelist $pack_file
    null
  }
  null
}
