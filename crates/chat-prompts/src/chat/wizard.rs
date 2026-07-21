use crate::{error::Result, utils::get_text_content_from_user_message, BuildChatPrompt};
use endpoints::chat::{
    ChatCompletionRequestMessage, ChatCompletionSystemMessage, ChatCompletionUserMessage,
};

/// Generate prompts for the `wizard-vicuna` model.
#[derive(Debug, Default, Clone)]
pub struct WizardCoderPrompt;
impl WizardCoderPrompt {
    /// Create a system prompt from a chat completion request message.
    fn create_system_prompt(&self, message: &ChatCompletionSystemMessage) -> String {
        let content = message.content();
        match content.is_empty() {
            true => String::from("Below is an instruction that describes a task. Write a response that appropriately completes the request."),
            false => content.to_string(),
        }
    }

    /// Create a user prompt from a chat completion request message.
    fn append_user_message(
        &self,
        system_prompt: impl AsRef<str>,
        message: &ChatCompletionUserMessage,
    ) -> String {
        let content = get_text_content_from_user_message(message);

        format!(
            "{system_prompt}\n\n### Instruction:\n{user_message}",
            system_prompt = system_prompt.as_ref().trim(),
            user_message = content.trim(),
        )
    }
}
impl BuildChatPrompt for WizardCoderPrompt {
    fn build(&self, messages: &mut Vec<ChatCompletionRequestMessage>) -> Result<String> {
        if messages.is_empty() {
            return Err(crate::error::PromptError::NoMessages);
        }

        // system prompt
        let system_prompt = match messages[0] {
            ChatCompletionRequestMessage::System(ref message) => {
                self.create_system_prompt(message)
            }
            _ => String::from("Below is an instruction that describes a task. Write a response that appropriately completes the request."),
        };

        let message = messages.last().unwrap();
        let mut prompt = match message {
            ChatCompletionRequestMessage::User(message) => {
                self.append_user_message(system_prompt, message)
            }
            _ => return Err(crate::error::PromptError::NoUserMessage),
        };

        prompt.push_str("\n\n### Response:");

        Ok(prompt)
    }
}
