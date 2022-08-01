use agent_sim::{Agent, World};
use rand::Rng;
use std::{thread, time};

fn main() {
    let mut rng = rand::thread_rng();
    let mut agents = Vec::<Agent>::new();

    for i in 0..10 {
        for j in 0..10 {
            if rng.gen_bool(0.1) {
                agents.push(Agent::new((i as f64, j as f64)));
            }
        }
    }

    agents[0].infect();

    let mut world = World::new_with_agents((10.0, 10.0), agents);

    println!("{}", world);
    for _ in 0..100 {
        world.step();
        println!("{}", world);

        thread::sleep(time::Duration::from_millis(1000));
    }
}
