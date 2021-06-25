//
// Dungeon generation support
//

#![allow(unused_imports, unused_variables, unused_mut)]

#[cfg(feature = "dungeon-generation")]
pub mod generation {

    use crate::{new_colour, present::*, Colour};
    use md_dungeon::{Direction, Element, Map};

    pub fn gen_image(map: &Map) -> Image {
        let mut image = Image::new(map.width, map.height);
        map.map
            .iter()
            .zip(image.fore_image.iter_mut())
            .for_each(|(m, e)| {
                *e = match m.elem {
                    Element::Empty => Colour::Black.into(),
                    Element::Floor | Element::Door(_) => new_colour(64, 64, 64),
                    Element::Wall => Colour::Black.into(),
                };
            });
        map.map
            .iter()
            .zip(image.back_image.iter_mut())
            .for_each(|(m, e)| {
                *e = match m.elem {
                    Element::Wall => new_colour(64, 64, 0),
                    _ => Colour::Black.into(),
                };
            });
        map.map
            .iter()
            .zip(image.text_image.iter_mut())
            .for_each(|(m, e)| {
                *e = match m.elem {
                    Element::Empty => b' ',
                    Element::Floor => b'.',
                    Element::Door(_) => b'+',
                    Element::Wall => b'#',
                } as u32;
            });

        image
    }
}
