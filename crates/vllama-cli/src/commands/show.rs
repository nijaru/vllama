use anyhow::Result;
use tracing::info;

pub async fn execute(model: String, _modelfile: bool, _parameters: bool) -> Result<()> {
    info!("Showing info for model: {}", model);
    println!("Model: {}", model);
    println!("(Model info not yet implemented)");

    Ok(())
}
