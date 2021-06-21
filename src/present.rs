//
// Presentation functionality
//
// Provides many functions to help render to md-mage's PresentIn structure.
//
// Copyright (C)2021 Matt Davies, all rights reserved.
//

use std::cmp::min;

pub struct Image<'a> {
    pub width: u32,
    pub height: u32,
    pub fore_image: &'a mut Vec<u32>,
    pub back_image: &'a mut Vec<u32>,
    pub text_image: &'a mut Vec<u32>,
}

pub struct PresentInput<'a> {
    pub image: Image<'a>,
}

impl<'a> Image<'a> {
    pub fn coords_to_index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }

    pub fn cls(&mut self, ink: u32, paper: u32) {
        self.draw_rect_filled(0, 0, self.width, self.height, b' ', ink, paper);
    }

    pub fn draw_char(&mut self, x: u32, y: u32, ch: u8, ink: u32, paper: u32) {
        if let Some(i) = self.coords_to_index(x, y) {
            self.fore_image[i] = ink;
            self.back_image[i] = paper;
            self.text_image[i] = ch as u32;
        }
    }

    pub fn draw_string(&mut self, x: u32, y: u32, text: &str, ink: u32, paper: u32) {
        if let Some(i) = self.coords_to_index(x, y) {
            let actual_len = min(text.len(), (self.width - x) as usize);

            self.fore_image[i..i + actual_len]
                .iter_mut()
                .for_each(|x| *x = ink);
            self.back_image[i..i + actual_len]
                .iter_mut()
                .for_each(|x| *x = paper);
            self.text_image[i..i + actual_len]
                .iter_mut()
                .enumerate()
                .for_each(|(j, x)| *x = (text.as_bytes()[j]) as u32);
        }
    }

    pub fn draw_rect(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        ch: u8,
        ink: u32,
        paper: u32,
    ) {
        if width < 3 || height < 3 {
            self.draw_rect_filled(x, y, width, height, ch, ink, paper);
        } else {
            // Draw top
            self.draw_rect_filled(x, y, width, 1, ch, ink, paper);
            // Draw bottom
            self.draw_rect_filled(x, y + height - 1, width, 1, ch, ink, paper);
            // Draw left
            self.draw_rect_filled(x, y + 1, 1, height - 2, ch, ink, paper);
            // Draw right
            self.draw_rect_filled(x + width - 1, y + 1, 1, height - 2, ch, ink, paper);
        }
    }

    pub fn draw_rect_filled(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        ch: u8,
        ink: u32,
        paper: u32,
    ) {
        if let Some(mut i) = self.coords_to_index(x, y) {
            let actual_width = min(width, self.width - x) as usize;
            let actual_height = min(height, self.height - y) as usize;

            [0..actual_height].iter().for_each(|_| {
                // Render a row
                self.fore_image[i..i + actual_width]
                    .iter_mut()
                    .for_each(|x| *x = ink);
                self.back_image[i..i + actual_width]
                    .iter_mut()
                    .for_each(|x| *x = paper);
                self.text_image[i..i + actual_width]
                    .iter_mut()
                    .for_each(|x| *x = ch as u32);

                i += self.width as usize;
            });
        }
    }
}
