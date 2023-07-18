mod bits;
mod header;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Mode {
    Narrowband,
    Wideband,
    UltraWideband,
}

impl From<i32> for Mode {
    fn from(value: i32) -> Self {
        match value {
            0 => Mode::Narrowband,
            1 => Mode::Wideband,
            2 => Mode::UltraWideband,
            _ => panic!("Unexpected value for Mode: {}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
