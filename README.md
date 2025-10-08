# Ray Tracing in Rust

This is a simple ray tracing engine implemented in Rust.

## Build

To build the project, run the following command:

```bash
cargo build --release
```

## Usage

The executable provides two main subcommands: `render` and `trace`.

### Render a Scene

To render a scene and generate an image, use the `render` subcommand. You need to provide a simulation configuration file.

```bash
cargo run --release -- render --config simulation.toml --output image.png
```

You can also override rendering parameters:

```bash
cargo run --release -- render --width 1920 --height 1080 --samples 500
```

### Trace Ray Paths

To simulate ray paths and export them as a `.obj` file for visualization, use the `trace` subcommand.

```bash
cargo run --release -- trace --config simulation.toml --output paths.obj
```
