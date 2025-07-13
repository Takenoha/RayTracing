use std::error::Error;

use raytracing_cli::cli;

fn main() -> Result<(), Box<dyn Error>> {
    cli()
}
