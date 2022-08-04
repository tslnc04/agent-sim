use agent_sim::{Agent, World};
use rand::Rng;
// use std::fs;
use std::{thread, time};

const CLEAR: &str = "\x1b[H\x1b[2J";

fn main() {
    let mut rng = rand::thread_rng();
    let mut agents = Vec::<Agent>::new();

    for i in 0..40 {
        for j in 0..40 {
            if rng.gen_bool(0.3) {
                agents.push(Agent::new((i as f64, j as f64)));
            }
        }
    }

    let mut world = World::new_with_agents((40.0, 40.0), agents);
    world.infect_index_case();

    println!("{}{}", CLEAR, world);
    for _ in 0..150 {
        world.step();
        println!("{}{}", CLEAR, world);

        thread::sleep(time::Duration::from_millis(100));
    }

    // fs::write("example.dot", world.contacts.to_string()).expect("file writing error");
}
