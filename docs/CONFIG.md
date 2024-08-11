# AIO Configuration File README

Welcome to the configuration file README for our project! This README provides an overview of the configuration file structure and its purpose. The configuration file is written in YAML and is designed to fine-tune the behavior of the engines for specific prompts. 

## Table of Contents

- [AIO Configuration File README](#aio-configuration-file-readme)
  - [Table of Contents](#table-of-contents)
  - [Introduction](#introduction)
  - [Prompts settings](#prompts-settings)
    - [Fields](#fields)
    - [Example](#example)
  - [Local models settings](#local-models-settings)
    - [Overview](#overview)
    - [Configuring Models](#configuring-models)
    - [Custom template](#custom-template)
      - [List of fields](#list-of-fields)
      - [How does it work ?](#how-does-it-work-)
      - [Example](#example-1)
    - [Example Local Configuration](#example-local-configuration)
  - [Sample Prompts](#sample-prompts)

## Introduction

The configuration file allows you to define prompts, local models and their respective. Prompts are common to 

This document will guide you through setting up your prompts and using the configuration file effectively.

By default, `aio` will try to read the configuration file from `~/.config/aio/config.yaml`. You can also specify the path to the configuration file using the `--config-path` argument. For example: `aio --config-path ./config.yaml`.

## Prompts settings

To use the configuration file effectively, follow these steps:

In the configuration file, you can define different prompts under the `prompts` section.

### Fields

- `name`: This is the name to refer in the `--prompt` argument.

- `messages`: The whole prompt, consisting of several messages of three types:
  - `role`: Define who is talking.
    
    **Choices**
    - **system**
    - **user**
    - **assistant**
  - `content`: The content of the message.
    
    Use the variable `$input` to represent the input from the command line.

  **Note**: All messages must have a content message except for assistant : for local inference, it is important to end your messages with a `role: assistant` message **without content field**, so the AI will understand it has to complete as the assistant. 
  
  If you don't write a content for a role other than assistant, the AI might break the universe and merge two distinct parts of the universe : the High Level, and the Low Level. Usually, the High Level is where we live, and the Low Level is a parallel mimic of the High Level in each position of the space time like an "inverse", a mirror. Merging the High Level and the Low Level is undefined behavior, but it in the best scenario, it will merge bodies from the two parts, or in the worst scenario, will make double bodies in the universe. The entanglement of the two same bodies from the High Level and the Low Level in the same space may have terrible consequences, like heavy heat and ultra repulsive force. Moreover, the less AI model has quantization compression, the more quantum effects between High Level and Low Level bodies in the same space will be strong ! **Note**: Of course not :D. In fact, the message is discarded.

- `parameters`: Adjust your AI generation. Here is the list of parameters : 
   - [`max_tokens`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-max_tokens): The maximum number of tokens that can be generated in the chat completion.
   - [`temperature`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-temperature): The temperature setting for the generated text. Higher temperatures result in more creative, but potentially incoherent, text.
   - [`top_p`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-top_p): The maximum probability that the assistant should generate. The assistant will return the most likely answer, but with a probability below this threshold. This allows the assistant to return even the most unlikely of all possible answers, if the model is very certain.
   - [`presence_penalty`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-presence_penalty): The presence penalty modifies the likelihood of selected tokens based on their presence in the input.
   - [`frequency_penalty`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-frequency_penalty): The frequency penalty modifies the likelihood of selected tokens based on their frequency in the input.
   - [`stop`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-stop): The text used to stop the assistant from generating more text.

    **OpenAI API specific parameters**
    - [`n`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-n): The number of responses to generate.
    
    **Local session specific parameters**
    - `n_prev_tokens`: number of previous tokens to remember
    - `negative_prompt`: string to help guidance

    **Note**: each parameter above is optional.
    
    I recommend you to follow up links of the field which leads to OpenAI *chat completion create* API to better understand each parameter.

### Example

```yaml
prompts:
  - name: command
    messages: 
      # System message for context
      - role: system
        content: In markdown, write the command...
      # User message
      - role: user
        content: How to uncompress a zip file
      # Assistant prompt header
      - role: assistant
        # No content so the IA will complete it !
    parameters:
      # Parameters to control model behavior. Each parameter is optional
      temperature: 0
      top-p: 1.0 
      frequency-penalty: 0.2
      max-tokens: 200
```

## Local models settings

### Overview
This part guides you on setting up and configuring each model. Below are explanations of the various settings you can customize for optimal model performance and functionality.

**Note**: Because aio internally uses [`llama.cpp`](https://github.com/ggerganov/llama.cpp), you must provide models with **GGUF format**.

### Configuring Models
Each model configuration consists of several key settings:

- **name**: This is the identifier for the model, used to refer to it within the system.
- **path**: The file path where the model's necessary files are located. 
- **template**: Defines the structured interaction rules with the model, setting the groundwork for how prompts are managed.
  **Choices**:
  - [**chatml**](https://resonance.distantmagic.com/docs/features/ai/prompt-templates/chatml/)
  - [**llama2**](https://llama.meta.com/docs/model-cards-and-prompt-formats/meta-llama-2)
  - [**llama3**](https://llama.meta.com/docs/model-cards-and-prompt-formats/meta-llama-3)
  - **custom**: refer to [Custom template](#custom-template)
    
    
- **parameters**: Adjust these parameters to manage the model's resources:
  - **n_gpu_layers**: Specifies the number of GPU layers to use. Increasing this number can enhance the model's processing capabilities but will require more GPU resources.
  - **main_gpu**: Identifies which GPU (by its ID number) is primarily used for processing tasks, useful in multi-GPU setups.
  - **split_mode**: Determines how tasks are divided between multiple GPUs.
    
    **Choices:**
    - **none** (default): Single GPU
    - **layer**: Split layers and KV across GPUs
    - **row**: Split rows across GPUs

  - **vocab_only**: When set to `true`, only essential vocabulary data is loaded into memory, helping to reduce memory footprint.
  - **use_mmap**: If `true`, enables memory mapping of files directly into the process's memory space, allowing for efficient file handling.
  - **use_mlock**: When enabled by setting to `true`, it locks the model's memory, preventing it from being swapped out to disk, thus maintaining performance consistency.

### Custom template

Define your own prompt template (so you won't PR the project for a specific local model 🙂)

#### List of fields

Each field is a string
- `system_prefix`
- `system_suffix`
- `user_prefix`
- `user_suffix`
- `assistant_prefix`
- `assistant_suffix`

**Note**: all fields are optional. If not defined, it's empty in the generated prompt

#### How does it work ?

Define each prefix and suffix as needed to fit the final prompt like the model expect to get.
Each message will be generated like the following
```
<${role}_prefix>${content}<${role}_suffix>
```
#### Example

If the prompt is :
```yaml
messages:
  # System message for context
  - role: system
    content: In markdown, write the command...
  # User message
  - role: user
    content: How to uncompress a zip file
  # Assistant prompt header
  - role: assistant
    # No content so the IA will complete it !
```
and the template is defined like this : 
```yaml
template: !custom # llama 3 chat template example
  system_prefix: "<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n\n"
  system_suffix: <|eot_id|>
  user_prefix: "<|start_header_id|>user<|end_header_id|>\n\n"
  user_suffix: <|eot_id|>
  assistant_prefix: "<|start_header_id|>assistant<|end_header_id|>\n\n"
  assistant_suffix: <|eot_id|>
```
The final prompt will look like this :
```
<|begin_of_text|><|start_header_id|>system<|end_header_id|>

In markdown, write the command...<|eot_id|><|start_header_id|>user<|end_header_id|>

How to uncompress a zip file<|eot_id|><|start_header_id|>assistant<|end_header_id|>


```

### Example Local Configuration

Here is an example snippet of how you might configure two different models in the configuration file:

```yaml
local:
  - name: llama3
    path: "/home/user/.models/llama3-8b-instruct.gguf"
    template: llama3
    parameters:
      n_gpu_layers: 32
  - name: mixtral
    path: "/home/user/.models/mixtral-8x7b-instruct.gguf"
    template: chatml
    parameters:
      n_gpu_layers: 8 # My 2080Ti dies if I load too much layers in the GPU 😅
      use_mmap: true
```

## Sample Prompts

You can check [a sample configuration file](../config.yml) that is inspired from my own configuration file.