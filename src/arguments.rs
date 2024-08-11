use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};

/// Program to communicate with large language models and AI API 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Configuration file
    #[arg(long, global = true, default_value_t = format!("{1}{0}config.yml", std::path::MAIN_SEPARATOR, crate::filesystem::config_dir()))]
    pub config_path: String,
    /// Credentials file
    /// 
    /// Used to store API keys
    #[arg(long, global = true, default_value_t = format!("{1}{0}creds.yml", std::path::MAIN_SEPARATOR, crate::filesystem::cache_dir()))]
    pub creds_path: String,
    /// Verbose mode
    /// 
    /// Count: 
    /// 0: errors,
    /// 1: warnings,
    /// 2: info,
    /// 3: debug,
    /// 4: trace
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Engine name
    /// 
    /// The name can be followed by custom prompt name from the configuration file
    /// (ex: openai:command)
    #[command(subcommand)]
    pub engine: Subcommands,
    /// Format the completion in the terminal
    /// 
    /// Possible values: markdown, raw
    #[arg(long, short, global = true, value_enum, default_value_t = Default::default())]
    pub formatter: FormatterChoice,
    /// Run code block if the language is supported
    #[arg(long, short, global = true, value_enum, default_value_t = Default::default())]
    pub run: RunChoice,
    /// User text prompt
    /// 
    /// If the text is empty, it will be read from stdin
    #[arg(global = true, default_value_t = Default::default())]
    pub input: String,
}

/// aio subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum Subcommands {
    /// OpenAI API
    Api(ApiArgs),
    /// Run local model
    FromContent(FromContentArgs),
    /// Display the content of a file
    Local(LocalArgs),
}

/// OpenAI API arguments
#[derive(ClapArgs, Debug, Clone)]
pub struct ApiArgs {
    /// API endpoint name
    /// 
    /// The name of the endpoint specified in the configuration file
    #[arg(long, short, default_value = "openai")]
    pub endpoint: String,
    /// Model name
    /// 
    /// The name of the model from /models API endpoint
    #[arg(long, short)]
    pub model: Option<String>,
    /// Prompt name
    /// 
    /// The name of the prompt defined in the configuration file
    #[arg(long, short)]
    pub prompt: Option<String>,
}
/// FromFile arguments (not used)
#[derive(ClapArgs, Debug, Clone)]
pub struct FromContentArgs {
    /// Interpret input as file path instead of text
    #[arg(long, short = 'p')]
    pub file: bool
}

/// Local model arguments
#[derive(ClapArgs, Debug, Clone)]
pub struct LocalArgs {
    /// Model name
    /// 
    /// The name of the model defined in the configuration file
    #[arg(long, short)]
    pub model: String,
    /// Prompt name
    /// 
    /// The name of the prompt defined in the configuration file.
    /// If not provided, it will select the "default" prompt in the configuration file
    /// or the first prompt in the configuration file if the "default" prompt is not defined
    #[arg(long, short)]
    pub prompt: Option<String>,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum FormatterChoice {
    /// Markdown display
    #[default]
    Markdown,
    /// Raw display
    Raw,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum RunChoice {
    /// Doesn't run anything
    #[default]
    No,
    /// Ask to run code
    Ask,
    /// Run code without asking
    Force
}