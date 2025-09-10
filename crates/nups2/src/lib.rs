#[cfg(feature = "adr")]
pub mod adr;
pub mod bin_utils;
#[cfg(feature = "cli")]
pub mod cli;
pub mod cli_utils;
pub mod crc32;
pub mod crc64;
#[cfg(feature = "dma")]
pub mod dma;
#[cfg(feature = "dme")]
pub mod dme;
mod error;
#[cfg(feature = "filename_scraper")]
pub mod filename_extractor;
#[cfg(feature = "fsb")]
pub mod fsb5;
pub mod json_utils;
#[cfg(feature = "pack1")]
pub mod pack1;
pub mod pack2;
#[cfg(feature = "manifests")]
pub mod pack2_manifest;
#[cfg(feature = "rainbow_table")]
pub mod rainbow_table;
pub use error::Nups2Error;

//pub mod jenkins_hash;
//pub mod to_glb;

/// mainly here in case some platform (risc5 or whatever) does something unexpected.
/// should never fail if i understand rust correctly.
#[cfg(test)]
mod sanity_checks {
    #[test]
    fn number_casting() {
        assert_eq!(1u32 as u64, 1u64);
    }
}
