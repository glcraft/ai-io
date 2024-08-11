use serde::{Deserialize, Serialize};
use crate::{
    config::prompt::{Message, Role},
    utils::vec_merge
};
use llama_cpp::{LlamaTokenizationError, Token};
use super::stop::{stop_manager, StopManager};

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct CustomTemplate {
    pub system_prefix: String,
    pub system_suffix: String,
    pub user_prefix: String,
    pub user_suffix: String,
    pub assistant_prefix: String,
    pub assistant_suffix: String,
}
#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptTemplate {
    #[default]
    ChatML,
    Llama2,
    Llama3,
    Custom(CustomTemplate)
}

impl PromptTemplate {
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            PromptTemplate::ChatML => "chatml",
            PromptTemplate::Llama2 => "llama2",
            PromptTemplate::Llama3 => "llama3",
            PromptTemplate::Custom(_) => "custom",
        }
    }
    pub fn messages_to_tokens(&self, model: &llama_cpp::LlamaModel, prompt: &[Message]) -> Result<Vec<Token>, LlamaTokenizationError> {
        let mut tokens = Vec::new();
        tokens.push(model.bos());
        match self {
            Self::ChatML => Self::tokens_chatml(&mut tokens, model, prompt),
            Self::Llama2 => Self::tokens_llama2(&mut tokens, model, prompt),
            Self::Llama3 => Self::tokens_llama3(&mut tokens, model, prompt),
            Self::Custom (custom_template) => Self::tokens_custom(&mut tokens, model, prompt, custom_template),
        }?;
        Ok(tokens)
    }
    pub fn tokens_chatml(tokens: &mut Vec<Token>, model: &llama_cpp::LlamaModel, prompt: &[Message]) -> Result<(), LlamaTokenizationError> {
        let im_start = model.tokenize_bytes("<|im_start|>", false, true)?;
        let im_end = model.tokenize_bytes("<|im_end|>", false, true)?;
        let nl = model.tokenize_bytes("\n", false, true)?;
        let [system, user, assistant] = [
            model.tokenize_bytes("system", false, true)?,
            model.tokenize_bytes("user", false, true)?,
            model.tokenize_bytes("assistant", false, true)?
        ];
        prompt.iter()
            .filter(|m| !(matches!(m.role, Role::System | Role::User) && m.content.is_none()))
            .for_each(|m| {
                let role_tokens = match m.role {
                    Role::System => &system,
                    Role::User => &user,
                    Role::Assistant => &assistant
                };
                vec_merge!(tokens, &im_start, role_tokens, &nl);
                if let Some(content) = m.content.as_ref() {
                    vec_merge!(tokens, &model.tokenize_bytes(content, false, false).unwrap(), &im_end, &nl);
                }
            });
        Ok(())
    }
    pub fn tokens_llama2(tokens: &mut Vec<Token>, model: &llama_cpp::LlamaModel, prompt: &[Message]) -> Result<(), LlamaTokenizationError> {
        let system_start = model.tokenize_bytes("<<SYS>>", false, true)?;
        let system_end = model.tokenize_bytes("<</SYS>>", false, true)?;
        let inst_start = model.tokenize_bytes("[INST]", false, true)?;
        let inst_end = model.tokenize_bytes("[/INST]", false, true)?;
        let eos = model.tokenize_bytes("</s>", false, true)?;
        let nl = model.tokenize_bytes("\n", false, true)?;
        prompt.iter()
            .for_each(|m| {
                let Some(content) = m.content.as_ref() else { return; };
                let content_tokens = model.tokenize_bytes(content, false, false).unwrap();
                match m.role {
                    Role::System => vec_merge!(tokens, &inst_start, &system_start, &content_tokens, &system_end, &inst_end, &nl),
                    Role::User => vec_merge!(tokens, &inst_start, &content_tokens, &inst_end, &nl),
                    Role::Assistant => vec_merge!(tokens, &content_tokens, &eos, &nl),
                }
            });
        Ok(())
    }
    pub fn tokens_llama3(tokens: &mut Vec<Token>, model: &llama_cpp::LlamaModel, prompt: &[Message]) -> Result<(), LlamaTokenizationError> {
        let start_header_id = model.tokenize_bytes("<|start_header_id|>", false, true)?;
        let end_header_id = model.tokenize_bytes("<|end_header_id|>", false, true)?;
        let eot_id = model.tokenize_bytes("<|eot_id|>", false, true)?;
        let nl = model.tokenize_bytes("\n", false, true)?;
        let [system, user, assistant] = [
            model.tokenize_bytes("system", false, true)?,
            model.tokenize_bytes("user", false, true)?,
            model.tokenize_bytes("assistant", false, true)?
        ];
        prompt.iter()
            .filter(|m| !(matches!(m.role, Role::System | Role::User) && m.content.is_none()))
            .for_each(|m| {
                let role_tokens = match m.role {
                    Role::System => &system,
                    Role::User => &user,
                    Role::Assistant => &assistant
                };
                vec_merge!(tokens, &start_header_id, role_tokens, &end_header_id, &nl, &nl);
                if let Some(content) = &m.content {
                    vec_merge!(tokens, &model.tokenize_bytes(content, false, false).unwrap_or_default(), &eot_id);
                }
            });
        Ok(())
    }
    pub fn tokens_custom(tokens: &mut Vec<Token>, model: &llama_cpp::LlamaModel, prompt: &[Message], custom_template: &CustomTemplate) -> Result<(), LlamaTokenizationError> {
        let system_prefix_tokens = model.tokenize_bytes(&custom_template.system_prefix, false, true)?;
        let system_suffix_tokens = model.tokenize_bytes(&custom_template.system_suffix, false, true)?;
        let user_prefix_tokens = model.tokenize_bytes(&custom_template.user_prefix, false, true)?;
        let user_suffix_tokens = model.tokenize_bytes(&custom_template.user_suffix, false, true)?;
        let assistant_prefix_tokens = model.tokenize_bytes(&custom_template.assistant_prefix, false, true)?;
        let assistant_suffix_tokens = model.tokenize_bytes(&custom_template.assistant_suffix, false, true)?;
        prompt.iter()
            .filter(|m| !(matches!(m.role, Role::System | Role::User) && m.content.is_none()))
            .for_each(|m| {
                match m.role {
                    Role::System => {
                        let content_tokens = model.tokenize_bytes(m.content.as_ref().unwrap(), false, false).unwrap();
                        vec_merge!(tokens, &system_prefix_tokens, &content_tokens, &system_suffix_tokens)
                    }
                    Role::User => {
                        let content_tokens = model.tokenize_bytes(m.content.as_ref().unwrap(), false, false).unwrap();
                        vec_merge!(tokens, &user_prefix_tokens, &content_tokens, &user_suffix_tokens)
                    }
                    Role::Assistant => {
                        vec_merge!(tokens, &assistant_prefix_tokens);
                        if let Some(content) = &m.content {
                            let content_tokens = model.tokenize_bytes(content, false, false).unwrap_or_default();
                            vec_merge!(tokens, &content_tokens, &assistant_suffix_tokens)
                        }
                    },
                }
            });
        Ok(())
    }
    pub fn stop_tokens(&self, model: &llama_cpp::LlamaModel) -> Result<StopManager, LlamaTokenizationError> {
        let eos_str = String::from_utf8_lossy(model.detokenize(model.eos()));
        match self {
            PromptTemplate::ChatML => Ok(stop_manager!["<|im_end|>", eos_str]),
            PromptTemplate::Llama2 => Ok(stop_manager!["[INST]", eos_str]),
            PromptTemplate::Llama3 => Ok(stop_manager!["<|eot_id|>", eos_str]),
            PromptTemplate::Custom(custom_template) => Ok(stop_manager![&custom_template.user_prefix, eos_str]),
        }
    }
}

