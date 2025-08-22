//! Universal traceability demo binary

use provchain_org::universal_demo::run_universal_traceability_demo;
use anyhow::Result;

fn main() -> Result<()> {
    println!("Starting Universal Traceability Platform Demo...\\n");
    
    run_universal_traceability_demo()?;
    
    println!("\\nDemo completed successfully!");
    Ok(())
}
