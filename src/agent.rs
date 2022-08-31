use rand::Rng;

use crate::geometry::Vec2D;
use crate::{BLUE, GREEN, ORANGE, RED, RESET, YELLOW};
use std::collections::HashMap;
use std::fmt;

/// Represents the status of each agent.
#[derive(Debug)]
pub enum Status {
    Susceptible,
    /// Exposed contains an integer representing the length of time since the
    /// agent was exposed. Only after a certain length of time being exposed
    /// will the agent become infectious. Time is measured in seconds.
    Exposed(i64),
    /// Infectious contains an integer that tells how long the agent has been
    /// infectious for. This is used to determine when they should transition
    /// into being recovered. Time is measured in seconds.
    Infectious(i64),
    Recovered,
    Dead,
}

impl Status {
    pub fn is_infectious(&self) -> bool {
        matches!(self, Status::Infectious(_))
    }

    pub fn is_susceptible(&self) -> bool {
        matches!(self, Status::Susceptible)
    }

    pub fn is_dead(&self) -> bool {
        matches!(self, Status::Dead)
    }
}

/// Task represents the current action that the agent is taking, allowing one to
/// determine where the agent is headed
// TODO(tslnc04): decide whether the task should include a none option or if it should just be
// wrapped in an Option<> when that would be necessary
#[derive(Debug)]
pub enum Task {
    Work,
    Home,
    School,
    None,
}

/// Each agent is a distinct entity that gets simulated. It currently only uses
/// the position and the status to determine infection and recovery.
#[derive(Debug)]
pub struct Agent {
    pub pos: Vec2D<f64>,
    pub status: Status,
    pub task: Task,
    pub home: Vec2D<f64>,
    pub workplace: Vec2D<f64>,
    pub school: Vec2D<f64>,
    /// speed is the distance the agent can move per second, regardless of the
    /// size of the simulation step.
    pub speed: f64,
    /// age is the time the agent has been alive for, in seconds. This is
    /// relative to the life of the agent, not the simulation.
    pub age: i64,
}

impl Agent {
    pub fn new(pos: Vec2D<f64>, speed: f64) -> Self {
        Agent {
            pos: pos,
            status: Status::Susceptible,
            task: Task::Home,
            home: Vec2D::new_nan(),
            workplace: Vec2D::new_nan(),
            school: Vec2D::new_nan(),
            speed: speed,
            age: 0,
        }
    }

    pub fn step<R: Rng>(&mut self, step_size: i64, rng: &mut R) {
        match self.status {
            Status::Exposed(t) => {
                // Simulates the incubation period for the agent
                if t > 21 * 86400 {
                    self.status = Status::Infectious(0);
                } else {
                    self.status = Status::Exposed(t + step_size);
                }
            }
            Status::Infectious(t) => {
                // Simulates the infectious period for the agent
                if t > 28 * 86400 {
                    self.status = Status::Recovered;
                } else {
                    self.status = Status::Infectious(t + step_size);
                }
            }
            _ => (),
        }

        self.age += step_size;

        if rng.gen_bool(self.death_probability(step_size)) {
            self.status = Status::Dead;
        }
    }

    /// Calculate the probability of death at a given age in seconds. These are
    /// based on the average of the male and female probabilities based on the
    /// SSA Actuarial Life Table for 2019 TR 2022. The piecewise linear
    /// components are just made to roughly approximate the actual function for
    /// annual probability of mortality.
    ///
    /// The annual probability of mortality is converted to probability over a
    /// step through dividing by the number of seconds in a year and multiplying
    /// by number of seconds in a step.
    ///
    /// A flat increase of 0.1% per year is added for infectious agents.
    ///
    /// https://www.ssa.gov/oact/STATS/table4c6.html
    pub fn death_probability(&self, step_size: i64) -> f64 {
        (match self.age / (365 * 86400) {
            0..=20 => 0.001,
            21..=50 => 0.0001 * (self.age as f64 - 20.0) + 0.001,
            51..=80 => 0.0001 * (self.age as f64 - 50.0) + 0.005,
            81..=100 => 0.01 * (self.age as f64 - 80.0) + 0.05,
            101..=119 => 0.03 * (self.age as f64 - 100.0) + 0.2,
            _ => 0.9,
        } + if self.status.is_infectious() {
            0.001
        } else {
            0.0
        }) / (365.0 * 86400.0)
            * step_size as f64
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            Status::Susceptible => write!(f, "{} S {}", GREEN, RESET),
            Status::Exposed(t) => write!(f, "{}E{} {}", ORANGE, t / 86400, RESET),
            Status::Infectious(t) => write!(f, "{}I{} {}", RED, t / 86400, RESET),
            Status::Recovered => write!(f, "{} R {}", YELLOW, RESET),
            Status::Dead => write!(f, "{} D {}", BLUE, RESET),
        }
    }
}

// build the graph during the simulation and use that to replace the src field
// of the agent struct
// TODO(tslnc04): turn this into a nonsimple digraph; eliminate the strict
// parent-child hierarchy and allow things like double edges
#[derive(Debug)]
pub struct ContactGraph {
    // this doesn't support deletion of nodes?
    // probably not too necessary though
    nodes: Vec<ContactNode>,
    /// agent_table provides a lookup between agent ids (keys) and nodes indices (values)
    agent_table: HashMap<usize, usize>,
}

impl ContactGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            agent_table: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, agent_id: usize, parent: Option<usize>) {
        let graph_parent = match parent {
            Some(parent_agent) => match self.agent_table.get(&parent_agent) {
                Some(parent_index) => Some(*parent_index),
                None => None,
            },
            None => None,
        };
        let new_node = ContactNode {
            index: self.nodes.len(),
            parent: graph_parent,
            children: Vec::new(),
            agent_id: agent_id,
        };

        if graph_parent.is_some() {
            if let Some(parent_node) = self.nodes.get_mut(graph_parent.unwrap()) {
                parent_node.children.push(new_node.index);
            }
        }

        self.agent_table.insert(agent_id, self.nodes.len());
        self.nodes.push(new_node);
    }

    pub fn get_average_degree(&self) -> f64 {
        let mut total_degree = 0;
        for node in &self.nodes {
            total_degree += node.get_degree();
        }
        total_degree as f64 / self.nodes.len() as f64
    }
}

impl fmt::Display for ContactGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "digraph ContactGraph {{")?;
        for node in self.nodes.iter() {
            write!(f, "{}", node)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

/// Each ContactNode stores the place of an agent in the contact-tracing graph.
/// The parent is the source of the infection and the children are all the agents
/// infected by this node's agent.
#[derive(Debug)]
struct ContactNode {
    index: usize,
    parent: Option<usize>,
    children: Vec<usize>,
    agent_id: usize,
}

impl ContactNode {
    pub fn get_degree(&self) -> usize {
        self.children.len() + if self.parent.is_some() { 1 } else { 0 }
    }
}

impl fmt::Display for ContactNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ContactNode{}[label=\"Agent {}\"];",
            self.index, self.agent_id
        )?;
        for child in self.children.iter() {
            write!(f, "ContactNode{} -> ContactNode{};", self.index, child)?;
        }
        Ok(())
    }
}
