pub mod config;
pub mod template;
pub mod stop;

use tokio_stream::StreamExt;

use llama_cpp::{
    standard_sampler::StandardSampler, LlamaModel, SessionParams, TokensToStrings
};
use once_cell::sync::OnceCell;
use log::{debug, info};
use crate::{
    args, config::Config, utils::hashmap
};
use super::{Error, ResultRun};

static LOCAL_LLAMA: OnceCell<LlamaModel> = OnceCell::new();

fn init_model(config: &config::Model) -> Result<(), Error> {
    info!("Loading LLaMA model at {}", config.path);
    debug!("Parameters: {:?}", config.parameters);
    let Ok(llama) = LlamaModel::load_from_file(
        &config.path,
        (&config.parameters).into(),
    ) else {
        return Err(Error::Custom("Failed to load LLaMA model".into()))
    };
    info!("LLaMA model loaded");
    LOCAL_LLAMA.set(llama).map_err(|_| Error::Custom("Failed to set LLaMA model".into()))
}

pub async fn run(
    config: Config, 
    args: args::LocalArgs,
    input: &str
) -> ResultRun {
    let prompt = match args.prompt {
        Some(prompt) => config.prompts.0
            .iter()
            .find(|v| v.name == prompt),
        None => config.prompts.0
            .iter()
            .find(|v| v.name == "default")
            .or_else(|| config.prompts.0.first())
    }
    .ok_or_else(|| Error::Custom("Prompt not found in config".into()))?;
    let messages = prompt.formatted_messages(&hashmap!(input => input));
    let model_config = config.local.models.into_iter()
        .find(|c| c.name == args.model)
        .ok_or_else(|| Error::Custom("Model not found in config".into()))?;
    if LOCAL_LLAMA.get().is_none() {
        init_model(&model_config)?;
    }
    let model = LOCAL_LLAMA.get().unwrap();
    
    let session_params = SessionParams::default();
    let mut session = model.create_session(session_params).map_err(|_| Error::Custom("Failed to create session".into()))?;
    
    let context_tokens = model_config.template.messages_to_tokens(model, &messages).map_err(|_| Error::Custom("Failed to convert prompt messages to tokens".into()))?;
    if log::log_enabled!(log::Level::Debug) {
        debug!("Tokens: ");
        for token in &context_tokens {
            print!("{}({})", String::from_utf8_lossy(model.detokenize(*token)), token.0);
        }
        println!();
        let (bos, eos) = (model.bos(), model.eos());
        debug!("Special tokens:");
        debug!("bos: {}({})", String::from_utf8_lossy(model.detokenize(bos)), bos.0);
        debug!("eos: {}({})", String::from_utf8_lossy(model.detokenize(eos)), eos.0);
    }
    session
        .advance_context_with_tokens_async(context_tokens).await
        .map_err(|_| Error::Custom("Failed to advance context".into()))?;

    let completion = session
        .start_completing_with(StandardSampler::default(), prompt.parameters.max_tokens.unwrap_or(1024) as _)
        .map_err(|e| Error::Custom(format!("Failed to start completion: {e}").into()))?;
    if log::log_enabled!(log::Level::Trace) {
        let completion_stream = StreamExt::map(completion,  |token| Ok(format!("{}({})", model.token_to_piece(token), token.0)));
        Ok(Box::pin(completion_stream))
    } else {
        let mut discard_tokens = model_config.template.stop_tokens(model).map_err(|_| Error::Custom("Failed to convert prompt messages to tokens".into()))?;
        let completion_stream = 
            StreamExt::map(
                StreamExt::take_while(
                    TokensToStrings::new(completion, model.clone()), 
                    move |token| !discard_tokens.check(token)
                ),
                Ok
            );
        
        Ok(Box::pin(completion_stream))
    }
}