use crate::SpeexBits;
use speex_sys::{
    speex_lib_get_mode, SpeexMode, SPEEX_MODEID_NB, SPEEX_MODEID_UWB, SPEEX_MODEID_WB,
};
use std::ffi::c_void;
use std::marker::{PhantomData, PhantomPinned};

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ModeId {
    NarrowBand = SPEEX_MODEID_NB,
    WideBand = SPEEX_MODEID_WB,
    UltraWideBand = SPEEX_MODEID_UWB,
}

impl ModeId {
    pub fn get_mode(self) -> &'static SpeexMode {
        speex_sys::speex_lib_get_mode(self as i32)
    }
}

impl SpeexMode {
    pub fn new(mode_id: ModeId) -> Self {
        let backing = unsafe { speex_lib_get_mode(mode_id as i32) };
        Self(backing)
    }
}

#[repr(C)]
pub struct SpeexEncoderHandle {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

impl SpeexEncoderHandle {
    pub fn create(mode: &SpeexMode) -> *mut Self {
        let ptr = unsafe {
            let mode_ptr = mode.0 as *const SysMode;
            speex_sys::speex_encoder_init(mode_ptr)
        };
        ptr as *mut SpeexEncoderHandle
    }

    pub fn destroy(handle: *mut SpeexEncoderHandle) {
        unsafe { speex_sys::speex_encoder_destroy(handle as *mut c_void) }
    }
}

pub struct SpeexEncoder<'a> {
    encoder_handle: *mut SpeexEncoderHandle,
    mode: SpeexMode,
    bits: SpeexBits<'a>,
}

pub struct SpeexDecoder {
    backing: SysMode,
}
