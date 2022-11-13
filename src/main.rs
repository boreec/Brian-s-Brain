use crate::graphics::run_gui;
use crate::world_state::WorldState;

use clap::Parser;

use std::time::Duration;
use std::thread;

/// Module containing vulkan initialization and
/// window handling.
mod graphics;

/// Module containing the cellular automaton
/// (cells, environment, rules, etc.).
mod world_state;

/// Program to run the Brian's Brain cellular automaton.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    
    #[arg(short, long, default_value_t = 50)]
    /// The number of time between two frames (in milliseconds).
    framerate: u64,
    
    /// The number of iterations to run for.
    #[arg(short, long, default_value_t = 100)]
    iter: u16,
    
    /// The size of the world in which the cells live.    
    #[arg(short, long, default_value_t = 10)]
    size: u16,
    
    /// Run the program with a graphical user interface.
    /// This is the default mode if no other viewing modes is selected.
    /// It can be used alongside with `cli`.
    #[arg(short, long, action, default_value_t = false)]
    gui: bool,

    /// Run the program in the terminal. Note that if the cellular
    /// automaton's environment is too huge, render may not be done
    /// properly. It can be used alongside with `gui` argument.
    #[arg(short, long, action, default_value_t = false)]
    cli: bool,
}

fn main() {
    let args = Args::parse();
    
    let mut ws = WorldState::new(args.size);
    ws.randomize(0.5);

    if args.gui || (!args.gui && !args.cli){
        match run_gui(&mut ws, args.framerate) {
            Ok(()) => {}
            Err(e) => {
                panic!(
                    "Can't run the program with a graphical
                    interface because of the following error.
                    Try to run it in the terminal with --cli.
                    \n{}", e);
            }
        }
    }
    
    if args.cli {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("{}", ws);
        thread::sleep(Duration::from_millis(args.framerate));
        for _ in 0..args.iter {
            ws.next();
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!("{}", ws);
            thread::sleep(Duration::from_millis(args.framerate));
        }
    }
}
