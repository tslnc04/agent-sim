use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

pub mod agent;
pub mod geometry;
use crate::agent::{Agent, ContactGraph, Status, Task};
use crate::geometry::Vec2D;

/// Representation of time within the simulation. `abs_time` is a variation on
/// epoch time, which is the number of seconds since the simulation began.
/// `day_time` is similar, but reset every day. `day_of_week` is an integer
/// representing which day of the week it is, starting with Sunday as 0.
struct Time {
    day_of_week: i64,
    abs_time: i64,
    day_time: i64,
}

impl Time {
    pub fn new() -> Self {
        Self {
            day_of_week: 0,
            abs_time: 0,
            day_time: 0,
        }
    }

    pub fn advance(&mut self, seconds: i64) {
        self.abs_time += seconds;
        self.day_time += seconds;

        if self.day_time >= 86400 {
            self.day_time -= 86400;
            self.day_of_week += 1;
            if self.day_of_week >= 7 {
                self.day_of_week = 0;
            }
        }
    }
}

pub enum Structure {
    Home(Vec2D<f64>),
    Workplace(Vec2D<f64>),
    School(Vec2D<f64>),
}

impl TryFrom<(&str, Vec2D<f64>)> for Structure {
    type Error = String;

    fn try_from(item: (&str, Vec2D<f64>)) -> Result<Self, Self::Error> {
        match item.0 {
            "home" => Ok(Structure::Home(item.1)),
            "workplace" => Ok(Structure::Workplace(item.1)),
            "school" => Ok(Structure::School(item.1)),
            _ => Err(format!("Invalid structure: {}", item.0)),
        }
    }
}

/// World is the wrapper for all simulation, with this struct being responsible
/// for managing all of the agents and anything else that can happen within the
/// simulation.
pub struct World<R: Rng> {
    agents: Vec<Agent>,
    /// curr_step measures simulation steps independent of time.
    curr_step: i64,
    /// step_size is the number of seconds between each simulation step.
    pub step_size: i64,
    size: Vec2D<f64>,
    infected: i64,
    rng: Box<R>,
    pub contacts: ContactGraph,
    homes: Vec<Vec2D<f64>>,
    workplaces: Vec<Vec2D<f64>>,
    time: Time,
    structures: HashMap<&'static str, Vec<Structure>>,
}

impl World<rand::prelude::ThreadRng> {
    pub fn new(size: Vec2D<f64>) -> Self {
        World {
            agents: Vec::new(),
            curr_step: 0,
            step_size: 1,
            size: size,
            infected: 0,
            rng: Box::new(rand::thread_rng()),
            contacts: ContactGraph::new(),
            homes: Vec::new(),
            workplaces: Vec::new(),
            time: Time::new(),
            structures: World::<rand::prelude::ThreadRng>::new_structure_map(),
        }
    }

    pub fn new_with_agents(size: Vec2D<f64>, agents: Vec<Agent>) -> Self {
        World {
            agents: agents,
            curr_step: 0,
            step_size: 1,
            size: size,
            infected: 0,
            rng: Box::new(rand::thread_rng()),
            contacts: ContactGraph::new(),
            homes: Vec::new(),
            workplaces: Vec::new(),
            time: Time::new(),
            structures: World::<rand::prelude::ThreadRng>::new_structure_map(),
        }
    }
}

impl<R> World<R>
where
    R: Rng,
{
    /// Randomly infect an agent as the index case. Does not check if other
    /// agents are already infected
    pub fn infect_index_case(&mut self) {
        let index_agent_id = self.rng.gen_range(0..self.agents.len());
        if let Some(index_agent) = self.agents.get_mut(index_agent_id) {
            if index_agent.status.is_susceptible() {
                index_agent.status = Status::Exposed(0);
                self.contacts.add_node(index_agent_id, None);
            }
        }
    }

    pub fn step(&mut self) {
        let now = Instant::now();
        for i in 0..self.agents.len() {
            for j in i + 1..self.agents.len() {
                if self.agents[i].pos.dist(self.agents[j].pos) < 2.0 {
                    // agent j attempts to infect agent i, if that fails agent i
                    // attempts to infect agent j
                    if let Some(true) = self.infect_agent(i, j) {
                        self.infected += 1;
                    } else if let Some(true) = self.infect_agent(j, i) {
                        self.infected += 1;
                    }
                }
            }
        }

        for agent in self.agents.iter_mut() {
            agent.step(self.step_size, &mut self.rng);
        }

        self.move_agents();

        self.curr_step += 1;
        self.time.advance(self.step_size);
        let step_duration = now.elapsed().as_millis();
        if step_duration >= 100 {
            self.step_size /= 2;
        } else if step_duration < 10 {
            self.step_size *= 2;
        }
    }

    fn move_agents(&mut self) {
        let distro = Uniform::from(0.0..1.0);
        for agent in self.agents.iter_mut() {
            if agent.status.is_dead() {
                continue;
            }

            let dest = match agent.task {
                Task::Home => agent.home,
                Task::Work => agent.workplace,
                Task::None => agent.home,
                Task::School => agent.school,
            };

            let dir = dest - agent.pos;

            if dir.mag() < 1e-6 {
                agent.task = match agent.task {
                    Task::Home => Task::Work,
                    Task::Work => Task::Home,
                    Task::None => Task::None,
                    Task::School => Task::Home,
                };
                continue;
            }

            let movement = (dir.normalize()
                * distro.sample(&mut self.rng)
                * agent.speed
                * self.step_size as f64)
                .clamp_mag(dir.mag());
            agent.pos += movement;
        }
    }

    /// Apply a random movement to each of the agents with a magnitude in the
    /// range of [0, max_mag). World boundaries are handled by clipping
    /// position, not by wrapping.
    #[allow(dead_code)]
    fn move_agents_random(&mut self, max_mag: f64) {
        let distro = Uniform::from(-1.0..1.0);
        for agent in self.agents.iter_mut() {
            if agent.status.is_dead() {
                continue;
            }

            // generate a movement vector with components in the range of [-1, 1)
            let movement = Vec2D::new(distro.sample(&mut self.rng), distro.sample(&mut self.rng));
            // scale the movement based on maximum magnitude and update position
            agent.pos += movement.normalize() * max_mag;
            // clamp position to world size
            agent.pos.x = agent.pos.x.clamp(0.0, self.size.x);
            agent.pos.y = agent.pos.y.clamp(0.0, self.size.y);
        }
    }

    fn infect_agent(&mut self, recip_agent_id: usize, src_agent_id: usize) -> Option<bool> {
        if self.agents.get(recip_agent_id)?.status.is_susceptible()
            && self.agents.get(src_agent_id)?.status.is_infectious()
        {
            self.agents.get_mut(recip_agent_id)?.status = Status::Exposed(0);
            self.contacts.add_node(recip_agent_id, Some(src_agent_id));
            Some(true)
        } else {
            Some(false)
        }
    }

    pub fn place_homes_and_workplaces(&mut self, homes: usize, workplaces: usize) {
        let x_distro = Uniform::from(0.0..self.size.x);
        let y_distro = Uniform::from(0.0..self.size.y);
        for _ in 0..homes {
            self.homes.push(Vec2D::new(
                x_distro.sample(&mut self.rng),
                y_distro.sample(&mut self.rng),
            ));
        }
        for _ in 0..workplaces {
            self.workplaces.push(Vec2D::new(
                x_distro.sample(&mut self.rng),
                y_distro.sample(&mut self.rng),
            ));
        }
    }

    // TODO(tslnc04): convert the whole hashmap setup to use enum variants and
    // then separate the data for each kind of structure. using strings when
    // it's just going into an enum doesn't make sense
    pub fn place_structures(&mut self, counts: HashMap<&'static str, usize>) -> Result<(), String> {
        let x_distro = Uniform::from(0.0..self.size.x);
        let y_distro = Uniform::from(0.0..self.size.y);

        for (structure, count) in counts.iter() {
            if !self.structures.contains_key(structure) {
                self.structures.insert(*structure, Vec::new());
            }

            if let Some(structure_vec) = self.structures.get_mut(structure) {
                for _ in 0..*count {
                    structure_vec.push(Structure::try_from((
                        *structure,
                        Vec2D::new(
                            x_distro.sample(&mut self.rng),
                            y_distro.sample(&mut self.rng),
                        ),
                    ))?);
                }
            }
        }

        Ok(())
    }

    pub fn assign_homes_and_workplaces(&mut self) {
        let homes_distro = Uniform::from(0..self.homes.len());
        let workplaces_distro = Uniform::from(0..self.workplaces.len());
        for agent in self.agents.iter_mut() {
            agent.home = self.homes[homes_distro.sample(&mut self.rng)];
            agent.workplace = self.workplaces[workplaces_distro.sample(&mut self.rng)];
        }
    }

    // TODO(tslnc04): randomly assign structures to agents, take into account
    // age and changing behavior since schools shouldn't go to older agents and
    // workplaces not to young agents
    pub fn assign_structures(&mut self) {
        return;
    }

    fn new_structure_map() -> HashMap<&'static str, Vec<Structure>> {
        HashMap::from([
            ("home", Vec::new()),
            ("workplace", Vec::new()),
            ("school", Vec::new()),
        ])
    }
}

/// Debug output for World is simply a listing of the agents and their statuses
impl<R> fmt::Debug for World<R>
where
    R: Rng,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "----- Time {:2}; Infected {} -----",
            self.curr_step, self.infected
        )?;
        for agent in self.agents.iter() {
            write!(f, "{}", agent)?
        }

        Ok(())
    }
}

/// Display output for World is a visualization of the agents on a grid
impl<R> fmt::Display for World<R>
where
    R: Rng,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dead: i64 = 0;
        for agent in self.agents.iter() {
            if agent.status.is_dead() {
                dead += 1;
            }
        }
        // since spatial storing of agents hasn't been implemented yet, each
        // grid square is an O(1) operation that only takes the first agent at a
        // given grid square. this could be problematic
        writeln!(
            f,
            "----- Time {:2} {}; Infected {}/{}; Dead {} -----",
            self.curr_step,
            self.step_size,
            self.infected,
            self.agents.len(),
            dead,
        )?;

        for i in 0..self.size.x.ceil() as i64 {
            for j in 0..self.size.y.ceil() as i64 {
                let mut object_found = false;
                for home in self.homes.iter() {
                    if home.x.round() as i64 == i && home.y.round() as i64 == j {
                        write!(f, " H ")?;
                        object_found = true;
                        break;
                    }
                }

                if object_found {
                    continue;
                }

                object_found = false;
                for workplace in self.workplaces.iter() {
                    if workplace.x.round() as i64 == i && workplace.y.round() as i64 == j {
                        write!(f, " W ")?;
                        object_found = true;
                        break;
                    }
                }

                if object_found {
                    continue;
                }

                let mut agent_found = false;
                for agent in self.agents.iter() {
                    if agent.pos.x.round() as i64 == i && agent.pos.y.round() as i64 == j {
                        write!(f, "{}", agent)?;
                        agent_found = true;
                        break;
                    }
                }

                if !agent_found {
                    write!(f, "   ")?;
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

const RED: &str = "\x1b[0;31m";
const ORANGE: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[0;33m";
const GREEN: &str = "\x1b[0;32m";
const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[0;34m";
