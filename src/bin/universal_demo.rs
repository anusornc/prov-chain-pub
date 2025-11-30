//! Universal traceability demo binary

use anyhow::Result;
use provchain_org::universal_demo::run_universal_traceability_demo;

fn main() -> Result<()> {
    println!("Starting Universal Traceability Platform Demo...\\n");

    run_universal_traceability_demo()?;

    println!("\\nDemo completed successfully!");
    Ok(())
}
