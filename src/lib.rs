use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::time::Instant;

pub mod agent;
pub mod geometry;
use crate::agent::{Agent, ContactGraph, Status, Task};
use crate::geometry::{Rect, Vec2D};

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
    pub agents: Quadtree,
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
    pub last_step_duration: u128,
}

impl World<rand::prelude::ThreadRng> {
    pub fn new(size: Vec2D<f64>) -> Self {
        World {
            agents: Quadtree::new(Rect::new(Vec2D::new_zero(), size)),
            curr_step: 0,
            step_size: 1,
            size,
            infected: 0,
            rng: Box::new(rand::thread_rng()),
            contacts: ContactGraph::new(),
            time: Time::new(),
            structures: HashMap::new(),
            last_step_duration: 0,
        }
    }

    pub fn new_with_agents(size: Vec2D<f64>, agents: Vec<Agent>) -> Self {
        World {
            agents: Quadtree::new_with_agents(Rect::new(Vec2D::new_zero(), size), agents),
            curr_step: 0,
            step_size: 1,
            size,
            infected: 0,
            rng: Box::new(rand::thread_rng()),
            contacts: ContactGraph::new(),
            time: Time::new(),
            structures: HashMap::new(),
            last_step_duration: 0,
        }
    }
}

impl<R> World<R>
where
    R: Rng,
{
    /// Randomly infect an agent as the index case. Does not check if other
    /// agents are already infected
    /// Is not actually random since the quadtree makes it hard.
    // TODO(tslnc04): Make this actually random, and add it to the contact graph
    pub fn infect_index_case(&mut self) {
        if self.agents.len() == 0 {
            return;
        }

        if let Some(agent) = self.agents.get_agent_mut(0) {
            agent.status = Status::Exposed(0);
            self.infected += 1;
        }
    }

    pub fn step(&mut self) {
        let now = Instant::now();
        for agent_id in self.agents.get_agent_ids() {
            let agent = self.agents.get_agent(agent_id).unwrap();
            if !agent.status.is_infectious() {
                continue;
            }

            // setup a 2x2 bounding box centered around the agent
            let bounds = Rect::new_centered(agent.pos, Vec2D::new_one() * 2.0);

            let leaves_to_check = self.agents.find_leaves_in_bounds(bounds);
            for leaf in leaves_to_check {
                if let Some(leaf_node) = self.agents.get_leaf_mut(leaf) {
                    for other_agent_id in leaf_node.agents.iter().copied().collect::<Vec<usize>>() {
                        if let Some(other_agent) = self.agents.get_agent_mut(other_agent_id) {
                            if other_agent.status.is_susceptible() {
                                other_agent.status = Status::Exposed(0);
                                self.contacts.add_node(other_agent_id, Some(agent_id));
                                self.infected += 1;
                            }
                        }
                    }
                }
            }
        }

        for agent in self.agents.iter_mut() {
            agent.step(self.step_size, &mut self.rng);
        }

        self.move_agents();
        self.agents.clean_tree();

        self.curr_step += 1;

        self.time.advance(self.step_size);
        self.last_step_duration = now.elapsed().as_millis();
        // TODO(tslnc04): i'm pretty sure this is backwards. if the goal is to
        // keep the ratio between simulation time and real time constant, the
        // step size should increase when the simulation is running slowly
        // but like also this is kinda unnecessary for now, ig that's why it's a todo
        // if step_duration >= 100 {
        //     self.step_size /= 2;
        // } else if step_duration < 10 {
        //     self.step_size *= 2;
        // }
    }

    fn move_agents(&mut self) {
        let distro = Uniform::from(0.0..1.0);
        for agent_id in self.agents.get_agent_ids() {
            // TODO(tslnc04): get rid of the unwrap
            let agent = self.agents.get_agent_mut(agent_id).unwrap();
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

            let new_pos = agent.pos + movement;
            self.agents.move_agent(agent_id, new_pos);
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
            "----- Time {:2} {}; Infected {}/{}; Dead {}; Step Duration: {} -----",
            self.curr_step,
            self.step_size,
            self.infected,
            self.agents.len(),
            dead,
            self.last_step_duration,
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

// TODO(tslnc04): add a way to remove agents from the tree
// TODO(tslnc04): make the node one type since they're basically the same, the
// enum just complicates things
pub struct Quadtree {
    bounds: Rect<f64>,
    leaf_capacity: usize,
    next_node_id: usize,
    next_agent_id: usize,
    nodes: Vec<QuadtreeNode>,
    agents: HashMap<usize, Agent>,
    open_node_indices: Vec<usize>,
    agent_to_node: HashMap<usize, usize>,
}

impl Quadtree {
    pub fn new(bounds: Rect<f64>) -> Self {
        let mut new_quadtree = Self {
            bounds,
            leaf_capacity: 4,
            next_node_id: 0,
            next_agent_id: 0,
            nodes: Vec::new(),
            agents: HashMap::new(),
            open_node_indices: Vec::new(),
            agent_to_node: HashMap::new(),
        };

        new_quadtree.add_node(QuadtreeNode::Leaf(QuadtreeLeaf::new(None, bounds)));

        new_quadtree
    }

    pub fn new_with_agents(bounds: Rect<f64>, agents: Vec<Agent>) -> Self {
        let mut new_quadtree = Self::new(bounds);

        for agent in agents {
            new_quadtree.add_agent(agent).unwrap();
        }

        new_quadtree
    }

    /// Returns an iterator over the agents in an arbitrary order
    pub fn iter(&self) -> impl Iterator<Item = &Agent> {
        self.agents.values()
    }

    fn iter_nodes(&self) -> impl Iterator<Item = &QuadtreeNode> {
        let open_node_indices = HashSet::<_>::from_iter(self.open_node_indices.iter());
        (0..self.nodes.len())
            .filter(move |i| !open_node_indices.contains(i))
            .map(|i| &self.nodes[i])
    }

    /// Returns a mutable iterator over the agents in an arbitrary order
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Agent> {
        self.agents.values_mut()
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    fn get(&self, id: usize) -> Option<&QuadtreeNode> {
        self.nodes.get(id)
    }

    fn get_leaf(&self, id: usize) -> Option<&QuadtreeLeaf> {
        match self.nodes.get(id) {
            Some(QuadtreeNode::Leaf(leaf)) => Some(leaf),
            _ => None,
        }
    }

    pub fn get_agent(&self, id: usize) -> Option<&Agent> {
        self.agents.get(&id)
    }

    fn get_mut(&mut self, id: usize) -> Option<&mut QuadtreeNode> {
        self.nodes.get_mut(id)
    }

    fn get_leaf_mut(&mut self, id: usize) -> Option<&mut QuadtreeLeaf> {
        match self.nodes.get_mut(id) {
            Some(QuadtreeNode::Leaf(leaf)) => Some(leaf),
            _ => None,
        }
    }

    fn get_agent_mut(&mut self, id: usize) -> Option<&mut Agent> {
        self.agents.get_mut(&id)
    }

    /// Return all of the agent ids currently being used, in an arbitrary order
    fn get_agent_ids(&self) -> Vec<usize> {
        self.agents.keys().copied().collect()
    }

    /// Adds the node to the quadtree and returns the id of the node
    fn add_node(&mut self, node: QuadtreeNode) -> usize {
        if self.open_node_indices.len() > 0 {
            let id = self.open_node_indices.pop().unwrap();
            self.nodes[id] = node;
            id
        } else {
            self.nodes.push(node);
            self.nodes.len() - 1
        }
    }

    /// Removes the node from the quadtree. Due to how this functions
    /// internally, the node is not actually removed and only overwritten when
    /// the index is given to another node.
    fn remove_node(&mut self, id: usize) {
        if id == self.nodes.len() - 1 {
            self.nodes.pop();
        } else {
            self.open_node_indices.push(id);
        }
    }

    /// Guaranteed to return a leaf node
    pub fn get_node_for_pos(&self, pos: Vec2D<f64>) -> Option<usize> {
        let mut curr = 0;

        loop {
            match self.get(curr) {
                Some(QuadtreeNode::Leaf(_)) => return Some(curr),
                Some(QuadtreeNode::Root(root)) => {
                    if !root.bounds.contains(pos) {
                        return None;
                    }

                    curr = root.children[root.bounds.get_quadrant(pos)];
                }
                None => return None,
            }
        }
    }

    /// Guaranteed to return a leaf node. The hint is a node to start from. This
    /// is intended to be used when one is moving an agent, since the agent is
    /// likely moved to a nearby node in the tree.
    fn get_node_for_pos_hinted(&self, pos: Vec2D<f64>, hint: usize) -> Option<usize> {
        let mut curr = hint;

        loop {
            let curr_node = self.get(curr)?;

            if !curr_node.get_bounds().contains(pos) {
                curr = curr_node.get_parent()?;
                continue;
            }

            match curr_node {
                QuadtreeNode::Leaf(_) => return Some(curr),
                QuadtreeNode::Root(root) => {
                    curr = root.children[root.bounds.get_quadrant(pos)];
                }
            }
        }
    }

    fn get_node_for_agent(&self, agent_id: usize) -> Option<usize> {
        self.agent_to_node.get(&agent_id).copied()
    }

    pub fn add_agent(&mut self, agent: Agent) -> Option<usize> {
        let leaf_id = self.get_node_for_pos(agent.pos)?;
        let agent_id = self.next_agent_id;

        self.agents.insert(agent_id, agent);
        self.agent_to_node.insert(agent_id, leaf_id);
        self.get_leaf_mut(leaf_id)?.agents.push(agent_id);

        self.next_agent_id += 1;
        self.check_capacity(leaf_id);

        Some(leaf_id)
    }

    fn check_capacity(&mut self, leaf_id: usize) {
        let leaf = self.get_leaf(leaf_id).unwrap();
        if leaf.agents.len() > self.leaf_capacity && leaf.bounds.get_width() > 2.0 {
            self.split(leaf_id);
        }
    }

    pub fn clean_tree(&mut self) {
        let mut leaf_parents = HashSet::new();
        for leaf in self.iter_nodes().filter(|node| node.is_leaf()) {
            if let Some(parent) = leaf.get_parent() {
                leaf_parents.insert(parent);
            }
        }

        for parent_id in leaf_parents.iter() {
            if let Some(QuadtreeNode::Root(parent)) = self.get(*parent_id) {
                if parent
                    .children
                    .iter()
                    .all(|child| self.get(*child).unwrap().is_leaf())
                    && parent
                        .children
                        .iter()
                        .map(|child| self.get_leaf(*child).unwrap().agents.len())
                        .sum::<usize>()
                        <= self.leaf_capacity
                {
                    self.join(*parent_id);
                }
            }
        }
    }

    fn split(&mut self, id: usize) -> Option<()> {
        let node = self.get_leaf(id)?;
        let node_parent = node.parent.clone();
        let node_bounds = node.bounds;
        let node_agents = node.agents.clone();

        let mut new_leaves = node_bounds
            .quarter()
            .into_iter()
            .map(|bound| QuadtreeLeaf::new(Some(id), bound))
            .collect::<Vec<_>>();

        for agent_id in node_agents.into_iter() {
            let agent = self.get_agent(agent_id)?;
            let quadrant = node_bounds.get_quadrant(agent.pos);
            new_leaves[quadrant].agents.push(agent_id);
        }

        let children = new_leaves
            .into_iter()
            .map(|leaf| self.add_node(QuadtreeNode::Leaf(leaf)))
            .collect::<Vec<_>>();

        for child_id in children.iter() {
            let agents = self.get_leaf(*child_id)?.agents.clone();
            for agent_id in agents.iter() {
                self.agent_to_node.insert(*agent_id, *child_id);
            }
        }

        self.nodes[id] = QuadtreeNode::Root(QuadtreeRoot {
            parent: node_parent,
            bounds: node_bounds,
            children,
        });

        Some(())
    }

    /// Join a root node with leaves as children into a single leaf node
    fn join(&mut self, id: usize) -> Option<()> {
        let node = match self.get(id)? {
            QuadtreeNode::Root(root) => root,
            _ => return None,
        };
        let node_bounds = node.bounds;
        let node_children = node.children.clone();
        let node_agents = node_children
            .iter()
            .flat_map(|child| {
                self.get_leaf(*child)
                    .unwrap()
                    .agents
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        for agent_id in node_agents.iter() {
            self.agent_to_node.insert(*agent_id, id);
        }

        for leaf_id in node_children.iter() {
            self.remove_node(*leaf_id);
        }

        let mut new_leaf = QuadtreeLeaf::new(Some(id), node_bounds);
        new_leaf.agents = node_agents;
        self.nodes[id] = QuadtreeNode::Leaf(new_leaf);

        Some(())
    }

    /// Find every leaf node which has bounds that overlap with the given bounds
    pub fn find_leaves_in_bounds(&self, bounds: Rect<f64>) -> Vec<usize> {
        let mut leaves = Vec::new();
        let mut to_visit = vec![0];

        while to_visit.len() > 0 {
            // unwrap since we know the vector isn't empty
            let curr = to_visit.pop().unwrap();
            let curr_node = self.get(curr).unwrap();

            if !curr_node.get_bounds().intersects(bounds) {
                continue;
            }

            match curr_node {
                QuadtreeNode::Leaf(_) => leaves.push(curr),
                QuadtreeNode::Root(root) => {
                    for child in root.children.iter() {
                        to_visit.push(*child);
                    }
                }
            }
        }

        return leaves;
    }

    fn move_agent(&mut self, agent_id: usize, new_pos: Vec2D<f64>) -> Option<()> {
        let node_id = self.get_node_for_agent(agent_id)?;
        let node_bounds = match self.get(node_id) {
            Some(QuadtreeNode::Leaf(leaf)) => leaf.bounds,
            Some(QuadtreeNode::Root(_)) => panic!("it was a root :("),
            _ => panic!("node not found {}", node_id),
        };

        if !node_bounds.contains(new_pos) {
            let new_node_id = self.get_node_for_pos(new_pos)?;
            let new_node = self.get_leaf_mut(new_node_id)?;

            new_node.agents.push(agent_id);
            self.agent_to_node.insert(agent_id, new_node_id);

            let curr_node = self.get_leaf_mut(node_id)?;
            curr_node.agents.retain(|&id| id != agent_id);

            self.check_capacity(new_node_id);
        }

        self.get_agent_mut(agent_id)?.pos = new_pos;

        Some(())
    }

    pub fn render_as_svg(&self) -> svg::Document {
        let mut doc = svg::Document::new().set(
            "viewBox",
            (
                self.bounds.bl.x,
                self.bounds.bl.y,
                self.bounds.tr.x,
                self.bounds.tr.y,
            ),
        );

        for node in self.iter_nodes() {
            let rect = svg::node::element::Rectangle::new()
                .set("x", node.get_bounds().bl.x)
                .set("y", node.get_bounds().bl.y)
                .set("width", node.get_bounds().get_width())
                .set("height", node.get_bounds().get_height())
                .set("fill", "none")
                .set("stroke", "black");

            doc = doc.add(rect);
        }

        doc
    }

    // TODO(tslnc04): implement rendering using graphviz
}

pub enum QuadtreeNode {
    Root(QuadtreeRoot),
    Leaf(QuadtreeLeaf),
}

impl QuadtreeNode {
    pub fn contains(&self, point: Vec2D<f64>) -> bool {
        match self {
            QuadtreeNode::Root(root) => root.bounds.contains(point),
            QuadtreeNode::Leaf(leaf) => leaf.bounds.contains(point),
        }
    }

    pub fn intersects(&self, bounds: Rect<f64>) -> bool {
        match self {
            QuadtreeNode::Root(root) => root.bounds.intersects(bounds),
            QuadtreeNode::Leaf(leaf) => leaf.bounds.intersects(bounds),
        }
    }

    fn get_bounds(&self) -> Rect<f64> {
        match self {
            QuadtreeNode::Root(root) => root.bounds,
            QuadtreeNode::Leaf(leaf) => leaf.bounds,
        }
    }

    fn get_parent(&self) -> Option<usize> {
        match self {
            QuadtreeNode::Root(root) => root.parent,
            QuadtreeNode::Leaf(leaf) => leaf.parent,
        }
    }

    fn is_leaf(&self) -> bool {
        match self {
            QuadtreeNode::Root(_) => false,
            QuadtreeNode::Leaf(_) => true,
        }
    }
}

pub struct QuadtreeRoot {
    pub parent: Option<usize>,
    pub bounds: Rect<f64>,
    pub children: Vec<usize>,
}

impl QuadtreeRoot {
    pub fn new(parent: Option<usize>, bounds: Rect<f64>, children: Vec<usize>) -> Self {
        Self {
            parent,
            bounds,
            children,
        }
    }
}

pub struct QuadtreeLeaf {
    pub parent: Option<usize>,
    pub bounds: Rect<f64>,
    pub agents: Vec<usize>,
}

impl QuadtreeLeaf {
    pub fn new(parent: Option<usize>, bounds: Rect<f64>) -> Self {
        Self {
            parent,
            bounds,
            agents: Vec::new(),
        }
    }
}

impl<'a> TryFrom<&'a mut QuadtreeNode> for &'a mut QuadtreeLeaf {
    type Error = ();

    fn try_from(value: &'a mut QuadtreeNode) -> Result<Self, Self::Error> {
        match value {
            QuadtreeNode::Leaf(leaf) => Ok(leaf),
            _ => Err(()),
        }
    }
}

const RED: &str = "\x1b[0;31m";
const ORANGE: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[0;33m";
const GREEN: &str = "\x1b[0;32m";
const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[0;34m";
