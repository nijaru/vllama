use anyhow::Result;
use tracing::info;

pub async fn execute(model: String) -> Result<()> {
    info!("Removing model: {}", model);
    println!("Removing {}...", model);
    println!("(Model removal not yet implemented)");

    Ok(())
}
