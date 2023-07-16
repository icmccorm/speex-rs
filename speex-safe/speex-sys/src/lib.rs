////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2023.                                                         /
// This Source Code Form is subject to the terms of the Mozilla Public License,/
// v. 2.0. If a copy of the MPL was not distributed with this file, You can    /
// obtain one at http://mozilla.org/MPL/2.0/.                                  /
////////////////////////////////////////////////////////////////////////////////
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod test {
    use std::{
        ffi::{c_char, c_void, CStr},
        ptr::null,
    };

    use super::*;

    #[test]
    fn linked_correctly() {
        let mut char_ptr: *const c_char = null();
        let ptr = &mut char_ptr as *mut *const c_char;
        let ptr = ptr as *mut c_void;
        let c_str = unsafe {
            speex_lib_ctl(SPEEX_LIB_GET_VERSION_STRING as i32, ptr);
            CStr::from_ptr(char_ptr)
        };
        let version_str = format!("{c_str:?}");
        assert_eq!(version_str, "\"speex-1.2.1\"".to_string())
    }
}
