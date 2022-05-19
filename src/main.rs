mod config;
use ansi_term::Color;
use config::{self as conf, MainConfig};
mod commands {
    pub mod out_commands;
    pub mod in_commands;
}
use commands::{out_commands::{self, OutArgs}, in_commands::{self, InArgs}};
use std::{process, path::{PathBuf, Path}, env};
use spinach::term;
use clap::Parser;

////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    
    let config_loc = setup();

    let args = env::args().collect::<Vec<String>>();
    if args.len() >= 2 {
        // <pie.exe> folder <args1> <args2>
        // <pie.exe> <abc.py> <args1> <args2>
        let path = PathBuf::from(&args[1]);
        if path.is_dir() {
            out_commands::run_folder(&args[1..]).unwrap();
            process::exit(0);
        } else if path.is_file() {
            out_commands::run_file(&args[1..]).unwrap();
            process::exit(0);
        } else if (&args[1]).ends_with(".py") {
            println!("{}", format!("{}{}{}", Color::Red.paint("X |> '"), Color::Red.bold().paint(&args[1]), Color::Red.paint("' Python file does not exist.")));
            process::exit(1);
        }
    }

    // IF PROJECT CONFIG (w/ VALIDATION) EXISTS
    if let Some(mut project_conf) = out_commands::is_in_proj(&Path::new(".").to_owned()) {
        let args = InArgs::parse();
            match args.command {
                in_commands::InSubCommands::Ver { ver } => { in_commands::version(ver, &mut project_conf).unwrap(); }
                in_commands::InSubCommands::Run { args } => { in_commands::run(args, project_conf).unwrap(); }
                in_commands::InSubCommands::Show { mut args } => { in_commands::run_pip("show", &mut args, true, Some(project_conf)).unwrap(); },
                in_commands::InSubCommands::Pip { mut args } => { in_commands::run_pip("", &mut args, true, Some(project_conf)).unwrap(); }
                in_commands::InSubCommands::List { mut args } => { in_commands::run_pip("list", &mut args, true, Some(project_conf)).unwrap(); }
                in_commands::InSubCommands::Install { mut args } => { in_commands::run_pip("install", &mut args, true, Some(project_conf)).unwrap(); }
                in_commands::InSubCommands::Uninstall { mut args } => { in_commands::run_pip("uninstall", &mut args, true, Some(project_conf)).unwrap(); }
                in_commands::InSubCommands::Reqs { install } => { in_commands::reqs(install, true, Some(project_conf)).unwrap(); }
                in_commands::InSubCommands::AutoInstall => {in_commands::auto_install(Some(project_conf)).unwrap();}
                in_commands::InSubCommands::Push { commit_msg, remote, branch } => { in_commands::push(commit_msg, remote, branch).unwrap(); }
            }
            return;
    }

    let args = OutArgs::parse();
    match args.command {
        out_commands::OutSubCommands::List => { out_commands::list().unwrap() }
        out_commands::OutSubCommands::DeleteProject { name } => { out_commands::delete_project(&name).unwrap(); }
        out_commands::OutSubCommands::Cfg => {out_commands::config(&config_loc, &conf::start_config_if_not).unwrap();}
        out_commands::OutSubCommands::New { name, short_description } => {out_commands::new(&name, &short_description).unwrap();}
        out_commands::OutSubCommands::Pkg { project, force } => { out_commands::pkg(&project, force).unwrap(); }
        out_commands::OutSubCommands::Unpkg { project, force } => { out_commands::unpkg(&project, force).unwrap(); }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

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