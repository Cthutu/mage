#![allow(dead_code, unused_imports)]

use std::time::Duration;
use thiserror::Error;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

pub trait App {
    fn start(&mut self);
    fn tick(&mut self, sim_input: SimInput) -> TickResult;
    fn present(&self, present_input: PresentInput);
}

pub enum TickResult {
    Continue,
    Stop,
}

pub struct KeyState {
    pub pressed: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub vkey: VirtualKeyCode,
}

pub struct MouseState {
    pub on_screen: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub x: i32,
    pub y: i32,
}

pub struct SimInput {
    pub dt: Duration,
    pub width: usize,
    pub height: usize,
    pub key: KeyState,
    pub mouse: MouseState,
}

pub struct PresentInput<'a> {
    pub width: usize,
    pub height: usize,
    pub fore_image: &'a Vec<u32>,
    pub back_image: &'a Vec<u32>,
    pub text_image: &'a Vec<u32>,
}

pub fn new_colour(r: u8, g: u8, b: u8) -> u32 {
    0xff000000u32 + ((b as u32) << 16) + ((g as u32) << 8) + (r as u32)
}

pub enum Colour {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl From<Colour> for u32 {
    fn from(c: Colour) -> Self {
        match c {
            Colour::Black => new_colour(0, 0, 0),
            Colour::Red => new_colour(255, 0, 0),
            Colour::Green => new_colour(0, 255, 0),
            Colour::Yellow => new_colour(255, 255, 0),
            Colour::Blue => new_colour(0, 0, 255),
            Colour::Magenta => new_colour(255, 0, 255),
            Colour::Cyan => new_colour(0, 255, 255),
            Colour::White => new_colour(255, 255, 255),
        }
    }
}

//
// Errors
//

#[derive(Error, Debug)]
pub enum RogueError {
    #[error(transparent)]
    OSError(#[from] winit::error::OsError),
}

pub type RogueResult<T> = Result<T, RogueError>;

//
// Rogue building
//

pub struct RogueBuilder {
    inner_size: (usize, usize),
    title: String,
}

pub struct RogueGame {
    builder: RogueBuilder,
}

impl RogueBuilder {
    pub fn new() -> Self {
        RogueBuilder {
            inner_size: (100, 100),
            title: "md-rogue window".to_string(),
        }
    }

    pub fn with_inner_size(&mut self, width: usize, height: usize) -> &mut Self {
        self.inner_size = (width, height);
        self
    }

    pub fn with_title(&mut self, title: &str) -> &mut Self {
        self.title = String::from(title);
        self
    }

    pub fn build(&self) -> Self {
        RogueBuilder {
            inner_size: self.inner_size,
            title: self.title.clone(),
        }
    }
}

impl Default for RogueBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run(rogue: RogueBuilder, mut app: impl App) -> RogueResult<()> {
    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(
            rogue.inner_size.0 as u32,
            rogue.inner_size.1 as u32,
        ))
        .with_title(rogue.title)
        .build(&event_loop)?;
    let mut input = WinitInputHelper::new();

    app.start();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
            } else {
            }
        }
    });
}
