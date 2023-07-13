use speex_sys::{
    speex_lib_get_mode, SpeexMode as SysMode, SPEEX_MODEID_NB, SPEEX_MODEID_UWB, SPEEX_MODEID_WB,
};

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum ModeId {
    NarrowBand = SPEEX_MODEID_NB,
    WideBand = SPEEX_MODEID_WB,
    UltraWideBand = SPEEX_MODEID_UWB,
}

impl ModeId {
    pub fn get_mode(self) -> SpeexMode {
        SpeexMode::new(self)
    }
}

pub struct SpeexMode {
    pub backing: *const SysMode,
}

impl SpeexMode {
    pub fn new(mode_id: ModeId) -> Self {
        let backing = unsafe { speex_lib_get_mode(mode_id as i32) };
        Self { backing }
    }
}

pub struct SpeexEncoder {
    backing: SysMode,
}

pub struct SpeexDecoder {
    backing: SysMode,
}
