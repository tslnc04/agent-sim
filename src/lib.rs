use rand::Rng;
use std::fmt;

/// World is the wrapper for all simulation, with this struct being responsible
/// for managing all of the agents and anything else that can happen within the
/// simulation.
pub struct World {
    agents: Vec<Agent>,
    curr_step: i64,
    size: (f64, f64),
}

impl World {
    pub fn new(size: (f64, f64)) -> Self {
        World {
            agents: Vec::new(),
            curr_step: 0,
            size: size,
        }
    }

    pub fn new_with_agents(size: (f64, f64), agents: Vec<Agent>) -> Self {
        World {
            agents: agents,
            curr_step: 0,
            size: size,
        }
    }

    pub fn step(&mut self) {
        for i in 0..self.agents.len() {
            for j in i + 1..self.agents.len() {
                if dist(self.agents[i].pos, self.agents[j].pos) < 2.0 {
                    if self.agents[i].status.is_infectious() {
                        self.agents[j].infect();
                    } else if self.agents[j].status.is_infectious() {
                        self.agents[i].infect();
                    }
                }
            }
        }

        for agent in self.agents.iter_mut() {
            agent.step();
        }

        self.move_agents_random(1.0);

        self.curr_step += 1;
    }

    /// Apply a random movement to each of the agents with a fixed magnitude.
    /// World boundaries are handled by clipping position, not by wrapping.
    fn move_agents_random(&mut self, mag: f64) {
        for agent in self.agents.iter_mut() {
            // TODO(tslnc04): literally no need to retrieve thread_rng every
            // loop, just put it as a part of the struct since it's used a lot
            let mut rng = rand::thread_rng();
            // generate a unit movement
            // TODO(tslnc04): use Uniform distribution instead of gen_range
            let movement_raw = (rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
            let movement = normalize(movement_raw);
            // add scaled movement to position
            agent.pos.0 += movement.0 * mag;
            agent.pos.1 += movement.1 * mag;
            // clamp positions to world size
            agent.pos.0 = agent.pos.0.clamp(0.0, self.size.0);
            agent.pos.1 = agent.pos.1.clamp(0.0, self.size.1);
        }
    }
}

/// Debug output for World is simply a listing of the agents and their statuses
impl fmt::Debug for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "----- Time {:2} -----", self.curr_step)?;
        for agent in self.agents.iter() {
            write!(f, "{}", agent)?
        }

        Ok(())
    }
}

/// Display output for World is a visualization of the agents on a grid
impl fmt::Display for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // since spatial storing of agents hasn't been implemented yet, each
        // grid square is an O(1) operation that only takes the first agent at a
        // given grid square. this could be problematic
        writeln!(f, "----- Time {:2} -----", self.curr_step)?;

        for i in 0..self.size.0.ceil() as i64 {
            for j in 0..self.size.1.ceil() as i64 {
                let mut agent_found = false;
                for agent in self.agents.iter() {
                    if agent.pos.0.round() as i64 == i && agent.pos.1.round() as i64 == j {
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

/// Represents the status of each agent.
#[derive(Debug)]
pub enum Status {
    Susceptible,
    /// Exposed contains an integer representing the length of time since the
    /// agent was exposed. Only after a certain length of time being exposed
    /// will the agent become infectious.
    Exposed(i64),
    /// Infectious contains an integer that tells how long the agent has been
    /// infectious for. This is used to determine when they should transition
    /// into being recovered.
    Infectious(i64),
    Recovered,
}

impl Status {
    pub fn is_infectious(&self) -> bool {
        matches!(self, Status::Infectious(_))
    }
}

/// Each agent is a distinct entity that gets simulated. It currently only uses
/// the position and the status to determine infection and recovery.
#[derive(Debug)]
pub struct Agent {
    pub pos: (f64, f64),
    pub status: Status,
}

impl Agent {
    pub fn new(pos: (f64, f64)) -> Self {
        Agent {
            pos: pos,
            status: Status::Susceptible,
        }
    }

    pub fn infect(&mut self) {
        if let Status::Susceptible = self.status {
            self.status = Status::Exposed(0);
        }
    }

    pub fn step(&mut self) {
        match self.status {
            Status::Exposed(t) => {
                if t > 2 {
                    self.status = Status::Infectious(0);
                } else {
                    self.status = Status::Exposed(t + 1);
                }
            }
            Status::Infectious(t) => {
                if t > 6 {
                    self.status = Status::Recovered;
                } else {
                    self.status = Status::Infectious(t + 1);
                }
            }
            _ => (),
        }
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            Status::Susceptible => write!(f, " S "),
            Status::Exposed(t) => write!(f, "E{} ", t),
            Status::Infectious(t) => write!(f, "I{} ", t),
            Status::Recovered => write!(f, " R "),
        }
    }
}

fn mag(x: (f64, f64)) -> f64 {
    (x.0 * x.0 + x.1 * x.1).sqrt()
}

fn dist(x: (f64, f64), y: (f64, f64)) -> f64 {
    mag((x.0 - y.0, x.1 - y.1))
}

fn normalize(x: (f64, f64)) -> (f64, f64) {
    let x_mag = mag(x);
    (x.0 / x_mag, x.1 / x_mag)
}
