use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Prompts(pub Vec<Prompt>);

impl Default for Prompts {
    fn default() -> Self {
        Prompts(vec![
                Prompt {
                    name: "command".to_string(),
                    messages: vec![
                        Message {
                            role: Role::System,
                            content: Some("In markdown, write the unix command that best fits my request in a block of code under a \"## Command\" then describe the program and each parameter in \"## Explanation\".".to_string()),
                        },
                        Message {
                            role: Role::User,
                            content: Some("$input".to_string()),
                        },
                        Message {
                            role: Role::Assistant,
                            content: None,
                        },
                    ],
                    parameters: Parameters {
                        max_tokens: Some(200),
                        temperature: Some(0.0),
                        top_p: Some(1.0),
                        presence_penalty: Some(0.0),
                        frequency_penalty: Some(0.2),
                        ..Default::default()
                    },
                },
                Prompt {
                    name: "ask".to_string(),
                    messages: vec![
                        Message {
                            role: Role::System,
                            content: Some("You are a powerful intelligent conversational chatbot. Unless I tell you otherwise, answer to me in an informative way. You should format the text in Markdown.".to_string()),
                        },
                        Message {
                            role: Role::User,
                            content: Some("$input".to_string()),
                        },
                        Message {
                            role: Role::Assistant,
                            content: None,
                        },
                    ],
                    parameters: Parameters {
                        max_tokens: Some(300),
                        temperature: Some(0.7),
                        top_p: Some(1.0),
                        presence_penalty: Some(0.0),
                        frequency_penalty: Some(0.0),
                        ..Default::default()
                    },
                },
            ])
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Prompt {
    pub name: String,
    pub messages: Vec<Message>,
    pub parameters: Parameters,
}

impl Prompt {
    pub fn from_input(input: &str) -> Self {
        Self {
            name: "noname".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: Some(input.into()),
            }],
            ..Default::default()
        }
    }
    pub fn format_contents(mut self, args: &HashMap<String, String>) -> Self {
        self.messages.iter_mut().for_each(|m|{ m.format_content_as_ref(args); });
        self
    }
    pub fn formatted_messages(&self, args: &HashMap<String, String>) -> Vec<Message> {
        self.messages.iter().cloned().map(|v| Message::format_content(v, args)).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "User"),
            Role::Assistant => write!(f, "Assistant"),
            Role::System => write!(f, "System"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Option<String>,
}

#[allow(dead_code)]
impl Message {
    pub fn format_content(mut self, args: &HashMap<String, String>) -> Self {
        self.content = self.content.map(|c| crate::config::format_content(&c, args).to_string());
        self
    }
    pub fn format_content_as_ref(&mut self, args: &HashMap<String, String>) -> &mut Self {
        self.content = self.content.as_mut().map(|c| crate::config::format_content(c, args).to_string());
        self
    }
}
#[cfg(feature = "local-llm")]
#[derive(Debug, Deserialize, Serialize)]
pub enum Algorithm {
    SoftMax{
        min_keep: usize,
    },
    Greedy,
    Mirostat{
        min_keep: usize,
        tau: f32,
        eta: f32,
        m: i32
    },
    MirostatV2{
        min_keep: usize,
        tau: f32,
        eta: f32,
    },
}
impl Default for Algorithm {
    fn default() -> Self {
        Algorithm::MirostatV2 { 
            min_keep: 50, 
            tau: 5.0, 
            eta: 0.1,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Stop::is_none")]
    #[serde(default)]
    pub stop: Stop,
    
    //OpenAI only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    //Local only
    #[cfg(feature = "local-llm")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_n: Option<i32>,
    #[cfg(feature = "local-llm")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[cfg(feature = "local-llm")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tail_free: Option<f32>,
    #[cfg(feature = "local-llm")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typical: Option<f32>,
    #[cfg(feature = "local-llm")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f32>,
    #[cfg(feature = "local-llm")]
    #[serde(default)]
    pub algorithm: Algorithm,
}

#[cfg(feature = "local-llm")]
impl From<Parameters> for llama_cpp::standard_sampler::StandardSampler {
    fn from(parameters: Parameters) -> Self {
        use llama_cpp::standard_sampler::SamplerStage;
        let mut stages = vec![];
        if let Some(last_n) = parameters.last_n {
            stages.push(SamplerStage::RepetitionPenalty{
                repetition_penalty: parameters.frequency_penalty.unwrap_or(1.0),
                frequency_penalty: parameters.frequency_penalty.unwrap_or(0.0),
                presence_penalty: parameters.presence_penalty.unwrap_or(0.0),
                last_n,
            });
        }
        if let Some(temp) = parameters.temperature {
            stages.push(SamplerStage::Temperature(temp));
        }
        if let Some(top_k) = parameters.top_k {
            stages.push(SamplerStage::TopK(top_k));
        }
        if let Some(tail_free) = parameters.tail_free {
            stages.push(SamplerStage::TailFree(tail_free));
        }
        if let Some(typical) = parameters.typical {
            stages.push(SamplerStage::Typical(typical));
        }
        if let Some(top_p) = parameters.top_p {
            stages.push(SamplerStage::TopP(top_p));
        }
        if let Some(min_p) = parameters.min_p {
            stages.push(SamplerStage::MinP(min_p));
        }
        match parameters.algorithm {
            Algorithm::SoftMax { min_keep } => Self::new_softmax(stages, min_keep),
            Algorithm::Greedy => Self::new_greedy(),
            Algorithm::Mirostat { min_keep, tau, eta, m } => Self::new_mirostat(stages, min_keep, tau, eta, m),
            Algorithm::MirostatV2 { min_keep, tau, eta } => Self::new_mirostat_v2(stages, min_keep, tau, eta),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum Stop {
    #[default]
    None,
    #[serde(untagged)]
    One(String),
    #[serde(untagged)]
    Many(Vec<String>),
}

impl Stop {
    pub fn is_none(&self) -> bool {
        matches!(self, Stop::None)
    }
}