use crossbeam_channel::{bounded, Receiver, Sender};
use cursive::CbSink;
use std::env;
use std::time::Duration;

use chip_8_emulator::chip::{
    chip8::cursive_display::Display, chip8::Chip8, Chip, ChipWithCursiveDisplay, LoadProgramError,
};

/// Error type for errors that occur during parsing the command line arguments
/// and loading the program based on the arguments.
enum Error {
    InvalidUsage(String),
    InvalidProgram(LoadProgramError),
}

/// Represents an event to be processed by the event loop. It is generic
/// over the type representing the pressed key.
enum Event<T> {
    /// Occurs when the key passed in the enum value was pressed.
    Key(T),

    /// Indicates that all keys are released. Note that this is a
    /// hack because OS X currently requires extra permissions to
    /// listen to key down/up events. To get around this we simply
    /// read the stdin (indirectly via registering for cursive
    /// events) and assign one key to trigger releasing all keys.
    KeyRelease,

    /// Decreases the sleep time after each cycle.
    SpeedUp,

    /// Increases the sleep time after each cycle.
    SlowDown,

    /// Shut down.
    Quit,
}

/// Represents the channels available to the event loop. It is generic
/// over the type representing the pressed keys.
#[derive(Clone)]
struct EventLoopChannels<T> {
    /// The channel to send the UI refresh messages to.
    gfx_sender: CbSink,

    /// The channel on which the Events are received.
    key_receiver: Receiver<Event<T>>,

    /// A channel to report that the thread has completed
    /// shutdown.
    shutdown_sender: Sender<()>,
}

/// The event loop. Constantly loops over (1) process event if there
/// is any. (2) Invoke cycle on the chip. (3) Update the UI. (4) Sleep
/// for the cycle sleep time (initially 1ms). (5) Start over.
fn event_loop<T, P, M>(mut chip: T, io_channels: EventLoopChannels<P>)
where
    T: Chip<PinAddress = P, MemoryAddress = M> + ChipWithCursiveDisplay,
{
    let mut cycle_sleep = 1;
    loop {
        match io_channels.key_receiver.try_recv() {
            Ok(Event::Key(key)) => {
                chip.set_input_pin(key, true);
            }
            Ok(Event::KeyRelease) => {
                chip.reset_input_pins();
            }
            Ok(Event::Quit) => {
                io_channels
                    .shutdown_sender
                    .send(())
                    .expect("Failed to orderly shutdown.");
                return;
            }
            Ok(Event::SpeedUp) => {
                if cycle_sleep > 5 {
                    cycle_sleep -= 5;
                }
            }
            Ok(Event::SlowDown) => {
                cycle_sleep += 5;
            }
            Err(_) => { /* do nothing */ }
        };

        chip.cycle();
        chip.update_ui(&io_channels.gfx_sender);

        std::thread::sleep(Duration::from_millis(cycle_sleep));
    }
}

/// Loads a program based on the given arguments. If there are no arguments, it
/// loads a simple default program, whereas it interprets the first argument as
/// path to the program to load and attempts to load the program from there.
fn load_program_from_args(chip8: &mut Chip8) -> Result<usize, Error> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => Err(Error::InvalidUsage(
            "Expecting path to the program to load as command line argument.".to_string(),
        )),
        _ => chip8.load_program(&args[1]).map_err(Error::InvalidProgram),
    }
}

/// Constructs the UI and spawns the event loop and the UI thread.
fn main() {
    let mut chip8 = Chip8::new();

    if let Err(e) = load_program_from_args(&mut chip8) {
        println!("{}", e);
        return;
    }

    let mut siv = cursive::default();

    let cb_sink = siv.cb_sink().clone();
    let (key_sender, key_receiver) = bounded::<Event<u8>>(10);
    let (shutdown_sender, shutdown_receiver) = bounded::<()>(1);

    std::thread::spawn(move || {
        event_loop(
            chip8,
            EventLoopChannels {
                gfx_sender: cb_sink,
                key_receiver: key_receiver,
                shutdown_sender: shutdown_sender,
            },
        );
    });

    let sender = key_sender.clone();
    siv.add_global_callback(cursive::event::Key::Esc, move |s| {
        sender.send(Event::Quit).unwrap();
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
            sender.send(Event::Key(*j as u8)).unwrap();
        });
    }

    let sender = key_sender.clone();
    siv.add_global_callback(' ', move |_s| {
        sender.send(Event::KeyRelease).unwrap();
    });

    let sender = key_sender.clone();
    siv.add_global_callback(cursive::event::Key::Up, move |_s| {
        sender.send(Event::SpeedUp).unwrap();
    });

    let sender = key_sender.clone();
    siv.add_global_callback(cursive::event::Key::Down, move |_s| {
        sender.send(Event::SlowDown).unwrap();
    });

    siv.add_layer(Display::default());

    siv.run();
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidUsage(message) => write!(f, "Usage: {}", message),
            Error::InvalidProgram(error) => write!(f, "{}", error),
        }
    }
}
