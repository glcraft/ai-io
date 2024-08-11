# CLI Arguments Documentation

- [CLI Arguments Documentation](#cli-arguments-documentation)
  - [Usage](#usage)
  - [Common arguments](#common-arguments)
    - [Global Usage Examples](#global-usage-examples)
  - [Commands](#commands)
    - [1. `aio api`](#1-aio-api)
      - [Usage Examples](#usage-examples)
    - [2. `aio from-content`](#2-aio-from-content)
      - [Usage Examples](#usage-examples-1)
    - [3. `aio local`](#3-aio-local)
      - [Usage Examples](#usage-examples-2)

## Usage

`aio <COMMAND> [OPTIONS] [INPUT] `

## Common arguments
- **`--config-path <PATH>`**: Path to the configuration file.
  
  **Default:** `~/.config/config.yml`

- **`--creds-path <PATH>`**: Path to the credentials file.
  
  **Default:** `~/.cache/creds.yml`

- **`-v, --verbose...`**: Verbose mode with different levels of logging.
  
  **Count:** 
    - 0 (default): errors only.
    - 1: warnings.
    - 2: info.
    - 3: debug.
    - 4: trace
  
  **Default:** 0

- **`-f, --formatter <CHOICE>`:** Format the completion in the terminal
  
  **Choice:**
    - **`markdown`:** Markdown display
    - **`raw`:** Raw display
  
  **Default:** markdown

- **`-r, --run <CHOICE>`**: Run code block if the language is supported
  
  **Choice:**
    - **`no`:** Doesn't run anything
    - **`ask`:** Ask to run block of code
    - **`force`:** Run code without asking
  
  **Default:** markdown

### Global Usage Examples

Set a custom path for configuration and credentials path
```bash
$ ./aio --config-path path/to/config.yml --creds-path path/to/creds.yml ...
```

Set log level to debug
```bash
$ ./aio -vvv ...
```

## Commands

### 1. `aio api`

Generate text using the OpenAI API.

**Arguments**:
- **`--model`** (or `-m`): The model name to use. The name must exist in the service. 
  Call `/models` GET API to get the list of all models provided by the service.
- **`--prompt`** (or `-p`): Optional. Provides a conversational prompt for the remote model. 
  The prompt configuration is defined in the configuration file. If this argument is not defined,
  a prompt with a user message containing the input will be generated.

#### Usage Examples

Generate text using GPT 3.5 Turbo model from OpenAI, with prompt "command" set in the configuration file:
```bash
$ ./program_name api --model gpt-3.5-turbo --prompt command "How to uncompress a tar.gz file ?"
```

Generate text with no formatting
```bash
$ ./program_name api --model gpt-3.5-turbo --prompt ask --formatter raw "What's the distance between the earth and the moon ?"
```

### 2. `aio from-content`

Displays the input like the AI completion does. It supports `--formatter` and `--run` arguments. If `--file` flag is filled, the input is a file path and will be read as the content.

**Arguments**:
- **`--file`** (or `-p`): Interpret the input as a file path instead of content

#### Usage Examples

Displays the markdown "# Hello\nWorld" in the console
```bash
$ ./program_name from-content "# Hello\nWorld"
```

Displays the content of the README file in the console
```bash
$ ./program_name from-content --file "./README.md"
```

Displays the content of stdin
```bash
$ cat ./README.md | ./program_name from-content
```

### 3. `aio local`

Generate text using local models.

**Arguments**:
- **`--model`** (or `-m`): Specifies the local model to use. The model configuration is defined in the configuration file.
- **`--prompt`** (or `-p`): Optional. Provides a conversational prompt for the local model. 
  The prompt configuration is defined in the configuration file. If this argument is not defined,
  a prompt with a user message containing the input will be generated.

#### Usage Examples

Generate text using "llama3" model, with prompt "command", both set in the configuration file:
```bash
$ ./program_name local --model llama3 --prompt command "How to uncompress a tar.gz file ?"
```
