use std::f64;

/// Represents the status of each agent.
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

/// Each agent is a distinct entity that gets simulated. It currently only uses
/// the position and the status to determine infection and recovery.
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

pub fn dist(x: (f64, f64), y: (f64, f64)) -> f64 {
    ((y.0 - x.0) * (y.0 - x.0) + (y.1 - x.1) * (y.1 - x.1)).sqrt()
}
