use crate::mode::ModeId;
use speex_sys::SpeexMode;
use std::ffi::c_void;
use std::marker::{PhantomData, PhantomPinned};

#[repr(C)]
pub struct SpeexDecoderHandle {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

impl SpeexDecoderHandle {
    pub fn create(mode: &SpeexMode) -> *mut Self {
        let ptr = unsafe {
            let mode_ptr = mode as *const SpeexMode;
            speex_sys::speex_decoder_init(mode_ptr)
        };
        ptr as *mut SpeexDecoderHandle
    }

    pub fn destroy(handle: *mut SpeexDecoderHandle) {
        unsafe {
            speex_sys::speex_decoder_destroy(handle as *mut c_void);
        }
    }
}

pub struct SpeexDecoder {
    encoder_handle: *mut SpeexDecoderHandle,
    mode: &'static SpeexMode,
}

impl SpeexDecoder {
    pub fn new(mode: ModeId) -> Self {
        let mode = mode.get_mode();
        let encoder_handle = SpeexDecoderHandle::create(mode);
        Self {
            encoder_handle,
            mode,
        }
    }
}

impl Drop for SpeexDecoder {
    fn drop(&mut self) {
        SpeexDecoderHandle::destroy(self.encoder_handle);
    }
}
