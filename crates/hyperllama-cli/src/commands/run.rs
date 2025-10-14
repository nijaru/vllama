use anyhow::Result;
use tracing::info;

pub async fn execute(model: String, prompt: Option<String>) -> Result<()> {
    info!("Running model: {}", model);

    if let Some(prompt_text) = prompt {
        println!("Sending prompt: {}", prompt_text);
    } else {
        println!("Interactive chat with {}", model);
        println!("(Interactive mode not yet implemented)");
    }

    Ok(())
}
