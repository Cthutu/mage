mod render;

use bytemuck::cast_slice;
use image::{EncodableLayout, GenericImageView, ImageFormat};
use render::*;
use std::{cmp::max, mem::replace, time::Duration};
use thiserror::Error;
use wgpu::SwapChainError;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

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
    pub vkey: Option<VirtualKeyCode>,
}

impl KeyState {
    pub fn alt_pressed(&self) -> bool {
        self.alt && !self.ctrl && !self.shift
    }
    pub fn ctrl_pressed(&self) -> bool {
        !self.alt && self.ctrl && !self.shift
    }
    pub fn shift_pressed(&self) -> bool {
        !self.alt && !self.ctrl && self.shift
    }
    pub fn key_pressed(&self, key: VirtualKeyCode) -> bool {
        if let Some(vkey) = self.vkey {
            if key == vkey {
                return true;
            }
        }
        false
    }
}

pub struct MouseState {
    pub on_screen: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub x: i32,
    pub y: i32,
}

pub struct SimInput<'a> {
    pub dt: Duration,
    pub width: u32,
    pub height: u32,
    pub key: &'a KeyState,
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

    let width = max(20, rogue.inner_size.0 as u32) / font_data.width * font_data.width;
    let height = max(20, rogue.inner_size.1 as u32) / font_data.height * font_data.height;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(width, height))
        .with_title(rogue.title)
        .with_min_inner_size(PhysicalSize::new(
            20 * font_data.width,
            20 * font_data.height,
        ))
        .build(&event_loop)?;
    let mut render = RenderState::new(&window, &font_data).await?;

    let mut key_state = KeyState {
        vkey: None,
        pressed: false,
        alt: false,
        ctrl: false,
        shift: false,
    };

    game.start();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        key_state.pressed = false;
        key_state.vkey = None;

        match event {
            //
            // Windowed Events
            //
            Event::WindowEvent { event, window_id } if window.id() == window_id => {
                match event {
                    //
                    // Closing the window
                    //
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                    //
                    // Keyboard Events
                    //
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode,
                                ..
                            },
                        ..
                    } => {
                        key_state.pressed = state == ElementState::Pressed;
                        key_state.vkey = virtual_keycode;

                        //
                        // Check for system keys
                        //
                        match key_state {
                            KeyState {
                                pressed: true,
                                vkey: Some(VirtualKeyCode::Escape),
                                ..
                            } => {
                                //
                                // Exit
                                //
                                *control_flow = ControlFlow::Exit;
                            }
                            KeyState {
                                pressed: true,
                                shift: false,
                                ctrl: false,
                                alt: true,
                                vkey: Some(VirtualKeyCode::Return),
                            } => {
                                //
                                // Toggle fullscreen
                                //
                                if window.fullscreen().is_some() {
                                    window.set_fullscreen(None);
                                } else if let Some(monitor) = window.current_monitor() {
                                    if let Some(video_mode) = monitor.video_modes().next() {
                                        if cfg!(any(target_os = "macos", unix)) {
                                            window.set_fullscreen(Some(Fullscreen::Borderless(
                                                Some(monitor),
                                            )));
                                        } else {
                                            window.set_fullscreen(Some(Fullscreen::Exclusive(
                                                video_mode,
                                            )));
                                        }
                                    };
                                };
                            }
                            _ => {}
                        }
                    }
                    //
                    // Modifier keys
                    //
                    WindowEvent::ModifiersChanged(mods) => {
                        key_state.alt = mods.alt();
                        key_state.ctrl = mods.ctrl();
                        key_state.shift = mods.shift();
                    }
                    //
                    // Resizing
                    //
                    WindowEvent::Resized(new_size) => render.resize(new_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        render.resize(*new_inner_size)
                    }

                    _ => {} // No more windowed events
                }
            }
            //
            // Idle
            //
            Event::MainEventsCleared => {
                if let TickResult::Stop = simulate(game.as_mut(), &render, &key_state) {
                    *control_flow = ControlFlow::Exit;
                }
                window.request_redraw();
            }
            //
            // Redraw
            //
            Event::RedrawRequested(_) => {
                present(game.as_ref(), &mut render);
                match render.render() {
                    Ok(_) => {}
                    Err(SwapChainError::Lost) => render.resize(window.inner_size()),
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                };
            }

            _ => {} // No more events
        }
    });
}

fn simulate(game: &mut dyn Game, render: &RenderState, key_state: &KeyState) -> TickResult {
    let (width, height) = render.chars_size();
    let sim_input = SimInput {
        dt: Duration::ZERO,
        width,
        height,
        key: key_state,
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
