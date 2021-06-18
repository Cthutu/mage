//
// Hello demo
//

#![cfg_attr(windows, windows_subsystem = "windows")]

use futures::executor::block_on;
use mage::*;

fn main() -> RogueResult<()> {
    let rogue = RogueBuilder::new()
        .with_inner_size(800, 600)
        .with_title("Hello, World!")
        .build();

    let app = HelloDemo::new();

    block_on(run(rogue, app))
}

struct HelloDemo {}

impl HelloDemo {
    fn new() -> Self {
        Self {}
    }
}

impl Game for HelloDemo {
    fn start(&mut self) {}

    fn tick(&mut self, _sim_input: SimInput) -> TickResult {
        TickResult::Continue
    }

    fn present(&self, _present_input: PresentInput) {}
}
