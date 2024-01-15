# Together AI CLI Client

This is a simple CLI client for the [Together AI API](https://api.together.xyz/). 
It allows you to list and interact with the models provided by this API.

## Building

This is a cargo project. In order to build it, you need to have Rust installed and then run this command:

```bash
 cargo build -r
```

## Prerequisites for runnign the CLI

Please go to [Together AI](https://www.together.ai/) and get an API key.

Then make sure that you have the API key in the environment variable `TOGETHER_API_KEY`.

You can set the API key manually or you can use a .env file like this one:

```bash
TOGETHER_API_KEY=your-api-key
```

## Usage Examples:

```bash
together_ai_experiments.exe --help
```

Displays the help message, like something along these lines:

```
Simple program to test on the command line together.ai's API.

 Here are some examples:
 together_ai_experiments.exe answer -p "Who is the Jeremy Howard, the founder of fast.ai?" -m upstage/SOLAR-10.7B-Instruct-v1.0 togethercomputer/falcon-40b-instruct mistralai/Mixtral-8x7B-Instruct-v0.1

Usage: together_ai_experiments.exe <COMMAND>

Commands:
  list-models  List models available to the API key
  answer       Answer a prompt
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Using all chat models in Together AI. (May take a while to complete):

```bash
together_ai_experiments.exe answer -p "What is the Knapsack problems in computer science?" -a
```

Using a specific model:

```bash
together_ai_experiments.exe answer -p "Who is the Jeremy Howard, the founder of fast.ai?" -m upstage/SOLAR-10.7B-Instruct-v1.0 togethercomputer/falcon-40b-instruct mistralai/Mixtral-8x7B-Instruct-v0.1
```

Using a file as input with `--prompt-is-file`:

```bash
together_ai_client.exe answer -p ..\..\prompts\prompt_example.txt -a --prompt-is-file
```






