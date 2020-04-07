mod pandemic;
mod prob;

use geom::Duration;
pub use pandemic::{Cmd, PandemicModel};
pub use prob::{erf_distrib_bounded, proba_decaying_sigmoid};
use rand::Rng;
use rand_xorshift::XorShiftRng;

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

// TODO not clear if Option<SeirEvent> is needed
fn transition(state: SEIR, maybe_event: Option<SeirEvent>, rng: &mut XorShiftRng) -> SEIR {
    match maybe_event {
        None => state,
        Some(event) => match (state, event) {
            (s @ SEIR::Sane, SeirEvent::Exposition(prob)) => {
                if rng.gen_bool(prob) {
                    SEIR::Exposed
                } else {
                    s
                }
            }
            (s @ SEIR::Exposed, SeirEvent::Incubation(prob)) => {
                if rng.gen_bool(prob) {
                    SEIR::Infectious
                } else {
                    s
                }
            }
            (s @ SEIR::Infectious, SeirEvent::Recovery(prob)) => {
                if rng.gen_bool(prob) {
                    SEIR::Recovered
                } else {
                    s
                }
            }
            _ => unreachable!(),
        },
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
