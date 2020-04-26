use crate::pandemic::{AnyTime, State};
use crate::{CarID, Command, Event, OffMapLocation, Grid, Person, PersonID, Scheduler, TripPhaseType, WalkingSimState,};
use geom::{Bounds, Distance, Duration, Pt2D, Time};
use map_model::{Traversable, LaneID, BuildingID, BusStopID, Map};
use rand::Rng;
use rand_xorshift::XorShiftRng;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

// TODO This does not model transmission by surfaces; only person-to-person.
// TODO If two people are in the same shared space indefinitely and neither leaves, we don't model
// transmission. It only occurs when people leave a space.

#[derive(Clone)]
pub struct PandemicModel {
    pop: BTreeMap<PersonID, State>,
    concentration: Grid,
    spacing: Distance,
    delta_t: Duration,

    bldgs: SharedSpace<BuildingID>,
    sidewalks: SharedSpace<LaneID>,
    remote_bldgs: SharedSpace<OffMapLocation>,
    bus_stops: SharedSpace<BusStopID>,
    buses: SharedSpace<CarID>,
    person_to_bus: BTreeMap<PersonID, CarID>,

    rng: XorShiftRng,
    initialized: bool,
}

// You can schedule callbacks in the future by doing scheduler.push(future time, one of these)
// TODO Transition/Transmission may be abit rough see if we change that in the future
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Cmd {
    Poll,
    BecomeHospitalized(PersonID),
    BecomeQuarantined(PersonID),
    CancelFutureTrips(PersonID),
    Transition(PersonID),
    Transmission(PersonID),
}

impl From<(State, PersonID)> for Cmd {
    fn from(s: (State, PersonID)) -> Cmd {
        match s {
            (State::Sane(_), p) => Cmd::Transmission(p),
            (State::Exposed(_), p) | (State::Infectious(_), p) | (State::Hospitalized(_), p) => Cmd::Transition(p),
            (State::Dead(_), _) | (State::Recovered(_), _) => unreachable!(),
        }
    }
}

// TODO Pretend handle_event and handle_cmd also take in some object that lets you do things like:
//
// - replace_future_trips(PersonID, Vec<IndividTrip>)
//
// I'm not exactly sure how this should work yet. Any place you want to change the rest of the
// simulation, just add a comment describing what you want to do exactly, and we'll figure it out
// from there.

impl PandemicModel {
    pub fn new(
        bounds: &Bounds,
        spacing: Distance,
        delta_t: Duration,
        rng: XorShiftRng,
    ) -> PandemicModel {
        let dx = spacing.inner_meters();
        let nx = (bounds.width() / dx).ceil() as usize;
        let ny = (bounds.height() / dx).ceil() as usize;

        PandemicModel {
            pop: BTreeMap::new(),
            concentration: Grid::zero(nx, ny),
            spacing: spacing,
            delta_t: delta_t,

            bldgs: SharedSpace::new(),
            sidewalks: SharedSpace::new(),
            remote_bldgs: SharedSpace::new(),
            bus_stops: SharedSpace::new(),
            buses: SharedSpace::new(),
            person_to_bus: BTreeMap::new(),

            rng,
            initialized: false,
        }
    }

    // Sorry, initialization order of simulations is still a bit messy. This'll be called at
    // Time::START_OF_DAY after all of the people have been created from a Scenario.
    pub fn initialize(&mut self, population: &Vec<Person>, scheduler: &mut Scheduler) {
        assert!(!self.initialized);
        self.initialized = true;

        // Seed initially infected people.
        // TODO the intial time is not well set. it should start "before"
        // the beginning of the day. Also
        for p in population {
            let state = State::new(0.95, 0.05);
            let state = if self.rng.gen_bool(State::ini_exposed_ratio()) {
                let next_state = state
                    .start(
                        AnyTime::from(Time::START_OF_DAY),
                        Duration::seconds(std::f64::MAX),
                        &mut self.rng,
                    )
                    .unwrap();
                let next_state = if self.rng.gen_bool(State::ini_infectious_ratio()) {
                    match next_state
                        .0
                        .next_default(AnyTime::from(Time::START_OF_DAY), &mut self.rng)
                    {
                        (s, Some(t)) => {
                            scheduler.push(t, Command::Pandemic(Cmd::from((s.clone(), p.id))));
                            s
                        }
                        (s, None) => s,
                    }
                } else {
                    match next_state {
                        (s, Some(t)) => {
                            scheduler.push(t, Command::Pandemic(Cmd::from((s.clone(), p.id))));
                            s
                        }
                        (s, None) => s,
                    }
                };
                next_state
            } else {
                state
            };
            self.pop.insert(p.id, state);
        }
        // TODO: no peoplewalk during the night (it's just a hack to see things happen faster).
        // scheduler.push(
        //     Time::START_OF_DAY + Duration::hours(7),
        //     Command::Pandemic(Cmd::Poll),
        // );
    }

    pub fn count_sane(&self) -> usize {
        self.pop
            .iter()
            .filter(|(_, state)| match state {
                State::Sane(_) => true,
                _ => false,
            })
            .count()
        // self.sane.len()
    }

    pub fn count_exposed(&self) -> usize {
        self.pop
            .iter()
            .filter(|(_, state)| match state {
                State::Exposed(_) => true,
                _ => false,
            })
            .count()
        // self.exposed.len()
    }

    pub fn count_infected(&self) -> usize {
        // self.infected.len()
        self.pop
            .iter()
            .filter(|(_, state)| match state {
                State::Infectious(_) | State::Hospitalized(_) => true,
                _ => false,
            })
            .count()
    }

    pub fn count_recovered(&self) -> usize {
        self.pop
            .iter()
            .filter(|(_, state)| match state {
                State::Recovered(_) => true,
                _ => false,
            })
            .count()
        // self.recovered.len()
    }

    pub fn count_dead(&self) -> usize {
        self.pop
            .iter()
            .filter(|(_, state)| match state {
                State::Dead(_) => true,
                _ => false,
            })
            .count()
        // self.recovered.len()
    }

    pub fn count_total(&self) -> usize {
        self.count_sane()
            + self.count_exposed()
            + self.count_infected()
            + self.count_recovered()
            + self.count_dead()
    }

    pub fn handle_event(&mut self, now: Time, ev: &Event, scheduler: &mut Scheduler) {
        assert!(self.initialized);

        match ev {
            Event::AgentEntersTraversable(person, _, t) => {
                if let Some(p) = person {
                    match *t {
                        Traversable::Lane(lid) => self.sidewalks.person_enters_space(now, *p, lid),
                        _ => ()
                    }
                }
            }
            Event::AgentLeavesTraversable(person, _, t) => {
                if let Some(p) = person {
                    match *t {
                        Traversable::Lane(lid) => {
                            if let Some(others) = self.sidewalks.person_leaves_space(now, *p, lid) {
                                self.transmission(now, *p, others, scheduler);
                            } else {
                                panic!("{} left {}, but they weren't inside", p, *t);
                            }
                        },
                        _ => ()
                    }
                }
            }
            Event::PersonEntersBuilding(person, bldg) => {
                self.bldgs.person_enters_space(now, *person, *bldg);
            }
            Event::PersonLeavesBuilding(person, bldg) => {
                if let Some(others) = self.bldgs.person_leaves_space(now, *person, *bldg) {
                    self.transmission(now, *person, others, scheduler);
                } else {
                    panic!("{} left {}, but they weren't inside", person, bldg);
                }
            }
            Event::PersonEntersRemoteBuilding(person, loc) => {
                self.remote_bldgs
                    .person_enters_space(now, *person, loc.clone());
            }
            Event::PersonLeavesRemoteBuilding(person, loc) => {
                if let Some(others) =
                    self.remote_bldgs
                        .person_leaves_space(now, *person, loc.clone())
                {
                    self.transmission(now, *person, others, scheduler);
                } else {
                    panic!("{} left {:?}, but they weren't inside", person, loc);
                }
            }
            Event::TripPhaseStarting(_, p, _, tpt) => {
                let person = *p;
                match tpt {
                    TripPhaseType::WaitingForBus(_, stop) => {
                        self.bus_stops.person_enters_space(now, person, *stop);
                    }
                    TripPhaseType::RidingBus(_, stop, bus) => {
                        let others = self
                            .bus_stops
                            .person_leaves_space(now, person, *stop)
                            .unwrap();
                        self.transmission(now, person, others, scheduler);

                        self.buses.person_enters_space(now, person, *bus);
                        self.person_to_bus.insert(person, *bus);
                    }
                    TripPhaseType::Walking => {
                        // A person can start walking for many reasons, but the only possible state
                        // transition after riding a bus is walking, so use this to detect the end
                        // of a bus ride.
                        if let Some(car) = self.person_to_bus.remove(&person) {
                            let others = self.buses.person_leaves_space(now, person, car).unwrap();
                            self.transmission(now, person, others, scheduler);
                        }
                    }
                    _ => {
                    }
                }
            }
            Event::PersonLeavesMap(_person, _, loc) => {
                if let Some(_loc) = loc {
                    // TODO Could make a SharedSpace for loc.parcel_id, representing buildings
                    // off-map.
                }
            }
            Event::PersonEntersMap(_person, _, loc) => {
                if let Some(_loc) = loc {
                    // TODO But we don't know how long the person spent at these parcels. They
                    // could've taken tons of trips to other off-map parcels in between
                    // PersonLeavesMap and PersonEntersMap.
                }
            }
            _ => {}
        }
    }

    fn pedestrian_transmission(
        &mut self,
        now: Time,
        sane_walkers: &Vec<(PersonID, Pt2D)>,
        bounds: &Bounds,
        dx: f64,
        scheduler: &mut Scheduler,
    ) {
        for (p, w) in sane_walkers {
            let x = ((w.x() - bounds.min_x) / dx).floor() as usize;
            let y = ((w.y() - bounds.min_y) / dx).floor() as usize;

            // TODO must think about how to make the transition more realistic
            // probably an erf function?
            if self.rng.gen_bool(self.concentration[(x, y)] / 100.0) {
                // When poeple become expose
                let state = self.pop.remove(&p).unwrap();
                assert_eq!(
                    state.get_event_time().unwrap().inner_seconds(),
                    std::f64::INFINITY
                );
                // The probability of transmission is handled in the if above
                let state = state.start_now(AnyTime::from(now), &mut self.rng).unwrap();
                let state = match state {
                    (s, Some(t)) => {
                        scheduler.push(t, Command::Pandemic(Cmd::from((s.clone(), *p))));
                        s
                    }
                    (s, None) => s,
                };
                self.pop.insert(*p, state);
                // if self.rng.gen_bool(0.1) {
                //     scheduler.push(
                //         now + self.rand_duration(Duration::hours(1), Duration::hours(3)),
                //         Command::Pandemic(Cmd::BecomeHospitalized(person)),
                //     );
                // }
            }
        }
    }

    pub fn handle_cmd(
        &mut self,
        now: Time,
        cmd: Cmd,
        walkers: &WalkingSimState,
        map: &Map,
        scheduler: &mut Scheduler,
    ) {
        assert!(self.initialized);

        // TODO Here we might enforce policies. Like severe -> become hospitalized
        // Symptomatic -> stay quaratined, and/or track contacts to quarantine them too (or test
        // them)
        match cmd {
            Cmd::BecomeHospitalized(_person) => {
                // self.hospitalized.insert(person);
            }
            Cmd::BecomeQuarantined(_person) => {
                // self.quarantined.insert(person);
            }
            // This is handled by the rest of the simulation
            Cmd::CancelFutureTrips(_) => unreachable!(),
            Cmd::Poll => {
                let infectious_ped = walkers
                    .get_unzoomed_agents(now, map)
                    .into_iter()
                    .filter_map(|x| {
                        // normally it is a person (already filtered by walkers)
                        if self.is_infectious(x.person.unwrap()) {
                            Some(x.pos)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Pt2D>>();

                let sane_ped = walkers
                    .get_unzoomed_agents(now, map)
                    .into_iter()
                    .filter_map(|x| {
                        // normally it is a person (already filtered by walkers)
                        if self.is_sane(x.person.unwrap()) {
                            Some((x.person.unwrap(), x.pos))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(PersonID, Pt2D)>>();

                if infectious_ped.len() > 0 {
                    self.concentration.add_sources(
                        &infectious_ped,
                        map.get_bounds(),
                        self.spacing.inner_meters(),
                        self.delta_t.inner_seconds(),
                        1.0,
                    );
                }

                self.concentration.diffuse(
                    0.002,
                    0.002,
                    self.spacing.inner_meters(),
                    self.delta_t.inner_seconds(),
                );
                self.concentration.absorb(0.01);
                // if now.inner_seconds() as usize % 3600 == 0 {
                //     // println!("{:?}", self.concentration);
                //     self.concentration
                //         .draw_autoscale(&format!("test_{}.png", now.inner_seconds() as usize));
                // }

                if sane_ped.len() > 0 {
                    self.pedestrian_transmission(
                        now,
                        &sane_ped,
                        map.get_bounds(),
                        self.spacing.inner_meters(),
                        scheduler,
                    );
                }
                scheduler.push(now + self.delta_t, Command::Pandemic(Cmd::Poll));
            },
            Cmd::Transition(person) => {
                self.transition(now, person, scheduler);
            },
            Cmd::Transmission(_) => {
                // TODO don't know if something can happen here.
                unreachable!()
            }
        }
    }

    pub fn get_time(&self, person: PersonID) -> Option<Time> {
        match self.pop.get(&person) {
            Some(state) => state.get_time(),
            None => unreachable!(),
        }
    }

    pub fn is_sane(&self, person: PersonID) -> bool {
        match self.pop.get(&person) {
            Some(state) => state.is_sane(),
            None => unreachable!(),
        }
    }

    pub fn is_infectious(&self, person: PersonID) -> bool {
        match self.pop.get(&person) {
            Some(state) => state.is_infectious(),
            None => unreachable!(),
        }
    }

    pub fn is_exposed(&self, person: PersonID) -> bool {
        match self.pop.get(&person) {
            Some(state) => state.is_exposed(),
            None => unreachable!(),
        }
    }

    pub fn is_recovered(&self, person: PersonID) -> bool {
        match self.pop.get(&person) {
            Some(state) => state.is_recovered(),
            None => unreachable!(),
        }
    }

    pub fn is_dead(&self, person: PersonID) -> bool {
        match self.pop.get(&person) {
            Some(state) => state.is_dead(),
            None => unreachable!(),
        }
    }

    fn infectious_contact(&self, person: PersonID, other: PersonID) -> Option<PersonID> {
        if self.is_sane(person) && self.is_infectious(other) {
            return Some(person);
        } else if self.is_infectious(person) && self.is_sane(other) {
            return Some(other);
        }
        None
    }

    fn transmission(
        &mut self,
        now: Time,
        person: PersonID,
        other_occupants: Vec<(PersonID, Duration)>,
        scheduler: &mut Scheduler,
    ) {
        // person has spent some duration in the same space as other people. Does transmission
        // occur?
        for (other, overlap) in other_occupants {
            if let Some(pid) = self.infectious_contact(person, other) {
                self.become_exposed(now, overlap, pid, scheduler);
            }
        }
    }

    // transition from a state to another without interaction with others
    fn transition(&mut self, now: Time, person: PersonID, scheduler: &mut Scheduler) {
        let state = self.pop.remove(&person).unwrap();
        let state = state.next(AnyTime::from(now), &mut self.rng);
        let state = match state {
            (s, Some(t)) => {
                scheduler.push(t, Command::Pandemic(Cmd::from((s.clone(), person))));
                s
            }
            (s, None) => s,
        };
        self.pop.insert(person, state);

        // if self.rng.gen_bool(0.1) {
        //     scheduler.push(
        //         now + self.rand_duration(Duration::hours(1), Duration::hours(3)),
        //         Command::Pandemic(Cmd::BecomeHospitalized(person)),
        //     );
        // }
    }

    fn become_exposed(
        &mut self,
        now: Time,
        overlap: Duration,
        person: PersonID,
        scheduler: &mut Scheduler,
    ) {
        // When poeple become expose
        let state = self.pop.remove(&person).unwrap();
        assert_eq!(
            state.get_event_time().unwrap().inner_seconds(),
            std::f64::INFINITY
        );
        let state = state
            .start(AnyTime::from(now), overlap, &mut self.rng)
            .unwrap();
        let state = match state {
            (s, Some(t)) => {
                scheduler.push(t, Command::Pandemic(Cmd::from((s.clone(), person))));
                s
            }
            (s, None) => s,
        };
        self.pop.insert(person, state);

        // if self.rng.gen_bool(0.1) {
        //
        // }
    }
}

#[derive(Clone)]
struct SharedSpace<T: Ord> {
    // Since when has a person been in some shared space?
    // TODO This is an awkward data structure; abstutil::MultiMap is also bad, because key removal
    // would require knowing the time. Want something closer to
    // https://guava.dev/releases/19.0/api/docs/com/google/common/collect/Table.html.
    occupants: BTreeMap<T, Vec<(PersonID, Time)>>,
}

impl<T: Ord> SharedSpace<T> {
    fn new() -> SharedSpace<T> {
        SharedSpace {
            occupants: BTreeMap::new(),
        }
    }

    fn person_enters_space(&mut self, now: Time, person: PersonID, space: T) {
        self.occupants
            .entry(space)
            .or_insert_with(Vec::new)
            .push((person, now));
    }

    // Returns a list of all other people that the person was in the shared space with, and how
    // long their time overlapped. If it returns None, then a bug must have occurred, because
    // somebody has left a space they never entered.
    fn person_leaves_space(
        &mut self,
        now: Time,
        person: PersonID,
        space: T,
    ) -> Option<Vec<(PersonID, Duration)>> {
        // TODO Messy to mutate state inside a retain closure
        let mut inside_since: Option<Time> = None;
        let occupants = self.occupants.entry(space).or_insert_with(Vec::new);
        occupants.retain(|(p, t)| {
            if *p == person {
                inside_since = Some(*t);
                false
            } else {
                true
            }
        });
        // TODO Bug!
        let inside_since = inside_since?;

        Some(
            occupants
                .iter()
                .map(|(p, t)| (*p, now - (*t).max(inside_since)))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(x: usize) -> Time {
        Time::START_OF_DAY + Duration::hours(x)
    }

    #[test]
    fn test_overlap() {
        let mut space = SharedSpace::new();
        let mut now = time(0);

        let bldg1 = BuildingID(1);
        let bldg2 = BuildingID(2);

        let person1 = PersonID(1);
        let person2 = PersonID(2);
        let person3 = PersonID(3);

        // Only one person
        space.person_enters_space(now, person1, bldg1);
        now = time(1);
        assert_eq!(
            space.person_leaves_space(now, person1, bldg1),
            Some(Vec::new())
        );

        // Two people at the same time
        now = time(2);
        space.person_enters_space(now, person1, bldg2);
        space.person_enters_space(now, person2, bldg2);
        now = time(3);
        assert_eq!(
            space.person_leaves_space(now, person1, bldg2),
            Some(vec![(person2, Duration::hours(1))])
        );

        // Bug
        assert_eq!(space.person_leaves_space(now, person3, bldg2), None);

        // Different times
        now = time(5);
        space.person_enters_space(now, person1, bldg1);
        now = time(6);
        space.person_enters_space(now, person2, bldg1);
        now = time(7);
        space.person_enters_space(now, person3, bldg1);
        now = time(10);
        assert_eq!(
            space.person_leaves_space(now, person1, bldg1),
            Some(vec![
                (person2, Duration::hours(4)),
                (person3, Duration::hours(3))
            ])
        );
        now = time(12);
        assert_eq!(
            space.person_leaves_space(now, person2, bldg1),
            Some(vec![(person3, Duration::hours(5))])
        );
    }
}
