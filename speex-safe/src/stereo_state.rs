use speex_sys::SpeexStereoState as SysStereoState;

pub struct SpeexStereoState {
    backing: SysStereoState,
}

impl SpeexStereoState {
    pub fn new() -> Self {
        let backing = unsafe {
            let ptr = speex_sys::speex_stereo_state_init();
            *ptr
        };

        Self { backing }
    }

    pub fn reset(&mut self) {
        let ptr = &mut self.backing as *mut SysStereoState;
        unsafe { speex_sys::speex_stereo_state_reset(ptr) }
    }
}

impl Default for SpeexStereoState {
    fn default() -> Self {
        Self::new()
    }
}
