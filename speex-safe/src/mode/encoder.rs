use crate::mode::{CoderMode, ControlError, ControlFunctions, ModeId, NbMode, UwbMode, WbMode};
use speex_sys::SpeexMode;
use std::ffi::c_void;
use std::marker::{PhantomData, PhantomPinned};

#[repr(C)]
pub struct SpeexEncoderHandle {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

impl SpeexEncoderHandle {
    pub fn create(mode: &SpeexMode) -> *mut Self {
        let ptr = unsafe {
            let mode_ptr = mode as *const SpeexMode;
            speex_sys::speex_encoder_init(mode_ptr)
        };
        ptr as *mut SpeexEncoderHandle
    }

    pub fn destroy(handle: *mut SpeexEncoderHandle) {
        unsafe { speex_sys::speex_encoder_destroy(handle as *mut c_void) }
    }
}

pub struct SpeexEncoder<T: CoderMode> {
    encoder_handle: *mut SpeexEncoderHandle,
    mode: &'static SpeexMode,
    _phantom: PhantomData<T>,
}

impl<T: CoderMode> ControlFunctions for SpeexEncoder<T> {
    unsafe fn ctl(&mut self, request: i32, ptr: *mut c_void) -> Result<(), ControlError> {
        let result = speex_sys::speex_encoder_ctl(self.encoder_handle as *mut c_void, request, ptr);
        Self::check_error(result, Some(request))
    }
}

impl<T: CoderMode> SpeexEncoder<T> {
    pub fn change_mode<M: CoderMode>(self) -> SpeexEncoder<M> {
        SpeexEncoder::<M> {
            encoder_handle: self.encoder_handle,
            mode: self.mode,
            _phantom: PhantomData,
        }
    }

    pub fn set_enhancement(&mut self, state: bool) {
        let state = state as i32;
        let ptr = &state as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_ENH, ptr).unwrap();
        }
    }

    pub fn get_enhancement(&mut self) -> bool {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_ENH, ptr).unwrap();
        }
        state != 0
    }
}

impl SpeexEncoder<NbMode> {
    pub fn new() -> SpeexEncoder<NbMode> {
        let mode = ModeId::NarrowBand.get_mode();
        let encoder_handle = SpeexEncoderHandle::create(&mode);
        Self {
            encoder_handle,
            mode,
            _phantom: PhantomData,
        }
    }
}

impl SpeexEncoder<WbMode> {
    pub fn new() -> SpeexEncoder<WbMode> {
        let mode = ModeId::WideBand.get_mode();
        let encoder_handle = SpeexEncoderHandle::create(&mode);
        Self {
            encoder_handle,
            mode,
            _phantom: PhantomData,
        }
    }
}

impl SpeexEncoder<UwbMode> {
    pub fn new() -> SpeexEncoder<UwbMode> {
        let mode = ModeId::UltraWideBand.get_mode();
        let encoder_handle = SpeexEncoderHandle::create(&mode);
        Self {
            encoder_handle,
            mode,
            _phantom: PhantomData,
        }
    }
}

impl<T: CoderMode> Drop for SpeexEncoder<T> {
    fn drop(&mut self) {
        SpeexEncoderHandle::destroy(self.encoder_handle);
    }
}
