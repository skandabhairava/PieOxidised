use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(
    author = "Terroid", 
    version = "0.0.1", 
    about = "A Loose structured and easy to use python project manager.", 
    long_about = "A Loose structured and easy to use python project manager. Different sets of commands can be accessed by using the cli inside a project directory or outside of it."
    )
]
pub struct InArgs {
    #[clap(subcommand)]
    pub command: InSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum InSubCommands{

    /// Displays Project config.
    List

}