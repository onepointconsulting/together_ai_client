use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
/// Simple program to test on the command line together.ai's API.{n}{n}
/// Here are some examples:{n}
/// together_ai_experiments.exe answer -p "Who is the Jeremy Howard, the founder of fast.ai?" -m upstage/SOLAR-10.7B-Instruct-v1.0 togethercomputer/falcon-40b-instruct mistralai/Mixtral-8x7B-Instruct-v0.1
#[command(author, version, about, long_about = None)]
pub(crate) struct ClapArgs {

    #[clap[subcommand]]
    pub(crate) subcommand: TogetherAiSubcommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum TogetherAiSubcommand {

    /// List models available to the API key.
    ListModels(ListModelsCommand),

    /// Answer a prompt.
    Answer(AnswerCommand),
}

#[derive(Debug, Args)]
pub(crate) struct ListModelsCommand {}

#[derive(Debug, Args)]
pub(crate) struct AnswerCommand {

    /// The prompt to send to the API.
    #[clap(short, long)]
    pub(crate) prompt: String,

    /// The model to use.
    #[clap(short, long, num_args = 0..)]
    pub(crate) models: Option<Vec<String>>,

    /// Whether to try out all chat models or not
    #[clap(short, long, action)]
    pub(crate) all_models: bool,

    /// Whether the prompt is a normal question of a file from which the prompt should be read.
    #[clap(long, action)]
    pub(crate) prompt_is_file: bool,

    /// The maximum tokens to generate.
    #[clap(long)]
    pub(crate) max_tokens: Option<usize>,

    /// The temperature to use. Defaults to 0.0.
    #[clap(short, long)]
    pub(crate) temperature: Option<f32>,

    /// The top p to use.
    #[clap(long)]
    pub(crate) top_p: Option<f32>,

    /// The top k to use.
    #[clap(long)]
    pub(crate) top_k: Option<u32>,

    /// The repetition penalty to use.
    #[clap(short, long)]
    pub(crate) repetition_penalty: Option<f32>,
}