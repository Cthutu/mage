//
// Presentation functionality
//
// Provides many functions to help render to md-mage's PresentIn structure.
//
// Copyright (C)2021 Matt Davies, all rights reserved.
//

use std::cmp::min;

//
// PresentInput
// This structure is passed to the Game trait's present() function
//

pub struct PresentInput<'a> {
    pub image: Image<'a>,
}

//
// Point
// An X, Y coordinate
//

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

//
// Char
// This represents a single ASCII character with an associated ink and paper colour.
//

#[derive(Debug, Clone, Copy)]
pub struct Char {
    pub ch: u8,
    pub ink: u32,
    pub paper: u32,
}

impl Char {
    pub fn new(ch: u8, ink: u32, paper: u32) -> Self {
        Char { ch, ink, paper }
    }
}

//
// RogueImage
// This represents a rectangular collection of RogueChars to render sprites and screens.
//

pub struct Image<'a> {
    pub width: u32,
    pub height: u32,
    pub fore_image: &'a mut Vec<u32>,
    pub back_image: &'a mut Vec<u32>,
    pub text_image: &'a mut Vec<u32>,
}

impl<'a> Image<'a> {
    pub fn coords_to_index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }

    pub fn clip(&self, p: Point, width: u32, height: u32) -> (u32, u32, u32, u32) {
        let mut x = p.x;
        let mut y = p.y;
        let mut width = width;
        let mut height = height;
        if x < 0 {
            width += x as u32;
            x = 0;
        }
        if y < 0 {
            height += y as u32;
            y = 0;
        }
        let x = x as u32;
        let y = y as u32;
        width = min(width, self.width - x);
        height = min(height, self.height - y);

        (x, y, width, height)
    }

    pub fn clear(&mut self, ink: u32, paper: u32) {
        self.draw_rect_filled(
            Point::new(0, 0),
            self.width,
            self.height,
            Char::new(b' ', ink, paper),
        );
    }

    pub fn draw_char(&mut self, p: Point, ch: Char) {
        if p.x >= 0 && p.y >= 0 {
            if let Some(i) = self.coords_to_index(p.x as u32, p.y as u32) {
                self.fore_image[i] = ch.ink;
                self.back_image[i] = ch.paper;
                self.text_image[i] = ch.ch as u32;
            }
        }
    }

    pub fn draw_string(&mut self, p: Point, text: &str, ink: u32, paper: u32) {
        let (x, y, w, _) = self.clip(p, text.len() as u32, 1);

        if let Some(i) = self.coords_to_index(x, y) {
            let w = w as usize;
            self.fore_image[i..i + w].iter_mut().for_each(|x| *x = ink);
            self.back_image[i..i + w]
                .iter_mut()
                .for_each(|x| *x = paper);
            self.text_image[i..i + w]
                .iter_mut()
                .enumerate()
                .for_each(|(j, x)| *x = (text.as_bytes()[j]) as u32);
        }
    }

    pub fn draw_rect(&mut self, p: Point, width: u32, height: u32, ch: Char) {
        if width < 3 || height < 3 {
            self.draw_rect_filled(p, width, height, ch);
        } else {
            // Draw top
            self.draw_rect_filled(p, width, 1, ch);
            // Draw bottom
            self.draw_rect_filled(Point::new(p.x, p.y + (height as i32) - 1), width, 1, ch);
            // Draw left
            self.draw_rect_filled(Point::new(p.x, p.y + 1), 1, height - 2, ch);
            // Draw right
            self.draw_rect_filled(
                Point::new(p.x + (width as i32) - 1, p.y + 1),
                1,
                height - 2,
                ch,
            );
        }
    }

    pub fn draw_rect_filled(&mut self, p: Point, width: u32, height: u32, ch: Char) {
        // Clip the coords and size to the image
        let (x, y, width, height) = self.clip(p, width, height);

        if let Some(mut i) = self.coords_to_index(x, y) {
            let width = width as usize;
            (0..height).for_each(|_| {
                // Render a row
                self.fore_image[i..i + width]
                    .iter_mut()
                    .for_each(|x| *x = ch.ink);
                self.back_image[i..i + width]
                    .iter_mut()
                    .for_each(|x| *x = ch.paper);
                self.text_image[i..i + width]
                    .iter_mut()
                    .for_each(|x| *x = ch.ch as u32);

                i += self.width as usize;
            });
        }
    }
}
