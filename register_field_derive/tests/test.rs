use register_field_derive::RegisterField;

#[derive(Debug, RegisterField, PartialEq, Eq)]
pub enum State {
    OFF = 0b0000,
    ON = 0b0001,
}

mod test {
    use register_field::RegisterField;

    use crate::State;
    
    #[test]
    fn it_works() {
        assert_eq!(State::OFF.into_bits(), 0u32);
        assert_eq!(State::from_bits(1u32), State::ON);
    }
}
