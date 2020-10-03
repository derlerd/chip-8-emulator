use crate::chip::{chip8::Chip8, Chip, ChipWithCursiveDisplay};

use cursive::{
    direction::Direction,
    event::{Event, EventResult},
    theme::{BaseColor, Color, ColorStyle},
    view::View,
    CbSink, Printer, Vec2,
};

/// Represents the display of the Chip 8
pub struct Display {
    pixels: [bool; 64 * 32],
}

impl Display {
    /// Creates a new display from a slice.
    pub fn new(pixels: &[bool]) -> Self {
        assert_eq!(pixels.len(), 64 * 32);
        let mut tmp = [false; 64 * 32];
        tmp.copy_from_slice(&pixels[..]);
        Display { pixels: tmp }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new(&[false; 64 * 32])
    }
}

/// Implements cursive::view::View for Display to enable drawing it
/// as a View out of the box.
impl View for Display {
    fn draw(&self, printer: &Printer) {
        printer.with_color(
            ColorStyle::new(Color::Dark(BaseColor::Black), Color::RgbLowRes(0, 0, 0)),
            |printer| {
                for x in 0..64 {
                    for y in 0..32 {
                        if self.pixels[x + 64 * y] {
                            printer.print((x, y), " ");
                        }
                    }
                }
            },
        );
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }

    fn on_event(&mut self, _event: Event) -> EventResult {
        EventResult::Ignored
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        Vec2 { x: 64, y: 32 }
    }
}

impl ChipWithCursiveDisplay for Chip8 {
    fn update_ui(&mut self, gfx_sink: &CbSink) {
        fn get_display(chip: &Chip8) -> Display {
            Display::new(chip.read_output_pins())
        }

        if !self.draw {
            return;
        }
        let display = get_display(&self);
        gfx_sink
            .send(Box::new(Box::new(move |s: &mut cursive::Cursive| {
                s.pop_layer();
                s.add_layer(display);
            })))
            .expect("Sending updated display failed");
        self.draw = false;
    }
}
