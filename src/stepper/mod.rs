use rp235x_hal::gpio::{FunctionSio, Pin, PinId, PullDown, SioOutput};

use embedded_hal::digital::{OutputPin, PinState};

/// Direction the motor turns in. Just reverses the order of the internal states.
#[derive(PartialEq, Eq)]
pub enum Direction {
    /// Default direction
    Normal,
    /// Reversed direction
    Reverse,
}

/// different positions of the motor.
/// Depending on the state different pins have to be high
/// |wire | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
/// | --- | - | - | - | - | - | - | - | - | - |
/// |  1  |   |   |   |   |   |   | x | x | x |
/// |  2  |   |   |   |   | x | x | x |   |   |
/// |  3  |   |   | x | x | x |   |   |   |   |
/// |  4  |   | x | x |   |   |   |   |   | x |
#[derive(Clone, Copy, Debug)]
enum State {
    State0,
    State1,
    State2,
    State3,
    State4,
    State5,
    State6,
    State7,
    State8,
}

fn get_pin_states(s: State) -> [PinState; 4] {
    match s {
        State::State0 => [PinState::Low, PinState::Low, PinState::Low, PinState::Low],
        State::State1 => [PinState::Low, PinState::Low, PinState::Low, PinState::High],
        State::State2 => [PinState::Low, PinState::Low, PinState::High, PinState::High],
        State::State3 => [PinState::Low, PinState::Low, PinState::High, PinState::Low],
        State::State4 => [PinState::Low, PinState::High, PinState::High, PinState::Low],
        State::State5 => [PinState::Low, PinState::High, PinState::Low, PinState::Low],
        State::State6 => [PinState::High, PinState::High, PinState::Low, PinState::Low],
        State::State7 => [PinState::High, PinState::Low, PinState::Low, PinState::Low],
        State::State8 => [PinState::High, PinState::Low, PinState::Low, PinState::High],
    }
}

fn get_next_state(s: State) -> State {
    match s {
        State::State0 => State::State1,
        State::State1 => State::State2,
        State::State2 => State::State3,
        State::State3 => State::State4,
        State::State4 => State::State5,
        State::State5 => State::State6,
        State::State6 => State::State7,
        State::State7 => State::State8,
        State::State8 => State::State1,
    }
}

fn get_prev_state(s: State) -> State {
    match s {
        State::State0 => State::State8,
        State::State1 => State::State8,
        State::State2 => State::State1,
        State::State3 => State::State2,
        State::State4 => State::State3,
        State::State5 => State::State4,
        State::State6 => State::State5,
        State::State7 => State::State6,
        State::State8 => State::State7,
    }
}

/// gets returned if en Error happens while stepping
#[derive(Debug)]
pub struct StepError;

type P<I> = Pin<I, FunctionSio<SioOutput>, PullDown>;

/// Struct representing a Stepper motor with the 4 driver pins
pub struct ULN2003<I1: PinId, I2: PinId, I3: PinId, I4: PinId> {
    gpio2: P<I1>,
    gpio3: P<I2>,
    gpio4: P<I3>,
    gpio5: P<I4>,
    state: State,
    dir: Direction,
}

impl<I1: PinId, I2: PinId, I3: PinId, I4: PinId> ULN2003<I1, I2, I3, I4> {
    /// Create a new StepperMotor from the 4 pins connected to te uln2003 driver.
    /// The delay parameter is needed if you want to use the step_for function.
    pub fn new(gpio2: P<I1>, gpio3: P<I2>, gpio4: P<I3>, gpio5: P<I4>) -> Self {
        Self {
            gpio2,
            gpio3,
            gpio4,
            gpio5,
            state: State::State0,
            dir: Direction::Normal,
        }
    }

    fn apply_state(&mut self) -> Result<(), StepError> {
        let states = get_pin_states(self.state);
        set_state(&mut self.gpio2, states[0])?;
        set_state(&mut self.gpio3, states[1])?;
        set_state(&mut self.gpio4, states[2])?;
        set_state(&mut self.gpio5, states[3])?;
        Ok(())
    }
}

pub fn set_state<P: PinId>(
    pin: &mut Pin<P, FunctionSio<SioOutput>, PullDown>,
    state: PinState,
) -> Result<(), StepError> {
    match pin.set_state(state) {
        Ok(_) => Ok(()),
        Err(_) => Err(StepError),
    }
}

impl<I1: PinId, I2: PinId, I3: PinId, I4: PinId> StepperMotor for ULN2003<I1, I2, I3, I4> {
    fn step(&mut self) -> Result<(), StepError> {
        match self.dir {
            Direction::Normal => self.state = get_next_state(self.state),
            Direction::Reverse => self.state = get_prev_state(self.state),
        }
        self.apply_state()?;
        Ok(())
    }

    fn set_direction(&mut self, dir: Direction) {
        self.dir = dir;
    }

    fn stop(&mut self) -> Result<(), StepError> {
        set_state(&mut self.gpio2, PinState::Low)?;
        set_state(&mut self.gpio3, PinState::Low)?;
        set_state(&mut self.gpio4, PinState::Low)?;
        set_state(&mut self.gpio5, PinState::Low)?;
        Ok(())
    }
}

/// trait to prevent having to pass around the struct with all the generic arguments
pub trait StepperMotor {
    /// Do a single step
    fn step(&mut self) -> Result<(), StepError>;
    /// Set the stepping direction
    fn set_direction(&mut self, dir: Direction);
    /// Stoping sets all pins low
    fn stop(&mut self) -> Result<(), StepError>;
}
