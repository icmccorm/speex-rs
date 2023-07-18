////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2023.                                                         /
// This Source Code Form is subject to the terms of the Mozilla Public License,/
// v. 2.0. If a copy of the MPL was not distributed with this file, You can    /
// obtain one at http://mozilla.org/MPL/2.0/.                                  /
////////////////////////////////////////////////////////////////////////////////


use crate::Mode;

pub struct Header {
    pub sample_rate: u32,
    pub mode: Mode,
    pub variable_bit_rate: bool,
}

impl Header {
    pub const MAGIC: &'static [u8; 8] = b"Speex   ";

    pub fn get_version_string() -> &'static str {
        "1.2rc2"
    }
}
