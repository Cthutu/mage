#![allow(dead_code, unused_imports, unused_variables)]

mod render;

use bytemuck::cast_slice;
use image::{EncodableLayout, GenericImageView, ImageFormat, RgbaImage};
use render::*;
use std::{mem::replace, time::Duration};
use thiserror::Error;
use wgpu::SwapChainError;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    monitor::{MonitorHandle, VideoMode},
    window::{Fullscreen, WindowBuilder},
};
use winit_fullscreen::WindowFullScreen;
use winit_input_helper::WinitInputHelper;

pub trait Game {
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
    pub width: u32,
    pub height: u32,
    pub key: Option<KeyState>,
    pub mouse: Option<MouseState>,
}

pub struct PresentInput<'a> {
    pub width: u32,
    pub height: u32,
    pub fore_image: &'a mut Vec<u32>,
    pub back_image: &'a mut Vec<u32>,
    pub text_image: &'a mut Vec<u32>,
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

    #[error(transparent)]
    RenderError(#[from] render::RenderError),

    #[error("Unable to read font data")]
    BadFont,
}

pub type RogueResult<T> = Result<T, RogueError>;

//
// Rogue building
//

pub struct RogueBuilder {
    inner_size: (usize, usize),
    title: String,
    font: RogueFont,
}

pub struct RogueGame {
    builder: RogueBuilder,
}

pub struct RogueFontData {
    data: Vec<u32>,
    width: u32,
    height: u32,
}

enum RogueFont {
    Default,
    Custom(RogueFontData),
}

impl RogueBuilder {
    pub fn new() -> Self {
        RogueBuilder {
            inner_size: (100, 100),
            title: "md-rogue window".to_string(),
            font: RogueFont::Default,
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

    pub fn with_font(&mut self, font: RogueFontData) -> &mut Self {
        self.font = RogueFont::Custom(font);
        self
    }

    pub fn build(&mut self) -> Self {
        RogueBuilder {
            inner_size: self.inner_size,
            title: self.title.clone(),
            font: replace(&mut self.font, RogueFont::Default),
        }
    }
}

impl Default for RogueBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn load_font_image(data: &[u8], format: ImageFormat) -> RogueResult<RogueFontData> {
    let font_image =
        image::load_from_memory_with_format(data, format).map_err(|_| RogueError::BadFont)?;
    let dimensions = font_image.dimensions();
    let font_rgba = font_image.to_rgba8();
    let font_data = font_rgba.as_bytes();
    let data_u32: &[u32] = cast_slice(font_data);
    let char_width = dimensions.0 / 16;
    let char_height = dimensions.1 / 16;
    if char_width == 0 || char_height == 0 {
        return Err(RogueError::BadFont);
    }

    Ok(RogueFontData {
        width: char_width,
        height: char_height,
        data: Vec::from(data_u32),
    })
}

pub async fn run(rogue: RogueBuilder, mut game: Box<dyn Game>) -> RogueResult<()> {
    let font_data = match rogue.font {
        RogueFont::Default => load_font_image(include_bytes!("font1.png"), ImageFormat::Png)?,
        RogueFont::Custom(font) => font,
    };

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(
            rogue.inner_size.0 as u32,
            rogue.inner_size.1 as u32,
        ))
        .with_title(rogue.title)
        .build(&event_loop)?;
    let mut input = WinitInputHelper::new();
    let mut render = RenderState::new(&window, &font_data).await?;

    game.start();

    // Give the game one chance to simulate and present so we have something to show on the first frame.
    if let TickResult::Stop = simulate(game.as_mut(), &render) {
        return Ok(());
    }
    present(game.as_ref(), &mut render);

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Wait;

        if input.update(&event) {
            //
            // Input has occurred
            //
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
            } else if input.key_pressed(VirtualKeyCode::Return) && input.held_alt() {
                // Fullscreen toggle
                window.toggle_fullscreen();
            }

            if let Some(size) = input.window_resized() {
                render.resize(size);
            }

            match render.render() {
                Ok(_) => {}
                Err(SwapChainError::Lost) => render.resize(window.inner_size()),
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            };
        } else {
            if let TickResult::Stop = simulate(game.as_mut(), &render) {
                *control_flow = ControlFlow::Exit
            } else {
                present(game.as_ref(), &mut render);
                window.request_redraw();
            }
        }
    });
}

fn simulate(game: &mut dyn Game, render: &RenderState) -> TickResult {
    let (width, height) = render.chars_size();
    let sim_input = SimInput {
        dt: Duration::ZERO,
        width,
        height,
        key: None,
        mouse: None,
    };

    game.tick(sim_input)
}

fn present(game: &dyn Game, render: &mut RenderState) {
    let (width, height) = render.chars_size();
    let (fore_image, back_image, text_image) = render.images();

    let present_input = PresentInput {
        width,
        height,
        fore_image,
        back_image,
        text_image,
    };

    game.present(present_input);
}
