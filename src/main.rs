use agent_sim::{agent::Agent, geometry::Vec2D, StructureType, World};
use rand::Rng;
// use std::fs;
// use std::process::Command;
use std::collections::HashMap;
use std::{thread, time};

const CLEAR: &str = "\x1b[H\x1b[2J";

fn main() {
    let mut rng = rand::thread_rng();
    let mut agents = Vec::<Agent>::new();

    for i in 0..50 {
        for j in 0..50 {
            if rng.gen_bool(0.6) {
                agents.push(Agent::new(
                    Vec2D::new(i as f64, j as f64),
                    (rng.gen::<f64>() + 0.5) * 3.0 / 86400.0,
                ));
            }
        }
    }

    let mut world = World::new_with_agents(Vec2D::new(50.0, 50.0), agents);
    world.step_size = 86400;
    world.infect_index_case();
    world
        .place_structures(HashMap::from([
            (StructureType::Home, 4),
            (StructureType::Work, 2),
            (StructureType::School, 1),
        ]))
        .unwrap();
    world.assign_structures();

    println!("{}{}", CLEAR, world);
    for step in 0..151 {
        world.step();

        if step % 10 == 0 {
            println!("{}{}", CLEAR, world);
            thread::sleep(time::Duration::from_millis(300));
        }
    }

    // println!("Average degree: {}", world.contacts.get_average_degree());
    // svg::save("quadtree.svg", &world.agents.render_as_svg()).unwrap();

    // fs::write("example.dot", world.contacts.to_string()).expect("file writing error");
    // Command::new("dot")
    //     .arg("-Tsvg")
    //     .arg("example.dot")
    //     .arg("-o")
    //     .arg("example.svg")
    //     .output()
    //     .expect("failed to execute process");
}
