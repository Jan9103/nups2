pub mod crc64;
pub mod pack2;
pub mod util;

use nu_plugin::{Plugin, PluginCommand};

pub struct Nups2Plugin;

impl Plugin for Nups2Plugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(pack2::Pack2LsCommand),
            Box::new(pack2::Pack2ExtractCommand),
            Box::new(crc64::Ps2Crc64Command),
            Box::new(pack2::Pack2ScrapeFilenamesCommand),
            Box::new(pack2::Pack2ManifestCreate),
        ]
    }
}
