mod pandemic;

use geom::{Duration, Time};
pub use pandemic::{Cmd, PandemicModel};
use rand::Rng;
use rand_distr::{Distribution, Exp, Normal};
use rand_xorshift::XorShiftRng;
use std::ops;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct AnyTime(f64);

impl AnyTime {
    fn inner_seconds(&self) -> f64 {
        self.0
    }

    pub fn is_finite(&self) -> bool {
        self.0.is_finite()
    }
}

impl ops::Add<Duration> for AnyTime {
    type Output = AnyTime;

    fn add(self, other: Duration) -> AnyTime {
        AnyTime(self.0 + other.inner_seconds())
    }
}

impl ops::AddAssign<Duration> for AnyTime {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl ops::Sub<Duration> for AnyTime {
    type Output = AnyTime;

    fn sub(self, other: Duration) -> AnyTime {
        AnyTime(self.0 - other.inner_seconds())
    }
}

impl ops::Sub for AnyTime {
    type Output = Duration;

    fn sub(self, other: AnyTime) -> Duration {
        Duration::seconds(self.0 - other.0)
    }
}

impl From<Time> for AnyTime {
    fn from(t: Time) -> AnyTime {
        AnyTime(t.inner_seconds())
    }
}

impl Into<Time> for AnyTime {
    fn into(self) -> Time {
        Time::START_OF_DAY + Duration::seconds(self.inner_seconds())
    }
}

impl From<f64> for AnyTime {
    fn from(t: f64) -> AnyTime {
        AnyTime(t)
    }
}

#[derive(Debug, Clone)]
pub enum StateEvent {
    Exposition,
    Incubation,
    Hospitalization,
    Recovery,
    Death,
}

#[derive(Debug, Clone)]
pub struct Event {
    s: StateEvent,
    p_hosp: f64,  // probability of people being hospitalized after infection
    p_death: f64, // probability of dying after hospitalizaion
    t: AnyTime,
}

impl Event {
    fn next(&self, now: AnyTime, rng: &mut XorShiftRng) -> (State, Option<Time>) {
        match self.s {
            StateEvent::Exposition => {
                let next_time = now + State::get_time_normal(State::T_INC, State::T_INC / 4.0, rng);
                (
                    State::Exposed((
                        Event {
                            s: StateEvent::Incubation,
                            p_hosp: self.p_hosp,
                            p_death: self.p_death,
                            t: next_time,
                        },
                        now.into(),
                    )),
                    Some(next_time.into()),
                )
            }
            StateEvent::Incubation => {
                if rng.gen_bool(self.p_death) {
                    let next_time =
                        now + State::get_time_normal(State::T_INF, State::T_INF / 4.0, rng);
                    (
                        State::Infectious((
                            Event {
                                s: StateEvent::Recovery,
                                p_hosp: self.p_hosp,
                                p_death: self.p_death,
                                t: next_time,
                            },
                            now.into(),
                        )),
                        Some(next_time.into()),
                    )
                } else {
                    let next_time =
                        now + State::get_time_normal(State::T_INF, State::T_INF / 4.0, rng);
                    (
                        State::Infectious((
                            Event {
                                s: StateEvent::Hospitalization,
                                p_hosp: self.p_hosp,
                                p_death: self.p_death,
                                t: next_time,
                            },
                            now.into(),
                        )),
                        Some(next_time.into()),
                    )
                }
            }
            StateEvent::Hospitalization => {
                if rng.gen_bool(self.p_hosp) {
                    let next_time =
                        now + State::get_time_normal(State::T_INF, State::T_INF / 4.0, rng);
                    (
                        State::Hospitalized((
                            Event {
                                s: StateEvent::Recovery,
                                p_hosp: self.p_hosp,
                                p_death: self.p_death,
                                t: next_time,
                            },
                            now.into(),
                        )),
                        Some(next_time.into()),
                    )
                } else {
                    let next_time =
                        now + State::get_time_normal(State::T_INF, State::T_INF / 4.0, rng);
                    (
                        State::Hospitalized((
                            Event {
                                s: StateEvent::Death,
                                p_hosp: self.p_hosp,
                                p_death: self.p_death,
                                t: next_time,
                            },
                            now.into(),
                        )),
                        Some(next_time.into()),
                    )
                }
            }
            StateEvent::Death => (State::Dead(now.into()), None),
            StateEvent::Recovery => (State::Recovered(now.into()), None),
        }
    }
}

#[derive(Debug, Clone)]
pub enum State {
    Sane((Event, Time)),
    Exposed((Event, Time)),
    Infectious((Event, Time)),
    Hospitalized((Event, Time)),
    Recovered(Time),
    Dead(Time),
}

impl State {
    const T_INF: f64 = 3600.0 * 24.0 * 1.0; // TODO dummy values
    const T_INC: f64 = 3600.0 * 24.0 * 1.0 / 24.0; // TODO dummy values
    const R_0: f64 = 2.5;
    // const S_RATIO: f64 = 0.985;
    const E_RATIO: f64 = 0.2;
    const I_RATIO: f64 = 0.5;
    // const R_RATIO: f64 = 0.0;

    pub fn ini_infectious_ratio() -> f64 {
        Self::I_RATIO
    }

    pub fn ini_exposed_ratio() -> f64 {
        Self::E_RATIO
    }

    fn new(p_hosp: f64, p_death: f64) -> Self {
        Self::Sane((
            Event {
                s: StateEvent::Exposition,
                p_hosp,
                p_death,
                t: AnyTime::from(std::f64::INFINITY),
            },
            Time::START_OF_DAY,
        ))
    }

    fn get_time_exp(lambda: f64, rng: &mut XorShiftRng) -> geom::Duration {
        let normal = Exp::new(lambda).unwrap();
        Duration::seconds(normal.sample(rng))
    }

    fn get_time_normal(mu: f64, sigma: f64, rng: &mut XorShiftRng) -> geom::Duration {
        let normal = Normal::new(mu, sigma).unwrap();
        Duration::seconds(normal.sample(rng))
    }

    fn is_sane(&self) -> bool {
        match self {
            State::Sane((ev, _)) => !ev.t.is_finite(),
            _ => false,
        }
    }

    fn is_exposed(&self) -> bool {
        match self {
            State::Exposed(_) => true,
            _ => false,
        }
    }

    fn is_infectious(&self) -> bool {
        match self {
            State::Infectious(_) | State::Hospitalized(_) => true,
            _ => false,
        }
    }

    fn is_recovered(&self) -> bool {
        match self {
            State::Recovered(_) => true,
            _ => false,
        }
    }

    fn is_dead(&self) -> bool {
        match self {
            State::Dead(_) => true,
            _ => false,
        }
    }

    pub fn get_time(&self) -> Option<Time> {
        match self {
            Self::Sane(_) => None,
            Self::Recovered(t)
            | Self::Dead(t)
            | Self::Exposed((_, t))
            | Self::Infectious((_, t))
            | Self::Hospitalized((_, t)) => Some(*t),
        }
    }

    pub fn get_event_time(&self) -> Option<AnyTime> {
        match self {
            Self::Sane((ev, _))
            | Self::Exposed((ev, _))
            | Self::Infectious((ev, _))
            | Self::Hospitalized((ev, _)) => Some(ev.t),
            Self::Recovered(_) | Self::Dead(_) => None,
        }
    }

    // pub fn set_time(self, new_time: AnyTime) -> Self {
    //     match self {
    //         Self::Sane(Event {
    //             s,
    //             p_hosp,
    //             p_death,
    //             t: _,
    //         }) => Self::Sane(Event {
    //             s,
    //             p_hosp,
    //             p_death,
    //             t: new_time,
    //         }),
    //         _ => unreachable!(),
    //     }
    // }

    // TODO: not sure if we want an option here...
    pub fn next_default(self, default: AnyTime, rng: &mut XorShiftRng) -> (Self, Option<Time>) {
        // TODO: when #![feature(bindings_after_at)] reaches stable
        // rewrite this part with it
        match self {
            Self::Sane((ev, _)) => (Self::Sane((ev, default.into())), Some(default.into())),
            Self::Exposed((ev, _)) => ev.next(default, rng),
            Self::Infectious((ev, _)) => ev.next(default, rng),
            Self::Hospitalized((ev, _)) => ev.next(default, rng),
            Self::Recovered(_) => (Self::Recovered(default.into()), None),
            Self::Dead(_) => (Self::Dead(default.into()), None),
        }
    }

    // TODO: not sure if we want an option here...
    pub fn next(self, now: AnyTime, rng: &mut XorShiftRng) -> (Self, Option<Time>) {
        // TODO: when #![feature(bindings_after_at)] reaches stable
        // rewrite this part with it
        match self {
            Self::Sane((ev, t)) => (Self::Sane((ev, t)), Some(t.into())),
            Self::Exposed((ev, _)) => ev.next(now, rng),
            Self::Infectious((ev, _)) => ev.next(now, rng),
            Self::Hospitalized((ev, _)) => ev.next(now, rng),
            Self::Recovered(t) => (Self::Recovered(t), None),
            Self::Dead(t) => (Self::Dead(t), None),
        }
    }

    // TODO: not sure if we want an option here... I guess here we want because we could have
    pub fn start(
        self,
        now: AnyTime,
        overlap: Duration,
        rng: &mut XorShiftRng,
    ) -> Result<(Self, Option<Time>), String> {
        // rewrite this part with it
        match self {
            Self::Sane((ev, t)) => {
                if overlap >= Self::get_time_exp(State::R_0 / State::T_INF, rng) {
                    Ok(ev.next(now, rng))
                } else {
                    Ok((Self::Sane((ev, t)), None))
                }
            }
            _ => Err(String::from(
                "Error: impossible to start from a non-sane situation.",
            )),
        }
    }

    // TODO: not sure if we want an option here... I guess here we want because we could have
    pub fn start_now(self, now: AnyTime, rng: &mut XorShiftRng) -> Result<(Self, Option<Time>), String> {
        // rewrite this part with it
        match self {
            Self::Sane((ev, _)) => Ok(ev.next(now, rng)),
            _ => Err(String::from(
                "Error: impossible to start from a non-sane situation.",
            )),
        }
    }
}
