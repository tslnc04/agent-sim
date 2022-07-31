use agent_sim::{dist, Agent, Status};
use rand::Rng;
use std::{thread, time};

fn main() {
    let mut rng = rand::thread_rng();
    let mut agents = Vec::<Agent>::new();

    for i in 0..10 {
        for j in 0..10 {
            if rng.gen_bool(0.4) {
                agents.push(Agent::new((i as f64, j as f64)));
            }
        }
    }

    agents[0].infect();

    for time in 0..100 {
        for i in 0..agents.len() {
            for j in i + 1..agents.len() {
                if dist(agents[i].pos, agents[j].pos) < 2.0 {
                    if let Status::Infectious(_) = agents[i].status {
                        agents[j].infect();
                    } else if let Status::Infectious(_) = agents[j].status {
                        agents[i].infect();
                    }
                }
            }
        }

        println!("----- Time {:2} -----", time);
        for agent in agents.iter() {
            match agent.status {
                Status::Susceptible => print!(" S "),
                Status::Exposed(t) => print!("E{} ", t),
                Status::Infectious(t) => print!("I{} ", t),
                Status::Recovered => print!(" R "),
            }
        }
        println!();

        for agent in agents.iter_mut() {
            agent.step();
        }

        thread::sleep(time::Duration::from_millis(300));
    }
}
