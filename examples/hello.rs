//
// Hello demo
//

#![cfg_attr(windows, windows_subsystem = "windows")]

use md_rogue::*;

fn main() -> RogueResult<()> {
    let rogue = RogueBuilder::new()
        .with_inner_size(800, 600)
        .with_title("Hello, World!")
        .build();

    let app = App::new();

    run(rogue, app)
}

struct App {}

impl App {
    fn new() -> Self {
        Self {}
    }
}

impl md_rogue::App for App {
    fn start(&mut self) {}

    fn tick(&mut self, _sim_input: SimInput) -> TickResult {
        TickResult::Continue
    }

    fn present(&self, _present_input: PresentInput) {}
}
