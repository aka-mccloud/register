
pub use register_macro::register;

#[cfg(feature = "derive")]
extern crate register_field_derive;

pub mod field {
    pub use register_field::RegisterField;

    #[cfg(feature = "derive")]
    pub mod derive {
        pub use register_field_derive::RegisterField;
    }
}



#[cfg(test)]
mod tests {
    use super::field::RegisterField;
    use super::field::derive::RegisterField;

    #[derive(RegisterField, Debug, PartialEq)]
    enum State {
        OFF,
        ON,
    }

    #[test]
    fn it_works() {
        assert_eq!(State::ON.into_bits(), 1);
    }
}
