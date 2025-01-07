pub fn humanise_bytes(bytes: f64) -> String {
    if bytes > 1073741824.0 {
        return format!("{:.2}GB", (bytes / 1073741824.0));
    }
    if bytes > 1048576.0 {
        return format!("{:.2}MB", (bytes / 1048576.0));
    }
    if bytes > 1024.0 {
        return format!("{:.2}KB", (bytes / 1024.0));
    }
    format!("{:.0}Bytes", bytes)
}
