pub(crate) mod decoder;
pub(crate) mod encoder;

pub use decoder::SpeexDecoder;
pub use encoder::SpeexEncoder;

use speex_sys::{SpeexMode, SPEEX_MODEID_NB, SPEEX_MODEID_UWB, SPEEX_MODEID_WB};
use std::error::Error;
use std::ffi::c_void;
use std::fmt::Display;

#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ModeId {
    NarrowBand = SPEEX_MODEID_NB,
    WideBand = SPEEX_MODEID_WB,
    UltraWideBand = SPEEX_MODEID_UWB,
}

#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum NbSubmodeId {
    VocoderLike = 1,
    ExtremeLow = 8,
    VeryLow = 2,
    Low = 3,
    Medium = 4,
    High = 5,
    VeryHigh = 6,
    ExtremeHigh = 7,
}

#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WbSubmodeId {
    NoQuantize = 1,
    v2 = 2,
    v3 = 3,
    v4 = 4,
}

/// Possible submodes for the UWB mode.
///
/// While this is an enum, UWB mode only has one submode, so it's effectively a constant.
#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum UwbSubmodeId {
    Only = WbSubmodeId::NoQuantize as i32,
}

impl ModeId {
    pub fn get_mode(self) -> &'static SpeexMode {
        unsafe {
            let ptr = speex_sys::speex_lib_get_mode(self as i32);
            // speexmodes are hard constants defined within the codebase itself, so the backing
            // memory *should* always be valid. Should.
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ControlError {
    UnknownRequest(i32),
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

/// Trait for the control functions of the encoder and decoder
///
/// This trait is implemented for both the encoder and decoder, and provides a common interface
/// for the control functions of both.
///
/// `ctl` is the only function that needs to be implemented, and is used to call the control
/// functions of the underlying speex library.
trait ControlFunctions {
    /// Internal function used to convert the error codes returned by the control function into
    /// a result type
    fn check_error(err_code: i32, param: Option<i32>) -> Result<(), ControlError> {
        match err_code {
            0 => Ok(()),
            -1 => Err(ControlError::UnknownRequest(param.unwrap())),
            -2 => Err(ControlError::InvalidParameter),
            _ => panic!("Unknown error code passed to make_error(), this is a bug"),
        }
    }

    /// Calls a control function of the underlying speex library
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

    fn set_vbr(&mut self, vbr: bool) {
        let state = if vbr { 1 } else { 0 };
        let ptr = &state as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_VBR, ptr).unwrap();
        }
    }

    fn get_vbr(&mut self) -> bool {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_VBR, ptr).unwrap();
        }
        state != 0
    }

    fn set_vbr_quality(&mut self, quality: f32) {
        let ptr = &quality as *const f32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_VBR_QUALITY, ptr).unwrap();
        }
    }

    fn get_vbr_quality(&mut self) -> f32 {
        let mut state = 0.0;
        let ptr = &mut state as *mut f32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_VBR_QUALITY, ptr).unwrap();
        }
        state
    }

    fn set_vad(&mut self, vad: bool) {
        let state = if vad { 1 } else { 0 };
        let ptr = &state as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_VAD, ptr).unwrap();
        }
    }

    fn get_vad(&mut self) -> bool {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_VAD, ptr).unwrap();
        }
        state != 0
    }

    fn set_abr(&mut self, abr: bool) {
        let state = if abr { 1 } else { 0 };
        let ptr = &state as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_ABR, ptr).unwrap();
        }
    }

    fn get_abr(&mut self) -> bool {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_ABR, ptr).unwrap();
        }
        state != 0
    }

    fn set_quality(&mut self, quality: i32) {
        let ptr = &quality as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_QUALITY, ptr).unwrap();
        }
    }

    fn set_complexity(&mut self, complexity: i32) {
        let ptr = &complexity as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_COMPLEXITY, ptr).unwrap();
        }
    }

    fn get_complexity(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_COMPLEXITY, ptr).unwrap();
        }
        state
    }

    fn set_bitrate(&mut self, bitrate: i32) {
        let ptr = &bitrate as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_BITRATE, ptr).unwrap();
        }
    }

    fn get_bitrate(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_BITRATE, ptr).unwrap();
        }
        state
    }

    fn set_samplingrate(&mut self, samplingrate: i32) {
        let ptr = &samplingrate as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_SAMPLING_RATE, ptr).unwrap();
        }
    }

    fn get_samplingrate(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_SAMPLING_RATE, ptr).unwrap();
        }
        state
    }

    fn reset_state(&mut self) {
        unsafe {
            self.ctl(speex_sys::SPEEX_RESET_STATE, std::ptr::null_mut())
                .unwrap();
        }
    }

    fn set_submode_encoding(&mut self, submode: i32) {
        let ptr = &submode as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_SUBMODE_ENCODING, ptr)
                .unwrap();
        }
    }

    fn get_submode_encoding(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_SUBMODE_ENCODING, ptr)
                .unwrap();
        }
        state
    }

    fn get_lookahead(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_LOOKAHEAD, ptr).unwrap();
        }
        state
    }

    fn set_plc_tuning(&mut self, tuning: i32) {
        let ptr = &tuning as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_PLC_TUNING, ptr).unwrap();
        }
    }

    fn get_plc_tuning(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_PLC_TUNING, ptr).unwrap();
        }
        state
    }

    fn set_vbr_max_bitrate(&mut self, max_bitrate: i32) {
        let ptr = &max_bitrate as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_VBR_MAX_BITRATE, ptr).unwrap();
        }
    }

    fn get_vbr_max_bitrate(&mut self) -> i32 {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_VBR_MAX_BITRATE, ptr).unwrap();
        }
        state
    }

    fn set_highpass(&mut self, highpass: bool) {
        let state = if highpass { 1 } else { 0 };
        let ptr = &state as *const i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_SET_HIGHPASS, ptr).unwrap();
        }
    }

    fn get_highpass(&mut self) -> bool {
        let mut state = 0;
        let ptr = &mut state as *mut i32 as *mut c_void;
        unsafe {
            self.ctl(speex_sys::SPEEX_GET_HIGHPASS, ptr).unwrap();
        }
        state != 0
    }
}

pub trait CoderMode {}

pub enum NbMode {}
impl CoderMode for NbMode {}
pub enum WbMode {}
impl CoderMode for WbMode {}
pub enum UwbMode {}
impl CoderMode for UwbMode {}
