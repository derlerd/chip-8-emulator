mod chip;

use cursive::{
    direction::Direction,
    event::{Event, EventResult},
    theme::{BaseColor, Color, ColorStyle},
    CbSink, Printer, Vec2,
};

use crossbeam_channel::{bounded, Receiver, Sender};
use std::time::Duration;

use std::fs;
use std::fs::File;
use std::io::Read;

use crate::chip::{chip8::Chip8, Chip};

struct Display {
    pixels: [bool; 64 * 32],
}

impl Display {
    fn new(pixels: [bool; 64 * 32]) -> Self {
        Display { pixels }
    }
}

impl cursive::view::View for Display {
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

fn load_program(chip8: &mut Chip8, program: &[u8]) {
    for i in 0..program.len() {
        chip8.set_memory_byte(program[i], 0x200 + i);
    }
}

fn execute(mut chip8: Chip8, io_channels: IoChannels) {
    loop {
        match io_channels.key_drain.recv_timeout(Duration::from_millis(1)) {
            Ok(KeyEvent::Key(key)) => {
                chip8.set_io_pin(key, true);
            }
            Ok(KeyEvent::Quit) => {
                io_channels
                    .shutdown_sink
                    .send(())
                    .expect("Failed to orderly shutdown.");
                return;
            }
            Err(_) => {}
        };
        chip8 = chip8.cycle();
        chip8.reset_io_pins();
        let display = chip8.get_gfx();
        io_channels
            .gfx_sink
            .send(Box::new(Box::new(move |s: &mut cursive::Cursive| {
                s.pop_layer();
                s.add_layer(Display::new(display));
            })))
            .expect("Sending updated display failed");
    }
}

#[derive(Clone)]
struct IoChannels {
    gfx_sink: CbSink,
    key_drain: Receiver<KeyEvent>,
    shutdown_sink: Sender<()>,
}

enum KeyEvent {
    Key(u8),
    Quit,
}

fn main() {
    let mut chip8 = Chip8::new();

    /*
        let filename = "space-invaders.ch8";
        let mut file = File::open(&filename).unwrap();
        let md = fs::metadata(&filename).unwrap();
        let mut buffer = vec![0; md.len() as usize];
        file.read(&mut buffer).unwrap();
    */
    load_program(
        &mut chip8,
        &[0xF0, 0x0A, 0xF0, 0x29, 0xD3, 0x35, 0x12, 0x00],
    );

    let (key_sender, key_receiver) = bounded::<KeyEvent>(10);
    let (shutdown_sender, shutdown_receiver) = bounded::<()>(1);

    let mut siv = cursive::default();

    let cb_sink = siv.cb_sink().clone();

    std::thread::spawn(move || {
        execute(
            chip8,
            IoChannels {
                gfx_sink: cb_sink,
                key_drain: key_receiver,
                shutdown_sink: shutdown_sender,
            },
        );
    });

    let quit_sender = key_sender.clone();
    siv.add_global_callback(cursive::event::Key::Esc, move |s| {
        quit_sender.send(KeyEvent::Quit).unwrap();
        shutdown_receiver.recv();
        s.quit();
    });

    for (i, j) in &[
        ('1', 0x1),
        ('2', 0x2),
        ('3', 0x3),
        ('4', 0xC),
        ('q', 0x4),
        ('w', 0x5),
        ('e', 0x6),
        ('r', 0xD),
        ('a', 0x7),
        ('s', 0x8),
        ('d', 0x9),
        ('f', 0xE),
        ('z', 0xA),
        ('x', 0x0),
        ('c', 0xB),
        ('v', 0xF),
    ] {
        let left_sender = key_sender.clone();
        siv.add_global_callback(*i, move |_s| {
            left_sender.clone().send(KeyEvent::Key(*j as u8)).unwrap();
        });
    }

    siv.add_layer(Display::new([true; 64 * 32]));

    siv.run();
}
