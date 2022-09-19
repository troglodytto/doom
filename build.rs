use anyhow::Result;
use std::{path::Path, process::Command};
fn main() -> Result<()> {
    let out_dir = if cfg!(debug_assertions) {
        "target/debug"
    } else {
        "target/release"
    };

    let path = Path::new(&out_dir);
    std::fs::create_dir_all(path.join("shaders"))?;

    println!("cargo:rerun-if-changed=src/shader.vert");
    Command::new("glslc")
        .args([
            "src/shader.vert",
            "-o",
            path.join("shaders/vert.spv")
                .to_str()
                .unwrap_or_else(|| "vert.spv"),
        ])
        .output()?;

    println!("cargo:rerun-if-changed=src/shader.frag");
    Command::new("glslc")
        .args([
            "src/shader.frag",
            "-o",
            path.join("shaders/frag.spv")
                .to_str()
                .unwrap_or_else(|| "frag.spv"),
        ])
        .output()?;

    Ok(())
}
