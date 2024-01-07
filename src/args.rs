use clap::{Args, Parser, Subcommand};

/// Simple program to test on the command line together.ai's API.
#[derive(Parser, Debug)]
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
pub(crate) struct ListModelsCommand {

}

#[derive(Debug, Args)]
pub(crate) struct AnswerCommand {

    /// The prompt to send to the API.
    #[clap(short, long)]
    pub(crate) prompt: String,

    /// The model to use.
    #[clap(short, long, num_args = 0..)]
    pub(crate) models: Option<Vec<String>>,

    /// The temperature to use.
    #[clap(short, long)]
    temperature: Option<f32>,

    /// The top p to use.
    #[clap(long)]
    top_p: Option<f32>,

    /// The top k to use.
    #[clap(long)]
    top_k: Option<u32>,

    /// The repetition penalty to use.
    #[clap(short, long)]
    repetition_penalty: Option<f32>,
}