////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2023.                                                         /
// This Source Code Form is subject to the terms of the Mozilla Public License,/
// v. 2.0. If a copy of the MPL was not distributed with this file, You can    /
// obtain one at http://mozilla.org/MPL/2.0/.                                  /
////////////////////////////////////////////////////////////////////////////////

pub(crate) mod decoder;
pub(crate) mod encoder;

use std::{error::Error, ffi::c_void, fmt::Display};

pub use decoder::SpeexDecoder;
pub use encoder::SpeexEncoder;
use speex_sys::{SpeexMode, SPEEX_MODEID_NB, SPEEX_MODEID_UWB, SPEEX_MODEID_WB};

/// Possible modes for the encoder and decoder.
#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ModeId {
    NarrowBand = SPEEX_MODEID_NB,
    WideBand = SPEEX_MODEID_WB,
    UltraWideBand = SPEEX_MODEID_UWB,
}

/// Possible submodes for the narrowband mode.
///
/// As wideband and ultra-wideband modes both embed narrowband, this is also
/// used for those.
#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum NbSubmodeId {
    /// 2150 bps "vocoder-like" mode for comfort noise
    VocoderLike = 1,
    /// 3.95 kbps very low bit-rate mode
    ExtremeLow = 8,
    /// 5.95 kbps very low bit-rate mode
    VeryLow = 2,
    /// 8 kbps low bit-rate mode
    Low = 3,
    /// 11 kbps medium bit-rate mode
    Medium = 4,
    /// 15 kbps high bit-rate mode
    High = 5,
    /// 18.2 kbps very high bit-rate mode
    VeryHigh = 6,
    /// 24.6 kbps very high bit-rate mode
    ExtremeHigh = 7,
}

impl From<i32> for NbSubmodeId {
    fn from(value: i32) -> Self {
        match value {
            1 => NbSubmodeId::VocoderLike,
            2 => NbSubmodeId::VeryLow,
            3 => NbSubmodeId::Low,
            4 => NbSubmodeId::Medium,
            5 => NbSubmodeId::High,
            6 => NbSubmodeId::VeryHigh,
            7 => NbSubmodeId::ExtremeHigh,
            8 => NbSubmodeId::ExtremeLow,
            _ => panic!("Invalid submode id"),
        }
    }
}

/// Possible submodes for the Wideband mode.
#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WbSubmodeId {
    /// disables innovation quantization entirely
    NoQuantize = 1,
    /// enables innovation quantization, but with a lower rate than the default
    QuantizedLow = 2,
    /// enables innovation quantization with the default rate
    QuantizedMedium = 3,
    /// enables innovation quantization, but with a higher rate than the default
    QuantizedHigh = 4,
}

impl From<i32> for WbSubmodeId {
    fn from(value: i32) -> Self {
        match value {
            1 => WbSubmodeId::NoQuantize,
            2 => WbSubmodeId::QuantizedLow,
            3 => WbSubmodeId::QuantizedMedium,
            4 => WbSubmodeId::QuantizedHigh,
            _ => panic!("Invalid submode id"),
        }
    }
}

/// Possible submodes for the UWB mode.
///
/// While this is an enum, UWB mode only has one submode, so it's effectively a
/// constant.
#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum UwbSubmodeId {
    Only = WbSubmodeId::NoQuantize as i32,
}

impl From<i32> for UwbSubmodeId {
    fn from(value: i32) -> Self {
        match value {
            1 => UwbSubmodeId::Only,
            _ => panic!("Invalid submode id"),
        }
    }
}

impl ModeId {
    pub fn get_mode(self) -> &'static SpeexMode {
        unsafe {
            let ptr = speex_sys::speex_lib_get_mode(self as i32);
            // speexmodes are hard constants defined within the codebase itself, so the
            // backing memory *should* always be valid. Should.
            let reference: &'static SpeexMode = &*ptr;
            reference
        }
    }

    pub fn get_frame_size(self) -> i32 {
        unsafe {
            let ptr = speex_sys::speex_lib_get_mode(self as i32);
            let mut frame_size = 0;
            let frame_size_ptr = &mut frame_size as *mut i32;
            speex_sys::speex_mode_query(
                ptr,
                speex_sys::SPEEX_MODE_FRAME_SIZE,
                frame_size_ptr as *mut c_void,
            );
            frame_size
        }
    }
}

/// Error type for the control functions of the encoder and decoder.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ControlError {
    /// The request type passed to the control function was invalid
    /// The parameter is the request type that was passed
    UnknownRequest(i32),
    /// The parameter passed to the control function was invalid (and probably
    /// caused a segfault, making this error unreachable)
    InvalidParameter,
}

impl Display for ControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlError::UnknownRequest(id) => {
                write!(
                    f,
                    "Unknown request type passed to a control function ({id})"
                )
            }
            ControlError::InvalidParameter => write!(f, "Invalid parameter"),
        }
    }
}

impl Error for ControlError {}

mod private {
    pub trait Sealed {}
}

/// Trait for the control functions of the encoder and decoder
///
/// This trait is implemented for both the encoder and decoder, and provides a
/// common interface for the control functions of both.
///
/// `ctl` is the only function that needs to be implemented, and is used to call
/// the control functions of the underlying speex library.
///
/// This trait is sealed, and cannot be implemented outside of this crate.
pub trait ControlFunctions: private::Sealed {
    /// Internal function used to convert the error codes returned by the
    /// control function into a result type
    fn check_error(err_code: i32, param: Option<i32>) -> Result<(), ControlError> {
        match err_code {
            0 => Ok(()),
            -1 => Err(ControlError::UnknownRequest(param.unwrap())),
            -2 => Err(ControlError::InvalidParameter),
            _ => panic!("Unknown error code passed to make_error(), this is a bug"),
        }
    }

    /// Calls a control function of the underlying speex library
    ///
    /// # Safety
    ///
    /// Implementations of this function call the control functions of the
    /// underlying speex library, and as such are unsafe. The caller must
    /// ensure that the parameters passed to this function are valid.
    unsafe fn ctl(&mut self, request: i32, ptr: *mut c_void) -> Result<(), ControlError>;

    /// Gets the frame size (in samples) of the encoder/decoder
    fn get_frame_size(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_FRAME_SIZE, ptr).unwrap();
        }
        state
    }

    /// Sets whether Variable BitRate is enabled or not
    fn set_vbr(&mut self, vbr: bool) {
        let state = if vbr { 1 } else { 0 };
        let ptr = &state as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_VBR, ptr).unwrap();
        }
    }

    /// Gets whether Variable BitRate is enabled or not
    fn get_vbr(&mut self) -> bool {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_VBR, ptr).unwrap();
        }
        state != 0
    }

    /// Sets the VBR quality of the encoder/decoder
    ///
    /// The value should be between 0 and 10, with 10 being the highest quality.
    fn set_vbr_quality(&mut self, quality: f32) {
        let ptr = &quality as *const f32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_VBR_QUALITY, ptr).unwrap();
        }
    }

    /// Gets the VBR quality of the encoder/decoder
    fn get_vbr_quality(&mut self) -> f32 {
        let mut state = 0.0;
        let ptr = &mut state as *mut f32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_VBR_QUALITY, ptr).unwrap();
        }
        state
    }

    /// Sets whether Voice Activity Detection is enabled or not
    fn set_vad(&mut self, vad: bool) {
        let state = if vad { 1 } else { 0 };
        let ptr = &state as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_VAD, ptr).unwrap();
        }
    }

    /// Gets whether Voice Activity Detection is enabled or not
    fn get_vad(&mut self) -> bool {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_VAD, ptr).unwrap();
        }
        state != 0
    }

    /// Sets the Average BitRate of the encoder/decoder
    fn set_abr(&mut self, abr: i32) {
        let ptr = &abr as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_ABR, ptr).unwrap();
        }
    }

    /// Gets the Average BitRate of the encoder/decoder
    fn get_abr(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_ABR, ptr).unwrap();
        }
        state
    }

    /// Sets the overall quality of the encoder/decoder
    /// The value should be between 0 and 10, with 10 being the highest quality.
    /// Default is 8.
    fn set_quality(&mut self, quality: i32) {
        let ptr = &quality as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_QUALITY, ptr).unwrap();
        }
    }

    /// Sets the current bitrate of the encoder/decoder
    fn set_bitrate(&mut self, bitrate: i32) {
        let ptr = &bitrate as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_BITRATE, ptr).unwrap();
        }
    }

    /// Gets the current bitrate of the encoder/decoder
    fn get_bitrate(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_BITRATE, ptr).unwrap();
        }
        state
    }

    /// Sets the sampling rate used for bitrate computation
    fn set_sampling_rate(&mut self, samplingrate: i32) {
        let ptr = &samplingrate as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_SAMPLING_RATE, ptr).unwrap();
        }
    }

    /// Gets the sampling rate used for bitrate computation
    fn get_sampling_rate(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_SAMPLING_RATE, ptr).unwrap();
        }
        state
    }

    /// resets the encoder/decoder memories to zero
    fn reset_state(&mut self) {
        unsafe {
            self.ctl(speex_sys::SPEEX_RESET_STATE, std::ptr::null_mut())
                .unwrap();
        }
    }

    /// Sets whether submode encoding is done in each frame
    ///
    /// Note that false breaks the specification for the format
    fn set_submode_encoding(&mut self, submode: bool) {
        let state = if submode { 1 } else { 0 };
        let ptr = &state as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_SUBMODE_ENCODING, ptr)
                .unwrap();
        }
    }

    /// Gets whether submode encoding is enabled or not
    fn get_submode_encoding(&mut self) -> bool {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_SUBMODE_ENCODING, ptr)
                .unwrap();
        }
        state != 0
    }

    /// Gets the lookahead value currently in use by the encoder/decoder
    ///
    /// Sum the lookahead of a Speex decoder and the lookahead of a Speex
    /// encoder to get the total lookahead.
    fn get_lookahead(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_LOOKAHEAD, ptr).unwrap();
        }
        state
    }

    /// Sets tuning for Packet-Loss Concealment (expected loss rate)
    fn set_plc_tuning(&mut self, tuning: i32) {
        let ptr = &tuning as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_PLC_TUNING, ptr).unwrap();
        }
    }

    /// Gets current Packet-Loss Concealment tuning value
    fn get_plc_tuning(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_PLC_TUNING, ptr).unwrap();
        }
        state
    }

    /// Sets the max bit-rate allowed in VBR mode
    fn set_vbr_max_bitrate(&mut self, max_bitrate: i32) {
        let ptr = &max_bitrate as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_VBR_MAX_BITRATE, ptr).unwrap();
        }
    }

    /// Gets the max bit-rate allowed in VBR mode
    fn get_vbr_max_bitrate(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_VBR_MAX_BITRATE, ptr).unwrap();
        }
        state
    }

    /// Enables or disables highpass filtering of the input/output
    fn set_highpass(&mut self, highpass: bool) {
        let state = if highpass { 1 } else { 0 };
        let ptr = &state as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_HIGHPASS, ptr).unwrap();
        }
    }

    /// Gets whether highpass filtering of the input/output is enabled
    fn get_highpass(&mut self) -> bool {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_HIGHPASS, ptr).unwrap();
        }
        state != 0
    }
}

/// Marker trait used to specify the mode of the de/encoder.
pub trait CoderMode {}

/// Narrowband mode (8kHz)
///
/// This is a marker type used to specify the mode of the de/encoder.
pub enum NbMode {}
impl CoderMode for NbMode {}
/// Wideband mode (16kHz)
///
/// This is a marker type used to specify the mode of the de/encoder.
pub enum WbMode {}
impl CoderMode for WbMode {}
/// Ultra-wideband mode (32kHz)
///
/// This is a marker type used to specify the mode of the de/encoder.
pub enum UwbMode {}
impl CoderMode for UwbMode {}
