use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use std::collections::HashMap;
use std::f64::consts::SQRT_2;
use std::fmt;

/// World is the wrapper for all simulation, with this struct being responsible
/// for managing all of the agents and anything else that can happen within the
/// simulation.
pub struct World<R: Rng> {
    agents: Vec<Agent>,
    curr_step: i64,
    size: (f64, f64),
    infected: i64,
    rng: Box<R>,
    pub contacts: ContactGraph,
}

impl World<rand::prelude::ThreadRng> {
    pub fn new(size: (f64, f64)) -> Self {
        World {
            agents: Vec::new(),
            curr_step: 0,
            size: size,
            infected: 0,
            rng: Box::new(rand::thread_rng()),
            contacts: ContactGraph::new(),
        }
    }

    pub fn new_with_agents(size: (f64, f64), agents: Vec<Agent>) -> Self {
        World {
            agents: agents,
            curr_step: 0,
            size: size,
            infected: 0,
            rng: Box::new(rand::thread_rng()),
            contacts: ContactGraph::new(),
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
        for i in 0..self.agents.len() {
            for j in i + 1..self.agents.len() {
                if dist(self.agents[i].pos, self.agents[j].pos) < 2.0 {
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
            agent.step();
        }

        self.move_agents_random(1.0);

        self.curr_step += 1;
    }

    /// Apply a random movement to each of the agents with a magnitude in the
    /// range of [0, max_mag). World boundaries are handled by clipping
    /// position, not by wrapping.
    fn move_agents_random(&mut self, max_mag: f64) {
        let distro = Uniform::from(-1.0..1.0);
        for agent in self.agents.iter_mut() {
            // generate a movement vector with components in the range of [-1, 1)
            let movement = (distro.sample(&mut self.rng), distro.sample(&mut self.rng));
            // scale the movement based on maximum magnitude and update position
            agent.pos.0 += movement.0 * max_mag / SQRT_2;
            agent.pos.1 += movement.1 * max_mag / SQRT_2;
            // clamp position to world size
            agent.pos.0 = agent.pos.0.clamp(0.0, self.size.0);
            agent.pos.1 = agent.pos.1.clamp(0.0, self.size.1);
        }
    }

    fn infect_agent(&mut self, recip_agent_id: usize, src_agent_id: usize) -> Option<bool> {
        if self.agents.get(recip_agent_id)?.status.is_susceptible()
            && self.agents.get(src_agent_id)?.status.is_infectious()
        {
            self.agents.get_mut(recip_agent_id)?.status = Status::Exposed(0);
            self.agents.get_mut(recip_agent_id)?.src = src_agent_id;
            self.contacts.add_node(recip_agent_id, Some(src_agent_id));
            Some(true)
        } else {
            Some(false)
        }
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
        // since spatial storing of agents hasn't been implemented yet, each
        // grid square is an O(1) operation that only takes the first agent at a
        // given grid square. this could be problematic
        writeln!(
            f,
            "----- Time {:2}; Infected {} -----",
            self.curr_step, self.infected
        )?;

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

    pub fn is_susceptible(&self) -> bool {
        matches!(self, Status::Susceptible)
    }
}

/// Each agent is a distinct entity that gets simulated. It currently only uses
/// the position and the status to determine infection and recovery.
#[derive(Debug)]
pub struct Agent {
    pub pos: (f64, f64),
    pub status: Status,
    /// src is the ID of the agent that infected this agent. Since it defaults
    /// to 0, one should check the status before assuming that this agent has
    /// been infected by src.
    src: usize,
}

impl Agent {
    pub fn new(pos: (f64, f64)) -> Self {
        Agent {
            pos: pos,
            status: Status::Susceptible,
            src: 0,
        }
    }

    /// Attempts to infect the agent, returns true only when the agent was
    /// susceptible and sucessfully infected.
    /// Deprecated in favor of control from the World struct
    pub fn infect(&mut self) -> bool {
        if let Status::Susceptible = self.status {
            self.status = Status::Exposed(0);
            true
        } else {
            false
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
            Status::Susceptible => write!(f, "{} S {}", GREEN, RESET),
            Status::Exposed(t) => write!(f, "{}E{} {}", ORANGE, t, RESET),
            Status::Infectious(t) => write!(f, "{}I{} {}", RED, t, RESET),
            Status::Recovered => write!(f, "{} R {}", YELLOW, RESET),
        }
    }
}

// TODO(tslnc04): implement the graph
// build the graph during the simulation and use that to replace the src field
// of the agent struct
#[derive(Debug)]
pub struct ContactGraph {
    nodes: Vec<ContactNode>,
    // root really just represents the index case
    root: usize,
    /// agent_table provides a lookup between agent ids (keys) and nodes indices (values)
    agent_table: HashMap<usize, usize>,
}

impl ContactGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: 0,
            agent_table: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, agent_id: usize, parent: Option<usize>) {
        // TODO(tslnc04): refactor this, it cannot be this ugly
        // consider making parent not an option since only the root shouldn't
        // have a parent
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

        // TODO(tslnc04): add a function for setting the root
        if graph_parent.is_none() {
            self.root = self.nodes.len();
        } else if let Some(parent_node) = self.nodes.get_mut(graph_parent.unwrap()) {
            parent_node.children.push(new_node.index);
        }

        self.agent_table.insert(agent_id, self.nodes.len());
        self.nodes.push(new_node);
    }
}

impl fmt::Display for ContactGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "digraph ContactGraph {{")?;
        for node in self.nodes.iter() {
            write!(f, "{}", node);
        }
        write!(f, "}}");
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

const RED: &str = "\x1b[0;31m";
const ORANGE: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[0;33m";
const GREEN: &str = "\x1b[0;32m";
const RESET: &str = "\x1b[0m";
