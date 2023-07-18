////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2023.                                                         /
// This Source Code Form is subject to the terms of the Mozilla Public License,/
// v. 2.0. If a copy of the MPL was not distributed with this file, You can    /
// obtain one at http://mozilla.org/MPL/2.0/.                                  /
////////////////////////////////////////////////////////////////////////////////

use std::ffi::c_void;
use std::marker::{PhantomData, PhantomPinned};

use speex_sys::SpeexMode;

use crate::mode::{CoderMode, ControlError, ControlFunctions, ModeId, NbMode, UwbMode, WbMode};
use crate::{dynamic_mapping, mode, shared_functions, NbSubmodeId, SpeexBits, WbSubmodeId};

/// Handle for the encoder, speex represents this as an opaque pointer so this
/// is an unconstructable type that is always intended to be behind a pointer.
#[repr(C)]
pub struct SpeexEncoderHandle {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

impl SpeexEncoderHandle {
    /// Create a new encoder handle for the given mode.
    ///
    /// # Safety
    /// This allocates, so you *must* call SpeexEncoderHandle::destroy whith the
    /// handle when are done with the handle.
    ///
    /// It is not recommended to use these methods directly, instead use the
    /// `SpeexEncoder` struct.
    pub unsafe fn create(mode: &SpeexMode) -> *mut Self {
        let ptr = unsafe {
            let mode_ptr = mode as *const SpeexMode;
            speex_sys::speex_encoder_init(mode_ptr)
        };
        ptr as *mut SpeexEncoderHandle
    }

    /// Destroy the encoder handle. This MUST be called when you are done with
    /// the encoder handle.
    ///
    /// # Safety
    /// This function must *only* be called on a handle that was created with
    /// `SpeexEncoderHandle::create`. It shouldn't be called on an already
    /// destroyed handle.
    pub unsafe fn destroy(handle: *mut SpeexEncoderHandle) {
        unsafe { speex_sys::speex_encoder_destroy(handle as *mut c_void) }
    }
}

/// A struct representing a speex encoder.
pub struct SpeexEncoder<T: CoderMode> {
    encoder_handle: *mut SpeexEncoderHandle,
    pub mode: &'static SpeexMode,
    _phantom: PhantomData<T>,
}

impl<T: CoderMode> mode::private::Sealed for SpeexEncoder<T> {}

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

    /// Sets the analysis complexity of the encoder.
    pub fn set_complexity(&mut self, complexity: i32) {
        let ptr = &complexity as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_COMPLEXITY, ptr).unwrap();
        }
    }

    /// Gets the analysis complexity of the encoder.
    pub fn get_complexity(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_COMPLEXITY, ptr).unwrap();
        }
        state
    }

    /// Encode one frame of audio into the given bits.
    pub fn encode(&mut self, input: &mut [f32], bits: &mut SpeexBits) {
        let input_ptr = input.as_mut_ptr();
        unsafe {
            speex_sys::speex_encode(
                self.encoder_handle as *mut c_void,
                input_ptr,
                bits.backing_mut_ptr(),
            );
        }
    }

    /// Encode one frame of audio into the given bits, using an integer
    /// representation.
    pub fn encode_int(&mut self, input: &mut [i16], bits: &mut SpeexBits) {
        let bits_ptr = bits.backing_mut_ptr();
        let input_ptr = input.as_mut_ptr();
        unsafe {
            speex_sys::speex_encode_int(self.encoder_handle as *mut c_void, input_ptr, bits_ptr);
        }
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
    /// Create a new wideband encoder.
    pub fn new() -> SpeexEncoder<WbMode> {
        let mode = ModeId::WideBand.get_mode();
        let encoder_handle = unsafe { SpeexEncoderHandle::create(mode) };
        Self {
            encoder_handle,
            mode,
            _phantom: PhantomData,
        }
    }

    /// Sets the submode of the narrowband part of the encoder.
    pub fn set_low_submode(&mut self, low_mode: NbSubmodeId) {
        self.set_low_submode_internal(low_mode);
    }

    /// Gets the submode of the narrowband part of the encoder.
    pub fn get_low_submode(&mut self) -> NbSubmodeId {
        self.get_low_submode_internal()
    }

    /// Sets the submode of the wideband part of the encoder.
    pub fn set_high_submode(&mut self, high_mode: WbSubmodeId) {
        self.set_high_submode_internal(high_mode);
    }

    /// Gets the submode of the wideband part of the encoder.
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
    /// Create a new ultra-wideband encoder.
    pub fn new() -> SpeexEncoder<UwbMode> {
        let mode = ModeId::UltraWideBand.get_mode();
        let encoder_handle = unsafe { SpeexEncoderHandle::create(mode) };
        Self {
            encoder_handle,
            mode,
            _phantom: PhantomData,
        }
    }

    /// Sets the submode of the narrowband part of the encoder.
    pub fn set_low_submode(&mut self, low_mode: NbSubmodeId) {
        self.set_low_submode_internal(low_mode);
    }

    /// Gets the submode of the narrowband part of the encoder.
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

/// An enumeration over the different encoder modes.
/// For usecases where the encoder mode is not known at compile time.
pub enum DynamicEncoder {
    Nb(SpeexEncoder<NbMode>),
    Wb(SpeexEncoder<WbMode>),
    Uwb(SpeexEncoder<UwbMode>),
}

impl DynamicEncoder {
    shared_functions!(DynamicEncoder);

    /// Sets the analysis complexity of the encoder.
    pub fn set_complexity(&mut self, complexity: i32) {
        dynamic_mapping!(self, DynamicEncoder, inner => inner.set_complexity(complexity))
    }

    /// Gets the analysis complexity of the encoder.
    pub fn get_complexity(&mut self) -> i32 {
        dynamic_mapping!(self, DynamicEncoder, inner => inner.get_complexity())
    }

    /// Encode one frame of audio into the given bits.
    pub fn encode(&mut self, input: &mut [f32], bits: &mut SpeexBits) {
        match self {
            DynamicEncoder::Nb(inner) => inner.encode(input, bits),
            DynamicEncoder::Wb(inner) => inner.encode(input, bits),
            DynamicEncoder::Uwb(inner) => inner.encode(input, bits),
        }
    }

    /// Encode one frame of audio into the given bits, using an integer
    /// representation.
    pub fn encode_int(&mut self, input: &mut [i16], bits: &mut SpeexBits) {
        match self {
            DynamicEncoder::Nb(inner) => inner.encode_int(input, bits),
            DynamicEncoder::Wb(inner) => inner.encode_int(input, bits),
            DynamicEncoder::Uwb(inner) => inner.encode_int(input, bits),
        }
    }

    pub fn new(mode: ModeId) -> DynamicEncoder {
        match mode {
            ModeId::NarrowBand => DynamicEncoder::Nb(SpeexEncoder::<NbMode>::new()),
            ModeId::WideBand => DynamicEncoder::Wb(SpeexEncoder::<WbMode>::new()),
            ModeId::UltraWideBand => DynamicEncoder::Uwb(SpeexEncoder::<UwbMode>::new()),
        }
    }

    pub fn into_nb(self) -> Option<SpeexEncoder<NbMode>> {
        match self {
            DynamicEncoder::Nb(nb) => Some(nb),
            _ => None,
        }
    }

    pub fn into_wb(self) -> Option<SpeexEncoder<WbMode>> {
        match self {
            DynamicEncoder::Wb(wb) => Some(wb),
            _ => None,
        }
    }

    pub fn into_uwb(self) -> Option<SpeexEncoder<UwbMode>> {
        match self {
            DynamicEncoder::Uwb(uwb) => Some(uwb),
            _ => None,
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    macro_rules! set_get_test {
        ($name:ident, $set:ident, $get:ident, $value:expr) => {
            #[test]
            fn $name() {
                let mut encoder = SpeexEncoder::<WbMode>::new();
                encoder.$set($value);
                let result = encoder.$get();

                assert_eq!(result, $value);
            }
        };
    }

    set_get_test!(
        set_get_high_submode,
        set_high_submode,
        get_high_submode,
        WbSubmodeId::NoQuantize
    );

    set_get_test!(set_get_vbr, set_vbr, get_vbr, true);

    set_get_test!(set_get_vbr_quality, set_vbr_quality, get_vbr_quality, 8.0);

    set_get_test!(set_get_vad, set_vad, get_vad, true);

    set_get_test!(set_get_abr, set_abr, get_abr, 2000);

    #[test]
    fn set_quality() {
        let mut encoder = SpeexEncoder::<WbMode>::new();
        encoder.set_quality(10);
    }

    set_get_test!(set_get_bitrate, set_bitrate, get_bitrate, 3950);

    set_get_test!(
        set_get_sampling_rate,
        set_sampling_rate,
        get_sampling_rate,
        3950
    );

    #[test]
    fn get_frame_size() {
        let mut encoder = SpeexEncoder::<WbMode>::new();
        encoder.get_frame_size();
    }

    #[test]
    fn encodes_frame_without_segfault() {
        let mut encoder = SpeexEncoder::<NbMode>::new();
        let mut bits = SpeexBits::new();
        let frame_size = encoder.get_frame_size();
        let mut input = vec![23i16; frame_size as usize];

        encoder.encode_int(&mut input, &mut bits);
    }
}
