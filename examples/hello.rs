//
// Hello demo
//

#![cfg_attr(windows, windows_subsystem = "windows")]

use futures::executor::block_on;
use mage::*;
use rand::Rng;

fn main() -> RogueResult<()> {
    let rogue = RogueBuilder::new()
        .with_inner_size(800, 600)
        .with_title("Hello, World!")
        .build();

    let demo = Box::new(HelloDemo::new());

    block_on(run(rogue, demo))
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

    fn present(&self, present_input: PresentInput) {
        for (i, e) in present_input.back_image.iter_mut().enumerate() {
            let x = (i as u32) % present_input.width;
            let y = (i as u32) / present_input.width;
            *e = if ((x ^ y) & 1) == 1 {
                0xff0000ffu32
            } else {
                0xffff00ffu32
            }
        }
        present_input
            .back_image
            .iter_mut()
            .for_each(|x| *x = rand::thread_rng().gen());
        present_input
            .fore_image
            .iter_mut()
            .for_each(|x| *x = rand::thread_rng().gen());
        present_input
            .text_image
            .iter_mut()
            .for_each(|x| *x = rand::thread_rng().gen::<u8>() as u32);
    }
}
