pub trait RegisterField {
    fn into_bits(self) -> u32;
    fn from_bits(value: u32) -> Self;
}

#[cfg(test)]
mod tests {
    use crate::RegisterField;

    #[derive(Debug, PartialEq, Eq)]
    pub enum State {
        OFF,
        ON,
    }
    
    impl RegisterField for State {
        fn into_bits(self) -> u32 {
            self as _
        }
    
        fn from_bits(value: u32) -> Self {
            match value {
                0u32 => Self::OFF,
                1u32 => Self::ON,
                _ => panic!()
            }
        }
    }

    #[test]
    fn it_works() {
        assert_eq!(State::OFF.into_bits(), 0u32);
        assert_eq!(State::from_bits(1u32), State::ON);
    }
}
