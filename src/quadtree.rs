use crate::{Agent, Rect, Vec2D};
use std::collections::{HashMap, HashSet};

pub struct Quadtree {
    bounds: Rect<f64>,
    leaf_capacity: usize,
    next_agent_id: usize,
    nodes: Vec<Node>,
    agents: HashMap<usize, Agent>,
    open_node_indices: Vec<usize>,
    agent_to_node: HashMap<usize, usize>,
}

impl Quadtree {
    pub fn new(bounds: Rect<f64>) -> Self {
        let mut new_quadtree = Self {
            bounds,
            leaf_capacity: 4,
            next_agent_id: 0,
            nodes: Vec::new(),
            agents: HashMap::new(),
            open_node_indices: Vec::new(),
            agent_to_node: HashMap::new(),
        };

        new_quadtree.add_node(Node::new_leaf(None, bounds));

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

    fn iter_nodes(&self) -> impl Iterator<Item = &Node> {
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

    fn get(&self, id: usize) -> Option<&Node> {
        self.nodes.get(id)
    }

    fn get_leaf(&self, id: usize) -> Option<&Node> {
        let node = self.get(id)?;

        match node.typ {
            NodeType::Leaf => Some(node),
            _ => None,
        }
    }

    pub fn get_agent(&self, id: usize) -> Option<&Agent> {
        self.agents.get(&id)
    }

    fn get_mut(&mut self, id: usize) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    fn get_leaf_mut(&mut self, id: usize) -> Option<&mut Node> {
        let node = self.get_mut(id)?;

        match node.typ {
            NodeType::Leaf => Some(node),
            _ => None,
        }
    }

    pub fn get_agent_mut(&mut self, id: usize) -> Option<&mut Agent> {
        self.agents.get_mut(&id)
    }

    /// Return all of the agent ids currently being used, in an arbitrary order
    pub fn get_agent_ids(&self) -> Vec<usize> {
        self.agents.keys().copied().collect()
    }

    /// Adds the node to the quadtree and returns the id of the node
    fn add_node(&mut self, node: Node) -> usize {
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
            let node = self.get(curr)?;
            match node.typ {
                NodeType::Leaf => return Some(curr),
                NodeType::Root => {
                    if !node.bounds.contains(pos) {
                        return None;
                    }

                    curr = node.children[node.bounds.get_quadrant(pos)];
                }
            }
        }
    }

    /// Guaranteed to return a leaf node. The hint is a node to start from. This
    /// is intended to be used when one is moving an agent, since the agent is
    /// likely moved to a nearby node in the tree.
    // fn get_node_for_pos_hinted(&self, pos: Vec2D<f64>, hint: usize) -> Option<usize> {
    //     let mut curr = hint;

    //     loop {
    //         let curr_node = self.get(curr)?;

    //         if !curr_node.get_bounds().contains(pos) {
    //             curr = curr_node.get_parent()?;
    //             continue;
    //         }

    //         match curr_node {
    //             QuadtreeNode::Leaf(_) => return Some(curr),
    //             QuadtreeNode::Root(root) => {
    //                 curr = root.children[root.bounds.get_quadrant(pos)];
    //             }
    //         }
    //     }
    // }

    fn get_node_for_agent(&self, agent_id: usize) -> Option<usize> {
        self.agent_to_node.get(&agent_id).copied()
    }

    pub fn add_agent(&mut self, agent: Agent) -> Option<usize> {
        let leaf_id = self.get_node_for_pos(agent.pos)?;
        let agent_id = self.next_agent_id;

        self.agents.insert(agent_id, agent);
        self.agent_to_node.insert(agent_id, leaf_id);
        self.get_leaf_mut(leaf_id)?.children.push(agent_id);

        self.next_agent_id += 1;
        self.check_capacity(leaf_id);

        Some(leaf_id)
    }

    pub fn remove_agent(&mut self, agent_id: usize) -> Option<Agent> {
        let leaf_id = self.get_node_for_agent(agent_id)?;
        let leaf = self.get_leaf_mut(leaf_id)?;
        leaf.children.retain(|id| *id != agent_id);

        self.agent_to_node.remove(&agent_id);
        self.agents.remove(&agent_id)
    }

    fn check_capacity(&mut self, leaf_id: usize) {
        let leaf = self.get_leaf(leaf_id).unwrap();
        if leaf.children.len() > self.leaf_capacity && leaf.bounds.get_width() > 2.0 {
            self.split(leaf_id);
        }
    }

    pub fn clean_tree(&mut self) {
        let mut leaf_parents = HashSet::new();
        for leaf in self.iter_nodes().filter(|node| node.is_leaf()) {
            if let Some(parent) = leaf.parent {
                leaf_parents.insert(parent);
            }
        }

        for parent_id in leaf_parents.iter() {
            if let Some(parent) = self.get(*parent_id) {
                if !parent.is_leaf()
                    && parent
                        .children
                        .iter()
                        .all(|child| self.get(*child).unwrap().is_leaf())
                    && parent
                        .children
                        .iter()
                        .map(|child| self.get_leaf(*child).unwrap().children.len())
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
        let node_agents = node.children.clone();

        let mut new_leaves = node_bounds
            .quarter()
            .into_iter()
            .map(|bound| Node::new_leaf(Some(id), bound))
            .collect::<Vec<_>>();

        for agent_id in node_agents.into_iter() {
            let agent = self.get_agent(agent_id)?;
            let quadrant = node_bounds.get_quadrant(agent.pos);
            new_leaves[quadrant].children.push(agent_id);
        }

        let children = new_leaves
            .into_iter()
            .map(|leaf| self.add_node(leaf))
            .collect::<Vec<_>>();

        for child_id in children.iter() {
            let agents = self.get_leaf(*child_id)?.children.clone();
            for agent_id in agents.iter() {
                self.agent_to_node.insert(*agent_id, *child_id);
            }
        }

        self.nodes[id] = Node::new_root(node_parent, node_bounds, children);

        Some(())
    }

    /// Join a root node with leaves as children into a single leaf node
    fn join(&mut self, id: usize) -> Option<()> {
        let node = self.get(id)?;
        let node_bounds = node.bounds;
        let node_children = node.children.clone();
        let node_agents = node_children
            .iter()
            .flat_map(|child| {
                self.get_leaf(*child)
                    .unwrap()
                    .children
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

        let mut new_leaf = Node::new_leaf(Some(id), node_bounds);
        new_leaf.children = node_agents;
        self.nodes[id] = new_leaf;

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

            if !curr_node.bounds.intersects(bounds) {
                continue;
            }

            match curr_node.typ {
                NodeType::Leaf => leaves.push(curr),
                NodeType::Root => {
                    for child in curr_node.children.iter() {
                        to_visit.push(*child);
                    }
                }
            }
        }

        return leaves;
    }

    pub fn find_agents_in_bounds(&self, bounds: Rect<f64>) -> Vec<usize> {
        let leaves = self.find_leaves_in_bounds(bounds);
        leaves
            .iter()
            .flat_map(|leaf| self.get_leaf(*leaf).unwrap().children.iter().copied())
            .collect()
    }

    pub fn move_agent(&mut self, agent_id: usize, new_pos: Vec2D<f64>) -> Option<()> {
        let node_id = self.get_node_for_agent(agent_id)?;
        let node_bounds = self.get_leaf(node_id)?.bounds;

        if !node_bounds.contains(new_pos) {
            let new_node_id = self.get_node_for_pos(new_pos)?;
            let new_node = self.get_leaf_mut(new_node_id)?;

            new_node.children.push(agent_id);
            self.agent_to_node.insert(agent_id, new_node_id);

            let curr_node = self.get_leaf_mut(node_id)?;
            curr_node.children.retain(|&id| id != agent_id);

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
                .set("x", node.bounds.bl.x)
                .set("y", node.bounds.bl.y)
                .set("width", node.bounds.get_width())
                .set("height", node.bounds.get_height())
                .set("fill", "none")
                .set("stroke", "black");

            doc = doc.add(rect);
        }

        doc
    }
}

enum NodeType {
    Root,
    Leaf,
}
struct Node {
    typ: NodeType,
    parent: Option<usize>,
    children: Vec<usize>,
    bounds: Rect<f64>,
}

impl Node {
    fn new_root(parent: Option<usize>, bounds: Rect<f64>, children: Vec<usize>) -> Self {
        Self {
            typ: NodeType::Root,
            parent,
            bounds,
            children,
        }
    }

    fn new_leaf(parent: Option<usize>, bounds: Rect<f64>) -> Self {
        Self {
            typ: NodeType::Leaf,
            parent,
            bounds,
            children: Vec::new(),
        }
    }

    fn is_leaf(&self) -> bool {
        match self.typ {
            NodeType::Leaf => true,
            NodeType::Root => false,
        }
    }
}
