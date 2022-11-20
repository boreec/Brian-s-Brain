# Brian's brain (by Cyprien Borée)

This project is an implementation of the cellular automaton called 
[Brian's Brain](https://en.wikipedia.org/wiki/Brian%27s_Brain). It was made  with 
[Rust](https://en.wikipedia.org/wiki/Rust_(programming_language)) and 
[Vulkan](https://en.wikipedia.org/wiki/Vulkan)'s' graphics API.

# User manual

## Compilation

First of all, rust language has to be installed (see [here](https://www.rust-lang.org/tools/install)).

The program can be run in a GUI (a CLI version exists), in order to do so, Vulkan API has to be 
installed on the system (if `vkcube` test program can be executed, it's good).

Additionally, basic packages (`build-essential`, `cmake`) and other languages packages (`g++`, `python3`)
are required by dependencies for a complete compilation.

Finally, the program can be built with `cargo`.
```console
user:~$ cargo build --release 
```

## Execution

When it's compiled properly, the executable will be placed into `target/release/`.

There are two ways to execute it. The first one is by simply using its path:

```console
user:~$ ./target/release/brian-s-brain
```

The other way is to use `cargo`:

```console
user:-$ cargo run
```

### Arguments

Note that to provide arguments through `cargo run`, they have to be separated with `--`.

The executable can be provided with arguments (see them with `--help` or `-h`) :

```console
user:-$ cargo run -- --help
program to run the Brian's Brain cellular automaton

Usage: brian-s-brain [OPTIONS]

Options:
  -f, --framerate <FRAMERATE>  The number of time between two frames (in milliseconds) [default: 50]
  -i, --iter <ITER>            The number of iterations to run for [default: 100]
  -s, --size <SIZE>            The size of the world in which the cells live [default: 10]
  -g, --gui                    Run the program with a graphical user interface. This is the default mode if no other viewing modes is selected
  -c, --cli                    Run the program in the terminal. Note that if the cellular automaton's environment is too huge, render may fail
  -e, --example <EXAMPLE>      Run the program with a specific start. `---example=1` depicts 5 period-3 oscillators [default: 0]
  -h, --help                   Print help information
  -V, --version                Print version information
```

# Programmer manual

## Documentation

The documentation can be generated with `cargo`:

```console
user:~$ cargo doc
```

The generated files will be placed in `target/doc/brian_s_brain` and the entry point is accessed by `index.html`.

To view the file, `cargo` can use the default web browser on the system with:

```console
user:~$ cargo doc --open
```

## Unit Tests

I wrote unit tests to assert the good behaviour of the cellular automaton (represented by `WorldState` struct). 
`cargo` can run the tests as follows:

```console
user:~$ cargo test --release
```
