use std::{env, fs};

use ansi_term::Color;
use clap::{Parser, Subcommand};
use pie::Result;

use super::out_commands;

/////////////////////////////////////////////////////////////////////
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

    /// Displays or edits the Project's version.
    Ver{ ver: Option<String> }

}

/////////////////////////////////////////////////////////////////////

pub fn version(ver: Option<String>) -> Result<()> {
    let project_conf = out_commands::is_in_proj(&env::current_dir().unwrap());
    if let Some(mut conf) = project_conf{
        println!("{}{}", Color::Green.paint("|> Current Version: "), Color::Green.bold().paint(conf.version));
        if let Some(version) = ver{
            conf.version = version;
            fs::write("project.json", serde_json::to_string_pretty(&conf)?)?;
            println!("{}{}", Color::Green.paint("√ |> New Version: "), Color::Green.bold().paint(conf.version));
        }
    } else {
        println!("{}", Color::Green.paint("X |> There was an error locating 'project.json'. Please try again."));
    }

    Ok(())
}