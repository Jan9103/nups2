#[derive(Debug)]
pub enum SoundContainer {
    Unknown,
    Mp3,
    Ogg,
    Wav,
}

impl SoundContainer {
    pub fn file_extension(&self) -> &'static str {
        match self {
            SoundContainer::Unknown => "bin",
            SoundContainer::Mp3 => "mp3",
            SoundContainer::Ogg => "ogg",
            SoundContainer::Wav => "wav",
        }
    }
}

#[derive(Debug)]
pub struct SoundFormat {
    pub id: u8,
    pub name: &'static str,
    pub sound_container: SoundContainer,
}
impl SoundFormat {
    pub fn get_by_id(wanted_id: u8) -> Option<&'static Self> {
        match SOUND_FORMATS.iter().find(|i| i.id == wanted_id) {
            Some(v) => Some(*v),
            None => None,
        }
    }
}

const SOUND_FORMATS: &[&SoundFormat] = &[
    &SoundFormat {
        id: 0,
        name: "none",
        sound_container: SoundContainer::Unknown,
    },
    &SoundFormat {
        id: 1,
        name: "pcm8",
        sound_container: SoundContainer::Wav,
    },
    &SoundFormat {
        id: 2,
        name: "pc16",
        sound_container: SoundContainer::Wav,
    },
    &SoundFormat {
        id: 3,
        name: "pcm24",
        sound_container: SoundContainer::Unknown,
    },
    &SoundFormat {
        id: 4,
        name: "pcm32",
        sound_container: SoundContainer::Wav,
    },
    &SoundFormat {
        id: 5,
        name: "pcm_float",
        sound_container: SoundContainer::Unknown,
    },
    // used by nintendo
    // lib: https://crates.io/crates/gc_adpcm
    &SoundFormat {
        id: 6,
        name: "gcadpcm",
        sound_container: SoundContainer::Unknown,
    },
    // there are multiple with this format..
    // my ffmpeg alone supports: alp, amv, apc, apm, cunning ,dat4, dka3, dka4, es_eacs, ea_sead, iss, moflex, mtf, oki, qt, rad, smjpeg, ssi, wav, westwood
    &SoundFormat {
        id: 7,
        name: "imaadpcm",
        sound_container: SoundContainer::Unknown,
    },
    &SoundFormat {
        id: 8,
        name: "vag",
        sound_container: SoundContainer::Unknown,
    },
    &SoundFormat {
        id: 9,
        name: "hevag",
        sound_container: SoundContainer::Unknown,
    },
    &SoundFormat {
        id: 10,
        name: "xma",
        sound_container: SoundContainer::Unknown,
    },
    &SoundFormat {
        id: 11,
        name: "mpeg",
        sound_container: SoundContainer::Mp3,
    },
    &SoundFormat {
        id: 12,
        name: "celt",
        sound_container: SoundContainer::Unknown,
    },
    &SoundFormat {
        id: 13,
        name: "alt9",
        sound_container: SoundContainer::Unknown,
    },
    &SoundFormat {
        id: 14,
        name: "xwma",
        sound_container: SoundContainer::Unknown,
    },
    &SoundFormat {
        id: 15,
        name: "vorbis",
        sound_container: SoundContainer::Ogg,
    },
];
