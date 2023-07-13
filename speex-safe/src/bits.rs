use speex_sys::SpeexBits as SysBits;
use std::ffi::{c_char, c_void};
use std::mem::MaybeUninit;

pub struct SpeexBits<'a> {
    // Only present when the speex_bits does not own the buffer
    buffer_ref: Option<&'a mut [u8]>,
    backing: SysBits,
}

impl<'a> SpeexBits<'a> {
    fn backing_mut_ptr(&mut self) -> *mut SysBits {
        &mut self.backing as *mut SysBits
    }

    pub fn new() -> Self {
        let backing = unsafe {
            let mut uninit: MaybeUninit<SysBits> = MaybeUninit::uninit();
            let ptr = uninit.as_mut_ptr();

            speex_sys::speex_bits_init(ptr);

            let initialized: SysBits = uninit.assume_init();
            initialized
        };

        Self {
            buffer_ref: None,
            backing,
        }
    }

    pub fn buffer<'b>(&mut self) -> &'a mut [u8] {
        todo!("")
        /*
        if let Some(buffer_ref) = &mut self.buffer_ref {
            *buffer_ref
        } else {
            let ptr = self.backing.chars as *mut u8;
            let len = 0;
            unsafe { from_raw_parts_mut(ptr, len) }
        }
         */
    }

    pub fn new_with_buffer(buffer: &'a mut [u8]) -> Self {
        let backing = unsafe {
            let mut uninit: MaybeUninit<SysBits> = MaybeUninit::uninit();
            let ptr = uninit.as_mut_ptr();

            let buffer_ptr = buffer.as_mut_ptr() as *mut std::ffi::c_void;

            speex_sys::speex_bits_init_buffer(ptr, buffer_ptr, buffer.len() as i32);

            let initialized: SysBits = uninit.assume_init();
            initialized
        };

        let buffer_ref = Some(buffer);
        Self {
            buffer_ref,
            backing,
        }
    }

    pub fn advance(&mut self, n: i32) {
        let ptr = self.backing_mut_ptr();
        unsafe { speex_sys::speex_bits_advance(ptr, n) };
    }

    pub fn insert_terminator(&mut self) {
        unsafe {
            speex_sys::speex_bits_insert_terminator(self.backing_mut_ptr());
        }
    }

    pub fn num_bytes(&mut self) -> i32 {
        unsafe { speex_sys::speex_bits_nbytes(self.backing_mut_ptr()) }
    }

    pub fn pack(&mut self, data: i32, num_bits: i32) {
        unsafe {
            speex_sys::speex_bits_pack(self.backing_mut_ptr(), data, num_bits);
        }
    }

    pub fn peek(&mut self) -> i32 {
        unsafe { speex_sys::speex_bits_peek(self.backing_mut_ptr()) }
    }

    pub fn peek_unsigned(&mut self, num_bits: i32) -> u32 {
        unsafe { speex_sys::speex_bits_peek_unsigned(self.backing_mut_ptr(), num_bits) }
    }

    pub fn read_whole_bytes(&mut self, bytes: &[u8]) {
        unsafe {
            let ptr = bytes.as_ptr() as *const c_char;
            speex_sys::speex_bits_read_whole_bytes(self.backing_mut_ptr(), ptr, bytes.len() as i32);
        }
    }

    pub fn remaining(&mut self) -> i32 {
        unsafe { speex_sys::speex_bits_remaining(self.backing_mut_ptr()) }
    }

    pub fn reset(&mut self) {
        unsafe {
            speex_sys::speex_bits_reset(self.backing_mut_ptr());
        }
    }

    pub fn rewind(&mut self) {
        unsafe {
            speex_sys::speex_bits_rewind(self.backing_mut_ptr());
        }
    }

    pub fn set_bit_buffer(&mut self, buffer: &mut [u8]) {
        unsafe {
            let ptr = buffer.as_mut_ptr() as *mut c_void;
            speex_sys::speex_bits_set_bit_buffer(self.backing_mut_ptr(), ptr, buffer.len() as i32);
        }
    }

    pub fn unpack_signed(&mut self, num_bits: i32) -> i32 {
        unsafe { speex_sys::speex_bits_unpack_signed(self.backing_mut_ptr(), num_bits) }
    }

    pub fn unpacked_unsigned(&mut self, num_bits: i32) -> u32 {
        unsafe { speex_sys::speex_bits_unpack_unsigned(self.backing_mut_ptr(), num_bits) }
    }

    pub fn write(&mut self, buffer: &mut [u8]) -> u32 {
        let buf_ptr = buffer.as_mut_ptr() as *mut i8;
        let len = buffer.len() as i32;
        unsafe { speex_sys::speex_bits_write(self.backing_mut_ptr(), buf_ptr, len) as u32 }
    }

    pub fn write_whole_bytes(&mut self, buffer: &mut [u8]) -> u32 {
        let buf_ptr = buffer.as_mut_ptr() as *mut i8;
        let len = buffer.len() as i32;
        unsafe {
            speex_sys::speex_bits_write_whole_bytes(self.backing_mut_ptr(), buf_ptr, len) as u32
        }
    }
}

impl<'a> Default for SpeexBits<'a> {
    fn default() -> Self {
        SpeexBits::new()
    }
}

impl<'a> Drop for SpeexBits<'a> {
    fn drop(&mut self) {
        let ptr = &mut self.backing as *mut speex_sys::SpeexBits;
        unsafe {
            speex_sys::speex_bits_destroy(ptr);
        }
    }
}
