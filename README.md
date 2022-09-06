# agent-sim

Agent-based infectious disease modeling, written in Rust.

## What?

agent-sim is currently a highly work in progress infectious disease model. It is
not made with the intention of competing with models developed by actual
epidemiologists, but rather be a fun way to run large-scale agent simulations.

Ultimately, the goal is to have a way to run simulations on the scale of many
thousand agents, with dynamically evolving diseases. See `TODO.md` for more on
milestones and general philosophy on where I want the project to go.

## Usage

The model can be run with `cargo run`, by default displaying a visualization of
the world being simulated. Commented-out code provides a way to visualize the
contact tracing graph with `graph-viz`.

## Licensing

Licensed under MIT.

Copyright 2022 Kirsten Laskoski
