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

There's two ways to execute it. The first one is by simply using its path:

```console
user:~$ ./target/release/brian-s-brain
```

The other way is to use `cargo`:

```console
user:-$ cargo run
```

## Documentation

## Unit Tests