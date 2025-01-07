use nu_plugin::{serve_plugin, MsgPackSerializer};
use nu_plugin_nups2;

fn main() {
    serve_plugin(&nu_plugin_nups2::Nups2Plugin, MsgPackSerializer {})
}
