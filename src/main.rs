mod chip;

use cursive::CbSink;

use std::env;

use crossbeam_channel::{bounded, Receiver, Sender};
use std::time::Duration;

use crate::chip::{chip8::Chip8, chip8::Display, Chip, ChipWithDisplayOutput};

enum KeyEvent {
    Key(u8),
    SpeedUp,
    SlowDown,
    Quit,
}

#[derive(Clone)]
struct ExecLoopIoChannels {
    gfx_sink: CbSink,
    key_drain: Receiver<KeyEvent>,
    shutdown_sink: Sender<()>,
}

fn execute(mut chip8: Chip8, io_channels: ExecLoopIoChannels) {
    let mut cycle_sleep = 5;
    loop {
        match io_channels.key_drain.recv_timeout(Duration::from_millis(0)) {
            Ok(KeyEvent::Key(key)) => {
                chip8.set_input_pin(key, true);
            }
            Ok(KeyEvent::Quit) => {
                io_channels
                    .shutdown_sink
                    .send(())
                    .expect("Failed to orderly shutdown.");
                return;
            }
            Ok(KeyEvent::SpeedUp) => {
                if cycle_sleep > 5 {
                    cycle_sleep -= 5;
                }
            }
            Ok(KeyEvent::SlowDown) => {
                cycle_sleep += 5;
            }
            Err(_) => {}
        };

        chip8 = chip8.cycle();

        chip8.update_ui(&io_channels.gfx_sink);

        std::thread::sleep(Duration::from_millis(cycle_sleep));
    }
}

fn load_program_from_args(chip8: &mut Chip8) {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            chip8.load_program_bytes(&[
                0x63, 0, 0x64, 0, 0xF0, 0x0A, 0xF0, 0x29, 0xD3, 0x45, 0x12, 0x02,
            ]);
        }
        _ => {
            chip8
                .load_program(&args[1])
                .expect("Could not load program.");
        }
    };
}

fn main() {
    let mut chip8 = Chip8::new();

    load_program_from_args(&mut chip8);

    let mut siv = cursive::default();

    let cb_sink = siv.cb_sink().clone();
    let (key_sender, key_receiver) = bounded::<KeyEvent>(10);
    let (shutdown_sender, shutdown_receiver) = bounded::<()>(1);

    std::thread::spawn(move || {
        execute(
            chip8,
            ExecLoopIoChannels {
                gfx_sink: cb_sink,
                key_drain: key_receiver,
                shutdown_sink: shutdown_sender,
            },
        );
    });

    let sender = key_sender.clone();
    siv.add_global_callback(cursive::event::Key::Esc, move |s| {
        sender.send(KeyEvent::Quit).unwrap();
        shutdown_receiver.recv().expect("Orderly shutdown failed");
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
        let sender = key_sender.clone();
        siv.add_global_callback(*i, move |_s| {
            sender.send(KeyEvent::Key(*j as u8)).unwrap();
        });
    }

    let sender = key_sender.clone();
    siv.add_global_callback(cursive::event::Key::Up, move |_s| {
        sender.send(KeyEvent::SpeedUp).unwrap();
    });

    let sender = key_sender.clone();
    siv.add_global_callback(cursive::event::Key::Down, move |_s| {
        sender.send(KeyEvent::SlowDown).unwrap();
    });

    siv.add_layer(Display::default());

    siv.run();
}
