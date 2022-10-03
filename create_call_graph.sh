#!/bin/bash

cargo build
valgrind --tool=callgrind ./target/debug/agent_sim
gprof2dot -f callgrind -o callgrind.out.dot callgrind.out.*
dot -Tsvg callgrind.out.dot -o callgrind.svg