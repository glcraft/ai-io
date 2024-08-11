pub mod arguments;
mod config;
mod credentials;
mod filesystem;
mod formatters;
mod generators;
mod runner;
mod serde_io;
#[cfg(test)]
mod tests;
mod utils;

mod openai {}

use arguments as args;
use clap::Parser;
use formatters::Formatter;
use log::{debug, info};
use serde_io::DeserializeExt;
use tokio_stream::StreamExt;

macro_rules! raise_str {
    ($expr:expr) => {
        raise_str!($expr, "{}")
    };
    ($expr:expr, $text:literal) => {{
        $expr.map_err(|e| format!($text, e))?
    }};
}

fn get_creds(creds_path: &str) -> Result<credentials::Credentials, String> {
    Ok(raise_str!(
        credentials::Credentials::from_yaml_file(filesystem::resolve_path(creds_path).as_ref()),
        "Failed to parse credentials file: {}"
    ))
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let app_args = {
        let mut args = args::Args::parse();
        if args.input.is_empty() {
            use std::io::Read;
            let mut str_input = std::string::String::new();
            let mut stdin = std::io::stdin();
            raise_str!(
                stdin.read_to_string(&mut str_input),
                "Failed to read input from stdin: {}"
            );
            args.input = str_input.trim().to_string();
        }
        args
    };

    let log_level = match app_args.verbose {
        0 => simplelog::LevelFilter::Error,
        1 => simplelog::LevelFilter::Warn,
        2 => simplelog::LevelFilter::Info,
        3 => simplelog::LevelFilter::Debug,
        _ => simplelog::LevelFilter::Trace,
    };

    simplelog::TermLogger::init(
        log_level,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();
    debug!("Arguments: {:#?}", app_args);
    let config =
        config::Config::from_yaml_file(filesystem::resolve_path(&app_args.config_path).as_ref())
            .map(|mut c| {
                c.insert_default_endpoints();
                c
            })
            .map_err(|e| {
                format!(
                    "An error occured while loading or creating configuration file: {}",
                    e
                )
            })?;
    info!("Loaded config");
    debug!("Config: {:#?}", config);
    // std::process::exit(0);
    let mut formatter: Box<dyn Formatter> = match app_args.formatter {
        args::FormatterChoice::Markdown => Box::new(formatters::new_markdown_formatter()),
        args::FormatterChoice::Raw => Box::new(formatters::new_raw_formatter()),
    };
    let mut runner = runner::Runner::new(app_args.run);

    let mut stream = match app_args.engine {
        args::Subcommands::Api(args_engine) => generators::openai::run(
            credentials::Credentials::get_api_key(&app_args.creds_path, &args_engine.endpoint),
            config,
            args_engine,
            &app_args.input,
        )
        .await
        .map_err(|e| format!("Failed to request OpenAI API: {}", e))?,
        args::Subcommands::Local(args_engine) => {
            generators::llama::run(config, args_engine, &app_args.input)
                .await
                .map_err(|e| format!("Unable to run local model: {}", e))?
        }
        args::Subcommands::FromContent(args_engine) => {
            generators::from_file::run(config, args_engine, &app_args.input)
                .await
                .map_err(|e| format!("Failed to read from file: {}", e))?
        }
    };

    loop {
        match stream.next().await {
            Some(Ok(token)) => {
                raise_str!(formatter.push(&token), "Failed to parse markdown: {}");
                raise_str!(
                    runner.push(&token),
                    "Failed push text in the runner system: {}"
                );
            }
            Some(Err(e)) => Err(e.to_string())?,
            None => break,
        }
    }
    raise_str!(formatter.end_of_document(), "Failed to end markdown: {}");
    raise_str!(runner.end_of_document(), "Failed to run code: {}");
    Ok(())
}
