def main [
  new_manifest_dir: path
  old_manifest_dir: path
  --nups2-bin: path = "./crates/nups2/target/release/nups2"
  --filename-list-file: path = "./ultimate_namelist.txt"
] {
  for pack2file in (ls $old_manifest_dir | get name | where $it =~ '\.pack2\.manifest$' | path basename) {
    print $'### Diff ($pack2file) ###'
    ^$nups2_bin manifest-diff-with-another ($new_manifest_dir | path join $pack2file) ($old_manifest_dir | path join $pack2file) --filename-list-file $filename_list_file
  }
}
