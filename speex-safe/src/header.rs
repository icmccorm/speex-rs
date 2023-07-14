use speex_sys::{SpeexHeader as SysHeader, SpeexMode};
use std::mem::MaybeUninit;

/// ## Why doesn't this implement `Drop`?
///
/// You may notice in `speex_sys` there is a `free` function for headers.
/// The data within `SpeexHeader` is actually entirely stack allocated. There is nothing to be
/// freed. The `free` is for the arrays/pointers allocated by `packet_to_header` and `header_to_packet`.
/// For `packet_to_header` instead of using a manual call to free, it is wrapped in a `Vec` which can
/// manage the memory just fine.
pub struct SpeexHeader {
    backing: SysHeader,
}

impl SpeexHeader {
    pub fn new(rate: i32, num_channels: i32, mode: &SpeexMode) -> Self {
        let backing = unsafe {
            let mut uninit: MaybeUninit<SysHeader> = MaybeUninit::uninit();
            let ptr = uninit.as_mut_ptr();

            let mode_ptr = mode as *const SpeexMode;
            speex_sys::speex_init_header(ptr, rate, num_channels, mode_ptr);

            let initialized: SysHeader = uninit.assume_init();
            initialized
        };
        Self { backing }
    }

    pub fn from_packet(packet: &mut [u8]) -> Self {
        let backing = unsafe {
            let ptr = packet.as_mut_ptr() as *mut i8;
            let length = packet.len() as i32;
            let header_ptr = speex_sys::speex_packet_to_header(ptr, length);
            let derefed = *header_ptr;
            speex_sys::speex_header_free(header_ptr as *mut std::ffi::c_void);
            derefed
        };
        Self { backing }
    }

    pub fn make_packet(&mut self) -> Vec<u8> {
        let ptr = &mut self.backing as *mut SysHeader;
        let mut size: i32 = 0;
        let size_ptr = &mut size as *mut i32;
        unsafe {
            let buff_ptr = speex_sys::speex_header_to_packet(ptr, size_ptr) as *mut u8;
            Vec::from_raw_parts(buff_ptr, size as usize, size as usize)
        }
    }
}
