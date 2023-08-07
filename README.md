# Rustphorus

Rustphorus is an implementation of the [Scratch](https://scratch.mit.edu/) virtual machine written in Rust.

What this means is that this is a program that runs Scratch projects (.sb3 files).

This is an very incomplete and inaccurate implementation.

# Development

```sh
git clone https://github.com/aspizu/rustphorus
cd rustphorus
RUST_LOG=info cargo run FILE_PATH.sb3
```

Running with `RUST_LOG=info` will print to stdout when a `say` block is executed.

# Devlog

Rust's borrow checker prevents Sprites from accessing each other's state while
iterating mutably over the Targets.

This makes it harder to implement the thing of thing block and potentially broadcasts.

SDL2 doesn't provide a function to render thick lines, so pen size has no effect.

Clones could be implemented by having a clones vector for each `Target` and
passing the original `TargetData` together with the clone's own `TargetState`
to the rendering and execution functions. Though implementing layering order with this
would require some thinking.

The order in which blocks are executed is slightly different from vanilla.
