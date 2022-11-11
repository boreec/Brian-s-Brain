use crate::graphics::init_vulkan;
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
    
    if args.gui || (!args.gui && !args.cli){
        init_vulkan().unwrap();
    }
    if args.cli {
        ws.randomize(0.5);
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("{}", ws);
        thread::sleep(Duration::from_millis(1000));
        for i in 0..args.iter {
            ws.next();
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!("{}", ws);
            thread::sleep(Duration::from_millis(1000));
        }
    }
}


