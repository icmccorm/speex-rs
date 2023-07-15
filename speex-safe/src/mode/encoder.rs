use crate::mode::{CoderMode, ControlError, ControlFunctions, ModeId, NbMode, UwbMode, WbMode};
use crate::{NbSubmodeId, WbSubmodeId};
use speex_sys::SpeexMode;
use std::ffi::c_void;
use std::marker::{PhantomData, PhantomPinned};

/// Handle for the encoder, speex represents this as an opaque pointer so this is an unconstructable
/// type that is always intended to be behind a pointer.
#[repr(C)]
pub struct SpeexEncoderHandle {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

impl SpeexEncoderHandle {
    /// Create a new encoder handle for the given mode.
    ///
    /// # Safety
    /// This allocates, so you *must* call SpeexEncoderHandle::destroy whith the handle when are
    /// done with the handle.
    ///
    /// It is not recommended to use these methods directly, instead use the `SpeexEncoder` struct.
    pub unsafe fn create(mode: &SpeexMode) -> *mut Self {
        let ptr = unsafe {
            let mode_ptr = mode as *const SpeexMode;
            speex_sys::speex_encoder_init(mode_ptr)
        };
        ptr as *mut SpeexEncoderHandle
    }

    /// Destroy the encoder handle. This MUST be called when you are done with the encoder handle.
    pub fn destroy(handle: *mut SpeexEncoderHandle) {
        unsafe { speex_sys::speex_encoder_destroy(handle as *mut c_void) }
    }
}

/// The encoder struct, this is a safe wrapper around the unsafe `SpeexEncoderHandle`.
///
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

impl SpeexEncoder<NbMode> {
    /// Create a new narrowband encoder.
    pub fn new() -> SpeexEncoder<NbMode> {
        let mode = ModeId::NarrowBand.get_mode();
        let encoder_handle = unsafe { SpeexEncoderHandle::create(mode) };
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

impl Default for SpeexEncoder<NbMode> {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeexEncoder<WbMode> {
    pub fn new() -> SpeexEncoder<WbMode> {
        let mode = ModeId::WideBand.get_mode();
        let encoder_handle = unsafe { SpeexEncoderHandle::create(mode) };
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

impl Default for SpeexEncoder<WbMode> {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeexEncoder<UwbMode> {
    pub fn new() -> SpeexEncoder<UwbMode> {
        let mode = ModeId::UltraWideBand.get_mode();
        let encoder_handle = unsafe { SpeexEncoderHandle::create(mode) };
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

impl Default for SpeexEncoder<UwbMode> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: CoderMode> Drop for SpeexEncoder<T> {
    fn drop(&mut self) {
        unsafe {
            SpeexEncoderHandle::destroy(self.encoder_handle);
        }
    }
}
