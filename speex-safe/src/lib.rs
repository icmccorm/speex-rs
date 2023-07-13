pub(crate) mod bits;
pub(crate) mod header;
pub(crate) mod mode;
pub(crate) mod stereo_state;

pub use bits::SpeexBits;
pub use header::SpeexHeader;
pub use mode::SpeexMode;
use speex_sys::{
    speex_lib_ctl, SPEEX_LIB_GET_EXTRA_VERSION, SPEEX_LIB_GET_MAJOR_VERSION,
    SPEEX_LIB_GET_MICRO_VERSION, SPEEX_LIB_GET_MINOR_VERSION, SPEEX_LIB_GET_VERSION_STRING,
};
use std::ffi::{c_char, c_int, c_void, CStr};
use std::ptr::null;
pub use stereo_state::SpeexStereoState;

pub fn get_major_version() -> i32 {
    let mut major_version = 0;
    unsafe {
        let ptr = &mut major_version as *mut i32;
        let ptr = ptr as *mut c_void;
        speex_lib_ctl(SPEEX_LIB_GET_MAJOR_VERSION as c_int, ptr);
    }
    major_version
}

pub fn get_minor_version() -> i32 {
    let mut minor_version = 0;
    unsafe {
        let ptr = &mut minor_version as *mut i32;
        let ptr = ptr as *mut c_void;
        speex_lib_ctl(SPEEX_LIB_GET_MINOR_VERSION as c_int, ptr);
    }
    minor_version
}

pub fn get_micro_version() -> i32 {
    let mut micro_version = 0;
    unsafe {
        let ptr = &mut micro_version as *mut i32;
        let ptr = ptr as *mut c_void;
        speex_lib_ctl(SPEEX_LIB_GET_MICRO_VERSION as c_int, ptr);
    }
    micro_version
}

pub fn get_extra_version() -> String {
    let cstr = unsafe {
        let mut str = null();
        let str_ptr = &mut str as *mut *const c_char;
        speex_lib_ctl(SPEEX_LIB_GET_EXTRA_VERSION as c_int, str_ptr as *mut c_void);
        CStr::from_ptr(str)
    };
    cstr.to_string_lossy().into_owned()
}

pub fn get_version_string() -> String {
    let cstr = unsafe {
        let mut str = null();
        let str_ptr = &mut str as *mut *const c_char;
        speex_lib_ctl(
            SPEEX_LIB_GET_VERSION_STRING as c_int,
            str_ptr as *mut c_void,
        );
        CStr::from_ptr(str)
    };
    cstr.to_string_lossy().into_owned()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn correct_version_outputs() {
        let version_string = get_version_string();
        assert_eq!(&version_string, "speex-1.2.1")
    }
}
