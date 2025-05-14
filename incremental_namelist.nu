#!/usr/bin/env nu

def main [
  --planetside-directory: path = "~/.var/app/com.valvesoftware.Steam/.steam/steam/steamapps/common/PlanetSide 2"
  --nups2-bin: path = "./crates/nups2/target/release/nups2"
  --final-file: path = "./ultimate_namelist.txt"
  --scrape-mode: int = 3
  --backup-old-namelist = true
  manifest_dir: path
] {
  let manifest_dir: path = ($manifest_dir | path expand)
  let final_file = ($final_file | path expand)
  let old_namelist: path = $'($final_file).back'
  mv --force $final_file $old_namelist

  let nups2_bin: path = ($nups2_bin | path expand)

  cd $planetside_directory
  let pack_files: list<string> = (ls **/*.pack2 | each {|file| $file.name | path expand})

  let tmpdir = (mktemp --directory)
  cd $tmpdir
  let thread_count: int = (([5, (sys cpu | length)] | math max) - 4)
  $pack_files
  | par-each --threads $thread_count {|pack_file|
    print $"Scraping ($pack_file | path basename).."
    let manifest: path = ($manifest_dir | path join $'($pack_file | path split | last).manifest')
    ^$nups2_bin "pack2-scrape-filenames" $pack_file $'($pack_file | path basename).namelist.txt' --scrape-mode $scrape_mode --manifest-from-last-scrape $manifest
    null
  }

  print "Combinding scraped namelists into one.."
  ls
  | each {|file| open --raw $file.name | lines}
  | flatten
  | append (scrape_ui_xml $planetside_directory)
  | append (open --raw $old_namelist | lines)
  | uniq -i
  | sort -i
  | str join "\n"
  | save --raw $final_file

  cd  # deleting the dir were in can be bad
  rm --permanent --recursive $tmpdir
}

def scrape_ui_xml [ps2_dir: path]: nothing -> list<string> {
  cd ($ps2_dir | path join 'UI' 'UiModules' 'Main')
  ^rg -Io '[a-zA-Z0-9_-]+\.swf'
  | lines
}
