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
    #[arg(short, long, action, default_value_t = false)]
    gui: bool,

    /// Run the program in the terminal. Note that if the cellular
    /// automaton's environment is too huge, render may fail.
    #[arg(short, long, action, default_value_t = false)]
    cli: bool,
    
    /// Run the program with a specific start.
    ///
    /// - `--example=1` depicts 5 period-3 oscillators.
    /// - `--example=2` depicts gliders creating a breeder.
    /// - `--example=3` depicts a wick.
    #[arg(short, long, default_value_t = 0)]
    example: u16,
    
    /// Do 100 runs of the program and for each of them:
    ///
    /// 1. Declare the size of the cellular automaton to be 100x100 (`WorldState::new()`)
    /// 2. Initialize the world with 50% random noise (`WorldState::randomize()`)
    /// 3. Do 100 iterations (`WorldState::next()`)
    ///
    /// Then, the average execution time for each call is displayed.
    #[arg(short, long, action, default_value_t = false)]
    benchmark: bool,
}

/// Entry point of the program.
fn main() {
    let args = Args::parse();
    
    let ws = match args.example {
        0 => { 
            let mut w = WorldState::new(args.size);
            w.randomize(0.5);
            w
        }        
        1 => { WorldState::example1() }
        2 => { WorldState::example2() }
        3 => { WorldState::example3() }
        _ => { panic!("There is no example with that number!"); }
    };

    if args.gui || !args.cli {
        match run_gui(ws.clone(), args.framerate) {
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
        run_cli(ws.clone(), args.iter, args.framerate);
    }
}

/// Run the cellular automaton in the terminal.
fn run_cli(mut ws: WorldState, iteration: u16, framerate: u64){
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("{}", ws);
        thread::sleep(Duration::from_millis(framerate));
        for _ in 0..iteration {
            ws.next();
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!("{}", ws);
            thread::sleep(Duration::from_millis(framerate));
        }
}
