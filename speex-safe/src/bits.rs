////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2023.                                                         /
// This Source Code Form is subject to the terms of the Mozilla Public License,/
// v. 2.0. If a copy of the MPL was not distributed with this file, You can    /
// obtain one at http://mozilla.org/MPL/2.0/.                                  /
////////////////////////////////////////////////////////////////////////////////

use std::ffi::{c_char, c_void};
use std::mem::MaybeUninit;

use speex_sys::SpeexBits as SysBits;

/// A struct that holds bits to be read or written to
///
/// Internally packs bits.
pub struct SpeexBits<'a> {
    // Only present when the speex_bits does not own the buffer
    pub buffer_ref: Option<&'a mut [u8]>,
    backing: SysBits,
}

impl<'a> SpeexBits<'a> {
    pub(crate) fn backing_mut_ptr(&mut self) -> *mut SysBits {
        &mut self.backing as *mut SysBits
    }

    /// Creates a new SpeexBits
    pub fn new() -> Self {
        let backing = unsafe {
            // SpeexBits has several padding fields reserved
            // for future use. These are left uninitialized by the C
            // library, so we zero them out here to ensure that the
            // struct is fully initialized when we call assume_init()
            let mut uninit: MaybeUninit<SysBits> = MaybeUninit::zeroed();
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
        // if let Some(buffer_ref) = &mut self.buffer_ref {
        // buffer_ref
        // } else {
        // let ptr = self.backing.chars as *mut u8;
        // let len = 0;
        // unsafe { from_raw_parts_mut(ptr, len) }
        // }
    }

    /// Creates a new SpeexBits with an existing buffer
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

    /// Advances the read pointer by `n` bits
    pub fn advance(&mut self, n: i32) {
        let ptr = self.backing_mut_ptr();
        unsafe { speex_sys::speex_bits_advance(ptr, n) };
    }

    /// Inserts a terminator so the data can be sent as a packet while
    /// autodetecting how many frames were in the packet
    pub fn insert_terminator(&mut self) {
        unsafe {
            speex_sys::speex_bits_insert_terminator(self.backing_mut_ptr());
        }
    }

    /// Returns the number of bytes in the bitstream, including the last partial
    /// byte
    pub fn num_bytes(&mut self) -> i32 {
        unsafe { speex_sys::speex_bits_nbytes(self.backing_mut_ptr()) }
    }

    /// Appends bits to the bitstream
    pub fn pack(&mut self, data: i32, num_bits: i32) {
        unsafe {
            speex_sys::speex_bits_pack(self.backing_mut_ptr(), data, num_bits);
        }
    }

    /// Gets the value of the next bit in the stream without advancing the read
    /// pointer
    pub fn peek(&mut self) -> i32 {
        unsafe { speex_sys::speex_bits_peek(self.backing_mut_ptr()) }
    }

    /// Gets the value of the next `num_bits` in the stream without advancing
    /// the read pointer
    pub fn peek_unsigned(&mut self, num_bits: i32) -> u32 {
        unsafe { speex_sys::speex_bits_peek_unsigned(self.backing_mut_ptr(), num_bits) }
    }

    pub fn read_from(&mut self, buffer: &mut [u8]) {
        unsafe {
            let ptr = buffer.as_mut_ptr() as *mut c_char;
            speex_sys::speex_bits_read_from(self.backing_mut_ptr(), ptr, buffer.len() as i32);
        }
    }

    /// Appends bytes to the bitstream
    pub fn read_whole_bytes(&mut self, bytes: &[u8]) {
        unsafe {
            let ptr = bytes.as_ptr() as *const c_char;
            speex_sys::speex_bits_read_whole_bytes(self.backing_mut_ptr(), ptr, bytes.len() as i32);
        }
    }

    /// Returns the number of bits remaining to be read from a stream
    pub fn remaining(&mut self) -> u32 {
        unsafe { speex_sys::speex_bits_remaining(self.backing_mut_ptr()) as u32 }
    }

    /// Resets SpeexBits to the initial state, erasing all content
    pub fn reset(&mut self) {
        unsafe {
            speex_sys::speex_bits_reset(self.backing_mut_ptr());
        }
    }

    /// Resets the read pointer to the beginning, without erasing the content
    pub fn rewind(&mut self) {
        unsafe {
            speex_sys::speex_bits_rewind(self.backing_mut_ptr());
        }
    }

    /// Sets an existing SpeexBits to use data from an existing buffer
    pub fn set_bit_buffer(&mut self, buffer: &mut [u8]) {
        unsafe {
            let ptr = buffer.as_mut_ptr() as *mut c_void;
            speex_sys::speex_bits_set_bit_buffer(self.backing_mut_ptr(), ptr, buffer.len() as i32);
        }
    }

    /// Interpret the next number of bits as a signed integer, advancing the
    /// read pointer
    pub fn unpack_signed(&mut self, num_bits: i32) -> i32 {
        unsafe { speex_sys::speex_bits_unpack_signed(self.backing_mut_ptr(), num_bits) }
    }

    /// Interpret the next number of bits as an unsigned integer, advancing the
    /// read pointer
    pub fn unpacked_unsigned(&mut self, num_bits: i32) -> u32 {
        unsafe { speex_sys::speex_bits_unpack_unsigned(self.backing_mut_ptr(), num_bits) }
    }

    /// Writes the content of the bitstream to a buffer
    pub fn write(&mut self, buffer: &mut [u8]) -> u32 {
        let buf_ptr = buffer.as_mut_ptr() as *mut i8;
        let len = buffer.len() as i32;
        unsafe { speex_sys::speex_bits_write(self.backing_mut_ptr(), buf_ptr, len) as u32 }
    }

    /// Writes the content of the bitstream to a buffer, writing whole bytes
    /// only. Removes any bytes that are successfully written from the
    /// bitstream.
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

#[cfg(test)]
mod test {
    use crate::SpeexBits;

    #[test]
    fn creates_and_drops() {
        {
            let mut bits = SpeexBits::new();
            let num_bytes = bits.num_bytes();
            assert_eq!(num_bytes, 0);
        }
    }

    #[test]
    fn encodes_value() {
        let mut bits = SpeexBits::new();
        bits.pack(1, 1);
        let num_bytes = bits.num_bytes();
        assert_eq!(num_bytes, 1);
    }

    #[test]
    fn write_arbitrary_bytes() {
        let mut bits = SpeexBits::new();
        let mut buffer = [12u8; 4];
        bits.write(&mut buffer);
        bits.rewind();
        let num_bytes = bits.num_bytes();
        assert_eq!(num_bytes, 4);
    }
}
