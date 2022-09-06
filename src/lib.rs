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

#[derive(Eq, Hash, PartialEq, Debug, Copy, Clone)]
pub enum StructureType {
    Home,
    Work,
    School,
}

impl fmt::Display for StructureType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StructureType::Home => write!(f, "H"),
            StructureType::Work => write!(f, "W"),
            StructureType::School => write!(f, "S"),
        }
    }
}

pub struct Structure {
    pub typ: StructureType,
    pub pos: Vec2D<f64>,
    pub capacity: i64,
}

impl Structure {
    pub fn new(typ: StructureType, pos: Vec2D<f64>, capacity: i64) -> Self {
        Self { typ, pos, capacity }
    }

    pub fn new_without_capacity(typ: StructureType, pos: Vec2D<f64>) -> Self {
        Self {
            typ,
            pos,
            capacity: 0,
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
    time: Time,
    structures: HashMap<StructureType, Vec<Structure>>,
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
            time: Time::new(),
            structures: HashMap::new(),
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
            time: Time::new(),
            structures: HashMap::new(),
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

        // TODO(tslnc04): i'm pretty sure this is backwards. if the goal is to
        // keep the ratio between simulation time and real time constant, the
        // step size should increase when the simulation is running slowly
        // but like also this is kinda unnecessary for now, ig that's why it's a todo
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
                Task::Work => agent.work,
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

    pub fn place_structures(
        &mut self,
        counts: HashMap<StructureType, usize>,
    ) -> Result<(), String> {
        let x_distro = Uniform::from(0.0..self.size.x);
        let y_distro = Uniform::from(0.0..self.size.y);

        for (structure, count) in counts.iter() {
            if !self.structures.contains_key(structure) {
                self.structures.insert(*structure, Vec::new());
            }

            if let Some(structure_vec) = self.structures.get_mut(structure) {
                for _ in 0..*count {
                    structure_vec.push(Structure::new_without_capacity(
                        *structure,
                        Vec2D::new(
                            x_distro.sample(&mut self.rng),
                            y_distro.sample(&mut self.rng),
                        ),
                    ));
                }
            }
        }

        Ok(())
    }

    // TODO(tslnc04): randomly assign structures to agents, take into account
    // age and changing behavior since schools shouldn't go to older agents and
    // workplaces not to young agents
    pub fn assign_structures(&mut self) {
        if let Some(home_structures) = self.structures.get(&StructureType::Home) {
            let homes_distro = Uniform::from(0..home_structures.len());
            for agent in self.agents.iter_mut() {
                agent.home = home_structures[homes_distro.sample(&mut self.rng)].pos;
            }
        }

        if let Some(work_structures) = self.structures.get(&StructureType::Work) {
            let work_distro = Uniform::from(0..work_structures.len());
            for agent in self.agents.iter_mut() {
                agent.work = work_structures[work_distro.sample(&mut self.rng)].pos;
            }
        }

        if let Some(school_structures) = self.structures.get(&StructureType::School) {
            let schools_distro = Uniform::from(0..school_structures.len());
            for agent in self.agents.iter_mut() {
                agent.school = school_structures[schools_distro.sample(&mut self.rng)].pos;
            }
        }
    }

    // TODO(tslnc04): determine whether this function is worth keeping
    #[allow(dead_code)]
    fn new_structure_map() -> HashMap<StructureType, Vec<Structure>> {
        HashMap::from([
            (StructureType::Home, Vec::new()),
            (StructureType::Work, Vec::new()),
            (StructureType::School, Vec::new()),
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
                let mut structure_found = false;
                for (structure_type, structures) in self.structures.iter() {
                    for structure in structures.iter() {
                        if structure.pos.x.floor() as i64 == i
                            && structure.pos.y.floor() as i64 == j
                        {
                            write!(f, " {} ", structure_type)?;
                            structure_found = true;
                            break;
                        }
                    }
                }

                if structure_found {
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

pub struct Quadtree {
    pos: Vec2D<f64>,
    dim: Vec2D<f64>,
    leaf_capacity: usize,
    next_id: usize,
    pub nodes: HashMap<usize, QuadtreeNode>,
}

impl Quadtree {
    pub fn new(pos: Vec2D<f64>, dim: Vec2D<f64>) -> Self {
        let mut new_quadtree = Self {
            pos,
            dim,
            leaf_capacity: 4,
            next_id: 0,
            nodes: HashMap::new(),
        };

        new_quadtree.add_node(QuadtreeNode::Leaf(QuadtreeLeaf::new(pos, dim)));

        new_quadtree
    }

    pub fn get(&self, id: usize) -> Option<&QuadtreeNode> {
        self.nodes.get(&id)
    }

    pub fn get_leaf(&self, id: usize) -> Option<&QuadtreeLeaf> {
        if let Some(QuadtreeNode::Leaf(leaf)) = self.get(id) {
            Some(leaf)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut QuadtreeNode> {
        self.nodes.get_mut(&id)
    }

    pub fn get_leaf_mut(&mut self, id: usize) -> Option<&mut QuadtreeLeaf> {
        if let Some(QuadtreeNode::Leaf(leaf)) = self.get_mut(id) {
            Some(leaf)
        } else {
            None
        }
    }

    fn add_node(&mut self, node: QuadtreeNode) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, node);
        id
    }

    pub fn get_node_for_pos(&self, pos: Vec2D<f64>) -> Option<usize> {
        let mut curr = 0;

        loop {
            if let Some(curr_node) = self.get(curr) {
                if !curr_node.contains(pos) {
                    return None;
                }

                match curr_node {
                    QuadtreeNode::Leaf(_) => return Some(curr),
                    QuadtreeNode::Root(root) => curr = pos.get_bounds_quadrant(root.pos, root.dim),
                }
            } else {
                return None;
            }
        }
    }

    pub fn add_agent(&mut self, agent: Agent) -> Option<usize> {
        if let Some(node_id) = self.get_node_for_pos(agent.pos) {
            if let Some(node) = self.get_leaf_mut(node_id) {
                node.agents.push(agent);
                self.check_capacity(node_id);
                return Some(node_id);
            }
        }

        None
    }

    fn check_capacity(&mut self, id: usize) {
        if let Some(node) = self.get_leaf_mut(id) {
            if node.agents.len() > self.leaf_capacity {
                self.split(id);
            }
        }
    }

    fn split(&mut self, id: usize) {
        let mut children = Vec::new();

        for y in 0..=1 {
            for x in 0..=1 {
                if let Some(node) = self.get_leaf_mut(id) {
                    let mut new_leaf = QuadtreeLeaf::new(
                        node.pos + Vec2D::new(x as f64, 1.0 - y as f64) * node.dim / 2.0,
                        node.dim / 2.0,
                    );
                    // preferred implementation if drain filter were stabilized
                    // new_leaf.agents = node
                    //     .agents
                    //     .drain_filter(|agent| agent.pos.is_in_bounds(new_leaf.pos, new_leaf.dim))
                    //     .collect();
                    let mut i = 0;
                    while i < node.agents.len() {
                        if node.agents[i].pos.is_in_bounds(new_leaf.pos, new_leaf.dim) {
                            new_leaf.agents.push(node.agents.swap_remove(i));
                        } else {
                            i += 1;
                        }
                    }

                    children.push(self.add_node(QuadtreeNode::Leaf(new_leaf)));
                }
            }
        }

        if let Some(node) = self.get_leaf(id) {
            let new_root = QuadtreeNode::Root(QuadtreeRoot::new(node.pos, node.dim, children));

            self.nodes.insert(id, new_root);
        }
    }
}

pub enum QuadtreeNode {
    Root(QuadtreeRoot),
    Leaf(QuadtreeLeaf),
}

impl QuadtreeNode {
    pub fn contains(&self, pos: Vec2D<f64>) -> bool {
        match self {
            QuadtreeNode::Root(root) => pos.is_in_bounds(root.pos, root.dim),
            QuadtreeNode::Leaf(leaf) => pos.is_in_bounds(leaf.pos, leaf.dim),
        }
    }
}

pub struct QuadtreeRoot {
    pub pos: Vec2D<f64>,
    pub dim: Vec2D<f64>,
    pub children: Vec<usize>,
}

impl QuadtreeRoot {
    pub fn new(pos: Vec2D<f64>, dim: Vec2D<f64>, children: Vec<usize>) -> Self {
        Self { pos, dim, children }
    }
}

pub struct QuadtreeLeaf {
    pub pos: Vec2D<f64>,
    pub dim: Vec2D<f64>,
    pub agents: Vec<Agent>,
}

impl QuadtreeLeaf {
    pub fn new(pos: Vec2D<f64>, dim: Vec2D<f64>) -> Self {
        Self {
            pos,
            dim,
            agents: Vec::new(),
        }
    }
}

const RED: &str = "\x1b[0;31m";
const ORANGE: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[0;33m";
const GREEN: &str = "\x1b[0;32m";
const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[0;34m";
