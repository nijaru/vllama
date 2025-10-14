use anyhow::Result;
use tracing::info;

pub async fn execute(model: String) -> Result<()> {
    info!("Pulling model: {}", model);
    println!("Downloading {}...", model);
    println!("(Model download not yet implemented)");

    Ok(())
}
