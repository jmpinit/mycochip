extern crate prost_build;

use clap_complete::{generate_to, shells::Bash};
use std::env;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = build_cli();
    generate_to(
        Bash,
        &mut cmd, // We need to specify what generator to use
        "mycochip",  // We need to specify the bin name manually
        outdir,   // We need to specify where to write to
    )?;

    // Generate code
    prost_build::compile_protos(&["messages/request.proto"], &["messages/"])?;

    Ok(())
}
