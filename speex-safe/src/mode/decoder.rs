use crate::mode::{CoderMode, ControlFunctions, ModeId};
use crate::{ControlError, NbMode, NbSubmodeId, UwbMode, WbMode, WbSubmodeId};
use speex_sys::SpeexMode;
use std::ffi::c_void;
use std::marker::{PhantomData, PhantomPinned};

#[repr(C)]
pub struct SpeexDecoderHandle {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

impl SpeexDecoderHandle {
    pub unsafe fn create(mode: &SpeexMode) -> *mut Self {
        let ptr = unsafe {
            let mode_ptr = mode as *const SpeexMode;
            speex_sys::speex_decoder_init(mode_ptr)
        };
        ptr as *mut SpeexDecoderHandle
    }

    /// Destroys a SpeexDecoderHandle
    ///
    /// # Safety
    ///
    /// This function must *only* be called on a handle that was created with `SpeexDecoderHandle::create`.
    /// It shouldn't be called on an already destroyed handle.
    pub unsafe fn destroy(handle: *mut SpeexDecoderHandle) {
        unsafe {
            speex_sys::speex_decoder_destroy(handle as *mut c_void);
        }
    }
}

pub struct SpeexDecoder<T: CoderMode> {
    encoder_handle: *mut SpeexDecoderHandle,
    mode: &'static SpeexMode,
    _phantom: PhantomData<T>,
}

impl<T: CoderMode> ControlFunctions for SpeexDecoder<T> {
    unsafe fn ctl(&mut self, request: i32, ptr: *mut c_void) -> Result<(), ControlError> {
        let result = speex_sys::speex_decoder_ctl(self.encoder_handle as *mut c_void, request, ptr);
        Self::check_error(result, Some(request))
    }
}

impl<T: CoderMode> SpeexDecoder<T> {
    /// Set whether to use
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

    fn get_low_submode_internal(&mut self) -> NbSubmodeId {
        let mut low_mode = 0;
        let ptr = &mut low_mode as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_LOW_MODE, ptr).unwrap();
        }
        low_mode.into()
    }

    fn set_low_submode_internal(&mut self, low_mode: NbSubmodeId) {
        let low_mode = low_mode as i32;
        let ptr = &low_mode as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_LOW_MODE, ptr).unwrap();
        }
    }

    fn set_high_submode_internal(&mut self, high_mode: WbSubmodeId) {
        let high_mode = high_mode as i32;
        let ptr = &high_mode as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_HIGH_MODE, ptr).unwrap();
        }
    }

    fn get_high_submode_internal(&mut self) -> WbSubmodeId {
        let mut high_mode = 0;
        let ptr = &mut high_mode as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_HIGH_MODE, ptr).unwrap();
        }
        high_mode.into()
    }
}

impl SpeexDecoder<NbMode> {
    /// Create a new narrowband encoder.
    pub fn new() -> SpeexDecoder<NbMode> {
        let mode = ModeId::NarrowBand.get_mode();
        let encoder_handle = unsafe { SpeexDecoderHandle::create(mode) };
        Self {
            encoder_handle,
            mode,
            _phantom: PhantomData,
        }
    }

    /// Sets the submode to use for encoding.
    pub fn set_submode(&mut self, submode: NbSubmodeId) {
        self.set_low_submode_internal(submode);
    }

    /// Gets the submode currently in use for encoding.
    pub fn get_submode(&mut self) -> NbSubmodeId {
        self.get_low_submode_internal()
    }
}

impl Default for SpeexDecoder<NbMode> {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeexDecoder<WbMode> {
    pub fn new() -> SpeexDecoder<WbMode> {
        let mode = ModeId::WideBand.get_mode();
        let encoder_handle = unsafe { SpeexDecoderHandle::create(mode) };
        Self {
            encoder_handle,
            mode,
            _phantom: PhantomData,
        }
    }

    pub fn set_low_submode(&mut self, low_mode: NbSubmodeId) {
        self.set_low_submode_internal(low_mode);
    }

    pub fn get_low_submode(&mut self) -> NbSubmodeId {
        self.get_low_submode_internal()
    }

    pub fn set_high_submode(&mut self, high_mode: WbSubmodeId) {
        self.set_high_submode_internal(high_mode);
    }

    pub fn get_high_submode(&mut self) -> WbSubmodeId {
        self.get_high_submode_internal()
    }
}

impl Default for SpeexDecoder<WbMode> {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeexDecoder<UwbMode> {
    pub fn new() -> SpeexDecoder<UwbMode> {
        let mode = ModeId::UltraWideBand.get_mode();
        let encoder_handle = unsafe { SpeexDecoderHandle::create(mode) };
        Self {
            encoder_handle,
            mode,
            _phantom: PhantomData,
        }
    }

    pub fn set_low_submode(&mut self, low_mode: NbSubmodeId) {
        self.set_low_submode_internal(low_mode);
    }

    pub fn get_low_submode(&mut self) -> NbSubmodeId {
        self.get_low_submode_internal()
    }
}

impl Default for SpeexDecoder<UwbMode> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: CoderMode> Drop for SpeexDecoder<T> {
    fn drop(&mut self) {
        unsafe { SpeexDecoderHandle::destroy(self.encoder_handle) }
    }
}
