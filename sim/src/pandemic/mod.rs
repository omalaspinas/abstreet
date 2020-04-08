mod pandemic;
mod prob;

use geom::Duration;
pub use pandemic::{Cmd, PandemicModel};
pub use prob::{erf_distrib_bounded, sigmoid_distrib};
use rand::Rng;
use rand_xorshift::XorShiftRng;
use geom::Time;

#[derive(PartialEq, Clone, Copy)]
pub enum SEIR {
    Sane,
    Exposed,
    Infectious,
    // Hospitalized,
    Recovered,
}

pub enum SeirEvent {
    Exposition(f64),
    Incubation(f64),
    // Hospitalization(f64),
    Recovery(f64),
}

fn is_chosen_or(new_state: SEIR, old_state: SEIR, rng: &mut XorShiftRng, prob: f64) -> SEIR {
    if rng.gen_bool(prob) {
        new_state
    } else {
        old_state
    }
}

impl SEIR {
    const T_INF: f64 = 3600.0 * 10.0; // TODO dummy values
    const T_INC: f64 = 3600.0; // TODO dummy values
    const R_0: f64 = 2.5;
    const S_RATIO: f64 = 0.985;
    const E_RATIO: f64 = 0.005;
    const I_RATIO: f64 = 0.01;
    const R_RATIO: f64 = 0.0;

    // TODO not clear if Option<SeirEvent> is needed
    pub fn transition(self, maybe_event: Option<SeirEvent>, rng: &mut XorShiftRng) -> SEIR {
        match maybe_event {
            None => self,
            Some(event) => match (self, event) {
                (s @ SEIR::Sane, SeirEvent::Exposition(prob)) => is_chosen_or(SEIR::Exposed, s, rng, prob),
                (s @ SEIR::Exposed, SeirEvent::Incubation(prob)) => is_chosen_or(SEIR::Infectious, s, rng, prob),
                (s @ SEIR::Infectious, SeirEvent::Recovery(prob)) => is_chosen_or(SEIR::Recovered, s, rng, prob),
                _ => unreachable!(),
            },
        }
    }

    // TODO change that name it's bad
    pub fn get_transition_time_from(state: Self) -> Duration {
        match state {
            SEIR::Sane => Duration::seconds(SEIR::T_INF / SEIR::R_0),
            SEIR::Exposed => Duration::seconds(SEIR::T_INC),
            SEIR::Infectious => Duration::seconds(SEIR::T_INF),
            SEIR::Recovered => unreachable!(),
        }
    }

    // TODO ATM the sigma is simply the duration / 2. Maybe look that a bit more.
    // TODO also change that name it's bad
    pub fn get_transition_time_uncertainty_from(state: Self) -> Duration {
        match state {
            SEIR::Sane => Duration::seconds(SEIR::T_INF / SEIR::R_0 / 2.0),
            SEIR::Exposed => Duration::seconds(SEIR::T_INC / 2.0),
            SEIR::Infectious => Duration::seconds(SEIR::T_INF / 2.0),
            SEIR::Recovered => panic!("Impossible to transition from Recovered state"),
        }
    }

    pub fn get_initial_ratio(state: Self) -> f64 {
        match state {
            SEIR::Sane => SEIR::S_RATIO,
            SEIR::Exposed => SEIR::E_RATIO,
            SEIR::Infectious => SEIR::I_RATIO,
            SEIR::Recovered => SEIR::R_RATIO,
        }
    }
}
