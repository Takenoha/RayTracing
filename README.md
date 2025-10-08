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

To render a scene and generate an image, use the `render` subcommand.

**Example:**
```bash
cargo run --release -- render --config simulation.toml --output image.png
```

You can also override rendering parameters:
```bash
cargo run --release -- render --width 1920 --height 1080 --samples 500
```

**Options:**
```text
Render a scene to an image file

Usage: raytracing.exe render [OPTIONS]

Options:
  -c, --config <FILE>      Path to the simulation configuration file [default: simulation.toml]
  -o, --output <FILE>      Path to the output image file [default: output.png]
      --width <WIDTH>      Override image width
      --height <HEIGHT>    Override image height
      --samples <SAMPLES>  Override samples per pixel
      --bounces <BOUNCES>  Override max bounces
  -h, --help               Print help
```

### Trace Ray Paths

To simulate ray paths and export them as a `.obj` file for visualization, use the `trace` subcommand.

**Example:**
```bash
cargo run --release -- trace --config simulation.toml --output paths.obj
```

**Options:**
```text
Trace rays based on a simulation configuration and output an OBJ file

Usage: raytracing.exe trace [OPTIONS]

Options:
  -c, --config <FILE>  Path to the simulation configuration file [default: simulation.toml]
  -o, --output <FILE>  Path to the output OBJ file [default: paths.obj]
  -h, --help           Print help
```