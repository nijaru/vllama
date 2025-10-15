use crate::{ChatMessage, ChatRole};

pub trait ChatTemplate {
    fn apply(&self, messages: &[ChatMessage]) -> String;
}

pub struct Llama3Template;

impl ChatTemplate for Llama3Template {
    fn apply(&self, messages: &[ChatMessage]) -> String {
        let mut result = String::from("<|begin_of_text|>");

        for msg in messages {
            match msg.role {
                ChatRole::System => {
                    result.push_str("<|start_header_id|>system<|end_header_id|>\n\n");
                    result.push_str(&msg.content);
                    result.push_str("<|eot_id|>");
                }
                ChatRole::User => {
                    result.push_str("<|start_header_id|>user<|end_header_id|>\n\n");
                    result.push_str(&msg.content);
                    result.push_str("<|eot_id|>");
                }
                ChatRole::Assistant => {
                    result.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");
                    result.push_str(&msg.content);
                    result.push_str("<|eot_id|>");
                }
                ChatRole::Tool => {
                    result.push_str("<|start_header_id|>tool<|end_header_id|>\n\n");
                    result.push_str(&msg.content);
                    result.push_str("<|eot_id|>");
                }
            }
        }

        result.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");

        result
    }
}

pub struct SimpleChatTemplate;

impl ChatTemplate for SimpleChatTemplate {
    fn apply(&self, messages: &[ChatMessage]) -> String {
        messages
            .iter()
            .map(|msg| match msg.role {
                ChatRole::System => format!("System: {}", msg.content),
                ChatRole::User => format!("User: {}", msg.content),
                ChatRole::Assistant => format!("Assistant: {}", msg.content),
                ChatRole::Tool => format!("Tool: {}", msg.content),
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

pub fn get_template_for_model(model_name: &str) -> Box<dyn ChatTemplate> {
    if model_name.to_lowercase().contains("llama") {
        Box::new(Llama3Template)
    } else {
        Box::new(SimpleChatTemplate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llama3_template() {
        let messages = vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("Hello!"),
        ];

        let template = Llama3Template;
        let result = template.apply(&messages);

        assert!(result.contains("<|begin_of_text|>"));
        assert!(result.contains("<|start_header_id|>system<|end_header_id|>"));
        assert!(result.contains("You are a helpful assistant."));
        assert!(result.contains("<|start_header_id|>user<|end_header_id|>"));
        assert!(result.contains("Hello!"));
        assert!(result.contains("<|eot_id|>"));
    }

    #[test]
    fn test_simple_template() {
        let messages = vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("Hello!"),
        ];

        let template = SimpleChatTemplate;
        let result = template.apply(&messages);

        assert!(result.contains("System: You are a helpful assistant."));
        assert!(result.contains("User: Hello!"));
    }

    #[test]
    fn test_get_template_for_llama() {
        let template = get_template_for_model("modularai/Llama-3.1-8B-Instruct-GGUF");
        let messages = vec![ChatMessage::user("Test")];
        let result = template.apply(&messages);

        assert!(result.contains("<|begin_of_text|>"));
    }

    #[test]
    fn test_get_template_for_other() {
        let template = get_template_for_model("some-other-model");
        let messages = vec![ChatMessage::user("Test")];
        let result = template.apply(&messages);

        assert!(result.contains("User: Test"));
    }
}
