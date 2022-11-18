use anyhow::Result;
use static_files::NpmBuild;
use std::{env, path::Path};

const FRONTEND_PROJECT_DIR: &str = "frontend";
const FRONTEND_OUTPUT_DIR: &str = "dist";
const FRONTEND_BUILD_COMMAND: &str = "build";
const FRONTEND_GENERATED_FILENAME: &str = "frontend.rs";
const FRONTEND_GENERATED_FN: &str = "frontend";
const NPM_EXECUTABLE: &str = "pnpm";

fn main() -> Result<()> {
    let output_dir = env::var("OUT_DIR")?;
    let output_dir = Path::new(&output_dir);
    let frontend_dir = Path::new(FRONTEND_PROJECT_DIR);
    let mut resource_dir = NpmBuild::new(frontend_dir)
        .change_detection()
        .executable(NPM_EXECUTABLE)
        .target(frontend_dir.join(FRONTEND_OUTPUT_DIR))
        .install()?
        .run(FRONTEND_BUILD_COMMAND)?
        .to_resource_dir();
    resource_dir
        .with_generated_filename(output_dir.join(FRONTEND_GENERATED_FILENAME))
        .with_generated_fn(FRONTEND_GENERATED_FN);
    resource_dir.build()?;

    Ok(())
}
