////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2023.                                                         /
// This Source Code Form is subject to the terms of the Mozilla Public License,/
// v. 2.0. If a copy of the MPL was not distributed with this file, You can    /
// obtain one at http://mozilla.org/MPL/2.0/.                                  /
////////////////////////////////////////////////////////////////////////////////

use crate::mode::{CoderMode, ControlFunctions, ModeId};
use crate::{mode, ControlError, NbMode, NbSubmodeId, SpeexBits, UwbMode, WbMode, WbSubmodeId};
use speex_sys::SpeexMode;
use std::ffi::{c_float, c_void};
use std::fmt::{Display, Formatter};
use std::marker::{PhantomData, PhantomPinned};

/// Handle for the encoder, speex represents this as an opaque pointer so this is an unconstructable
/// type that is always intended to be behind a pointer.
#[repr(C)]
pub struct SpeexDecoderHandle {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

impl SpeexDecoderHandle {
    /// Create a new decoder handle for the given mode.
    ///
    /// # Safety
    ///
    /// This allocates, so you *must* call SpeexDecoderHandle::destroy with the handle when created
    /// once you are done with the handle.
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

/// A struct representing a speex decoder.
pub struct SpeexDecoder<T: CoderMode> {
    encoder_handle: *mut SpeexDecoderHandle,
    pub mode: &'static SpeexMode,
    _phantom: PhantomData<T>,
}

impl<T: CoderMode> mode::private::Sealed for SpeexDecoder<T> {}

impl<T: CoderMode> ControlFunctions for SpeexDecoder<T> {
    unsafe fn ctl(&mut self, request: i32, ptr: *mut c_void) -> Result<(), ControlError> {
        let result = speex_sys::speex_decoder_ctl(self.encoder_handle as *mut c_void, request, ptr);
        Self::check_error(result, Some(request))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum DecoderError {
    TooSmallBuffer,
    EndOfStream,
    CorruptStream,
}

impl Display for DecoderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DecoderError::TooSmallBuffer => write!(f, "Buffer is too small to decode into"),
            DecoderError::EndOfStream => write!(f, "End of stream reached while decoding"),
            DecoderError::CorruptStream => write!(f, "Corrupt stream was unable to be decoded"),
        }
    }
}

impl<T: CoderMode> SpeexDecoder<T> {
    /// Set whether to use enhancement.
    pub fn set_enhancement(&mut self, state: bool) {
        let state = state as i32;
        let ptr = &state as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_ENH, ptr).unwrap();
        }
    }

    /// Get whether enhancement is turned on or not.
    pub fn get_enhancement(&mut self) -> bool {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_ENH, ptr).unwrap();
        }
        state != 0
    }

    /// Decode one frame of speex data from the bitstream
    pub fn decode(&mut self, bits: &mut SpeexBits, out: &mut [f32]) -> Result<(), DecoderError> {
        let frame_size = self.get_frame_size() as usize;
        if out.len() < frame_size {
            return Err(DecoderError::TooSmallBuffer);
        }
        let out_ptr = out.as_mut_ptr();
        let bits_ptr = bits.backing_mut_ptr();
        let result = unsafe {
            speex_sys::speex_decode(self.encoder_handle as *mut c_void, bits_ptr, out_ptr)
        };
        match result {
            0 => Ok(()),
            -1 => Err(DecoderError::EndOfStream),
            -2 => Err(DecoderError::CorruptStream),
            _ => panic!("Unexpected return value from speex_decode"),
        }
    }

    /// Decode one frame of speex data from the bitstream into a new Vec<f32>
    pub fn decode_to_owned(&mut self, bits: &mut SpeexBits) -> Result<Vec<f32>, DecoderError> {
        let frame_size = self.get_frame_size() as usize;
        let mut out = vec![0.0; frame_size];
        self.decode(bits, &mut out)?;
        Ok(out)
    }

    /// Decode one frame of speex data from the bitstream, as i16
    pub fn decode_int(
        &mut self,
        bits: &mut SpeexBits,
        out: &mut [i16],
    ) -> Result<(), DecoderError> {
        let frame_size = self.get_frame_size() as usize;
        if out.len() < frame_size {
            return Err(DecoderError::TooSmallBuffer);
        }
        let out_ptr = out.as_mut_ptr();
        let bits_ptr = bits.backing_mut_ptr();
        let result = unsafe {
            speex_sys::speex_decode_int(self.encoder_handle as *mut c_void, bits_ptr, out_ptr)
        };
        match result {
            0 => Ok(()),
            -1 => Err(DecoderError::EndOfStream),
            -2 => Err(DecoderError::CorruptStream),
            _ => panic!("Unexpected return value from speex_decode"),
        }
    }

    /// Decode one frame of speex data from the bitstream into a new Vec<i16>
    pub fn decode_int_to_owned(&mut self, bits: &mut SpeexBits) -> Result<Vec<i16>, DecoderError> {
        let frame_size = self.get_frame_size() as usize;
        let mut out = vec![0; frame_size];
        self.decode_int(bits, &mut out)?;
        Ok(out)
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
    /// Create a new WideBand encoder.
    pub fn new() -> SpeexDecoder<WbMode> {
        let mode = ModeId::WideBand.get_mode();
        let encoder_handle = unsafe { SpeexDecoderHandle::create(mode) };
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

impl Default for SpeexDecoder<WbMode> {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeexDecoder<UwbMode> {
    /// Create a new Ultra WideBand encoder.
    pub fn new() -> SpeexDecoder<UwbMode> {
        let mode = ModeId::UltraWideBand.get_mode();
        let encoder_handle = unsafe { SpeexDecoderHandle::create(mode) };
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
