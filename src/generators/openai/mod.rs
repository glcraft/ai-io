pub mod credentials;

use log::info;
use serde::{Serialize, Deserialize};
use tokio_stream::StreamExt;
use crate::{
    args, config::prompt::Stop, utils::{
        hashmap, FlattenTrait, SplitBytesFactory
    }
};
use crate::config::prompt::{Prompt, Parameters as PromptParameters, Message, Role};

use super::{ResultRun, Error};

#[derive(Debug, Default, Serialize)]
pub struct ChatRequestParameters {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<bool>,
    #[serde(skip_serializing_if = "Stop::is_none")]
    pub stop: Stop,
}

impl From<PromptParameters> for ChatRequestParameters {
    fn from(parameters: PromptParameters) -> Self {
        Self {
            max_tokens: parameters.max_tokens,
            temperature: parameters.temperature,
            top_p: parameters.top_p,
            presence_penalty: parameters.presence_penalty,
            frequency_penalty: parameters.frequency_penalty,
            n: parameters.n,
            stop: parameters.stop,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(flatten)]
    parameters: ChatRequestParameters,
}
#[allow(dead_code)]
impl ChatRequest {
    pub fn new(model: String) -> Self {
        Self {
            model,
            ..Default::default()
        }
    }
    pub fn add_message(mut self, role: Role, content: String) -> Self {
        self.messages.push(Message { role, content: Some(content) });
        self
    }
    pub fn add_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages.extend(messages.into_iter().filter(|m| m.content.is_some()));
        self
    }
    pub fn set_parameters(mut self, parameters: ChatRequestParameters) -> Self {
        self.parameters = parameters;
        self
    }
    pub fn into_stream(mut self) -> Self {
        self.parameters.stream = Some(true);
        self
    }
}
impl Default for ChatRequest {
    fn default() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            messages: Vec::new(),
            parameters: Default::default()
        }
    }
}


#[derive(Debug, Deserialize)]
struct Delta {
    // pub role: Option<Role>,
    pub content: Option<String>
}
#[derive(Debug, Deserialize)]
struct Choice {
    pub delta: Delta,
    // pub index: u32,
    // pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ChatResponse {
    Message{
        // id: String,
        // object: String,
        // created: u64,
        // model: String,
        choices: Vec<Choice>,
    },
    Status {
        status: String
    },
    #[serde(rename = "[DONE]")]
    Done,
}

impl std::fmt::Display for ChatResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ChatResponse::Message{choices, ..} => {
                if choices.is_empty() {
                    return Ok(());
                }
                let choice = &choices[0];
                if let Some(content) = choice.delta.content.as_ref() {
                    write!(f, "{}", content)?;
                }
                Ok(())
            },
            ChatResponse::Status { status } => write!(f, "<Status from OpenAI API: {}>", status),
            ChatResponse::Done => {
                if cfg!(feature = "debug") {
                    write!(f, "\n<<Stream finished>>")
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl ChatResponse {
    pub fn from_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        // eprintln!("from_bytes|1: {}", String::from_utf8_lossy(&bytes));
        if !bytes.starts_with(b"data: ") {
            use serde::de::Error;
            let json = match serde_json::from_slice::<serde_json::Value>(bytes) {
                Ok(v) => v,
                _ => return Err(serde_json::Error::custom("Not a data line")),
            };
            return if let Some(error) = json.get("error") {
                Err(serde_json::Error::custom(format!("OpenAI Error (type: {}, code: {}): {}", error["type"].as_str().unwrap_or(""), error["code"].as_str().unwrap_or(""), error["message"].as_str().unwrap_or(""))))
            } else if let Some(status) = json.get("status") {
                Ok(ChatResponse::Status { status: status.as_str().ok_or(serde_json::Error::custom("OpenAI Status is not a string"))?.to_string() })
            } else {
                Err(serde_json::Error::custom("Json found but unknown format"))
            }
        }
        let bytes = &bytes[6..];
        // eprintln!("from_bytes|2: {}", String::from_utf8_lossy(&bytes));
        if bytes.starts_with(b"[DONE]") {
            return Ok(ChatResponse::Done);
        }
        serde_json::from_slice(bytes)
    }
    #[inline]
    pub fn from_bytes(bytes: bytes::Bytes) -> Result<Self, serde_json::Error> {
        Self::from_slice(&bytes)
    }
}

pub async fn run(api_key: Option<String>, mut config: crate::config::Config, args: args::ApiArgs, input: &str) -> ResultRun {
    // if api_key.is_empty() {
    //     return Err(Error::Custom("OpenAI API key not found".into()));
    // }
    let endpoint = config.endpoints
        .remove(&args.endpoint)
        .ok_or(Error::Custom(format!("Endpoint {} not found", args.endpoint).into()))?;

    let prompt = if let Some(config_prompt) = args.prompt {
        config.prompts.0.into_iter()
            .find(|prompt| prompt.name == config_prompt)
            .ok_or(Error::Custom("Prompt not found".into()))?
            .format_contents(&hashmap!(input => input))
    } else {
        Prompt::from_input(input)
    };

    let Some(model) =  args.model.or(endpoint.default_model) else {
        return Err(Error::Custom("Model not found".into()));
    };

    // Send a request
    let chat_request = ChatRequest::new(model)
        .add_messages(prompt.messages)
        .set_parameters(prompt.parameters.into())
        .into_stream();

    let client = reqwest::Client::new();
    let mut stream = client.post(format!("{}/chat/completions", endpoint.url))
        .header("User-Agent", aio_cargo_info::user_agent!())
        .json(&chat_request);
    if let Some(api_key) = api_key {
        info!("API key found");
        stream = stream.header("Authorization", format!("Bearer {}", api_key))
    }
    let stream = stream.send()
        .await?
        .bytes_stream();

    let mut split_bytes_factory = SplitBytesFactory::new(b"\n\n");

    let stream_string = stream
        .map(move |input| -> Result<_, Error> {
            let input = input?;
            #[cfg(debug_assertions)]
            {
                use std::io::Write;
                static LOG: once_cell::sync::Lazy<std::sync::Mutex<std::fs::File>> = once_cell::sync::Lazy::new(|| {
                    std::sync::Mutex::new(
                        std::fs::File::options()
                            .create(true)
                            .write(true)
                            .open(format!("{}/openai_stream.txt", crate::filesystem::cache_dir()))
                            .expect("Failed to open log file")
                    )
                });
                if let Ok(mut log) = LOG.lock() { 
                    log.write_all(&input)
                        .and_then(|_| log.write_all(b"\n---\n"))
                        .expect("Debug: Failed to write to log file"); 
                }
            }
            
            Ok(split_bytes_factory.new_iter(input))
        })
        .flatten_result_iter()
        .map(|v| {
            let v = v?;
            let chat_resp = ChatResponse::from_bytes(v);
            match chat_resp {
                Ok(resp) => Ok(resp),
                Err(e) => Err(Error::SerializeJSON(e))
            }
        })
        .map_while(|resp| {
            match resp {
                Ok(msg @ (ChatResponse::Message { .. } | ChatResponse::Status{ .. })) => Some(Ok(msg.to_string())),
                Ok(ChatResponse::Done) => None,
                Err(e) => Some(Err(e)),
            }
        });
    Ok(Box::pin(stream_string))
}