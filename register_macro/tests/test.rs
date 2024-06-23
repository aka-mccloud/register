use register::register;

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    OFF,
    ON,
}

impl State {
    fn from_bits(val: u32) -> State {
        if val == 0 { State::OFF } else { State::ON }
    }

    fn into_bits(val: State) -> u32 {
        match val {
            State::OFF => 0,
            State::ON => 1,
        }
    }
}

#[test]
fn it_works() {
    #[register]
    pub struct ClockControlRegister {
        #[bits(1, rw, get = hsi_get_state, set = hsi_set)]
        HSION: State,

        #[bits(1, ro, get = hsi_is_ready)]
        HSIRDY: bool,

        #[bits(1)]
        __: u8,

        #[bits(
            5,
            rwc,
            get = hsi_get_trimming_value,
            set = hsi_set_trimming_value,
            clear = hsi_clear_trimming_value
        )]
        HSITRIM: u8,

        #[bits(
            8,
            ro,
            get = hsi_get_calibration_value,
            set = hsi_set_trimming_value,
            clear = hsi_clear_trimming_value
        )]
        HSICAL: u8,

        #[bits(1, rw, get = hse_get_state, set = hse_set)]
        HSEON: State,

        #[bits(1, ro, get = hse_is_ready)]
        HSERDY: bool,

        #[bits(1, rw, get = hse_bypass_get_state, set = hse_bypass_set)]
        HSEBYP: State,

        #[bits(1, rw, get = css_get_state, set = css_set)]
        CSSON: State,

        #[bits(4)]
        __: u8,

        #[bits(1, rw, get = pll_get_state, set = pll_set)]
        PLLON: State,

        #[bits(1, ro, get = pll_is_ready)]
        PLLRDY: bool,

        #[bits(1, rw, get = pll_i2s_get_state, set = pll_i2s_set)]
        PLLI2SON: State,

        #[bits(1, ro, get = pll_i2s_is_ready)]
        PLLI2SRDY: bool,

        #[bits(1, rw, get = pll_sai_get_state, set = pll_sai_set)]
        PLLSAION: State,

        #[bits(1, ro, get = pll_sai_is_ready)]
        PLLSAIRDY: bool,

        #[bits(2)]
        __: u8,
    }

    let mut r = ClockControlRegister((0b00000000_00000010_00000000_00000000).into());

    assert_eq!(r.hse_get_state(), State::OFF);
    assert_eq!(r.hse_is_ready(), true);

    r.hse_set(State::ON);
    r.hsi_set_trimming_value(11);

    assert_eq!(r.hse_get_state(), State::ON);
    // assert_eq!(r.hsi_is_ready(), false);
    // assert_eq!(r.hsi_get_trimming_value(), 11);

    // r.hsi_disable();

    // assert_eq!(r.hsi_is_enabled(), false);
}
