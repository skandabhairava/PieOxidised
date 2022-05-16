mod config;
use config::{self as conf, MainConfig, ProjectConfig};
mod commands {
    pub mod out_commands;
    pub mod in_commands;
}
use commands::{out_commands::{self, OutArgs}, in_commands::{self, InArgs}};
use std::{process, path::{Path, PathBuf}, fs};
use spinach::term;
use clap::Parser;

////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    
    let config_loc = setup();

    // IF PROJECT CONFIG (w/ VALIDATION) EXISTS
    if let Some(_project_conf) = is_in_proj() {
        let args = InArgs::parse();
            match args.command {
                in_commands::InSubCommands::List => {}
            }
            return;
    }

    let args = OutArgs::parse();
    match args.command {
        out_commands::OutSubCommands::List => { out_commands::list().unwrap() }
        out_commands::OutSubCommands::Cfg => {out_commands::config(&config_loc, &conf::start_config_if_not).unwrap();}
        out_commands::OutSubCommands::New { name, short_description } => {out_commands::new(&name, &short_description).unwrap();}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

fn is_in_proj() -> Option<ProjectConfig>{
    let project_conf = Path::new("project.json");
    if project_conf.exists(){
        let project_conf_result: Result<ProjectConfig, serde_json::Error> = serde_json::from_str(&fs::read_to_string(project_conf).expect("Could not read 'project.json', please try again."));
        if !project_conf_result.is_err() {
            return Some(project_conf_result.unwrap());
        }
    }
    None
}

fn setup() -> PathBuf{
    #[cfg(windows)]
    let _enabled = ansi_term::enable_ansi_support();

    ctrlc::set_handler(|| {
        term::show_cursor();
        process::exit(1);
    }).expect("Error setting Ctrl-C Handler");

    let config_loc = MainConfig::get_file_loc().unwrap();
    conf::start_config_if_not(&config_loc).unwrap();

    config_loc
}