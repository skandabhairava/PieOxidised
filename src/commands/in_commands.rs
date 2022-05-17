use std::{env, fs, path::Path, process};

use ansi_term::Color;
use clap::{Parser, Subcommand};
use pie::{Result, run_cmd};

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
    Ver{ ver: Option<String> },

    Run{
        #[clap(allow_hyphen_values = true)] 
        args: Vec<String> 
    },

}

/////////////////////////////////////////////////////////////////////

enum RunPy{
    Run,
    DontRun
}

pub fn run(mut args: Vec<String>) -> Result<()> {
    let venv_path = Path::new("venv");
    #[cfg(windows)]
    {
        if !venv_path.exists(){
            println!("{}", Color::Red.paint("X |> Venv Not Found. Initialising a venv. Please wait"));
            run_cmd("python", &vec!["-m", "venv", "venv"], false, ||{}, ||{});
            run_venv_cmd("pip", &mut vec!["install", "-r", Path::new("..").join("requirements.txt").to_str().unwrap()].into_iter().map(String::from).collect(), RunPy::DontRun, false)?;
            println!("{}", Color::Green.paint("√ |> Initialised a venv, and installed requirements. Please restart the program."));
            process::exit(1);
        }

        run_venv_cmd("python", &mut args, RunPy::Run, true)?;
    }

    #[cfg(not(windows))]
    {
        if !venv_path.exists(){
            println!("{}", Color::Red.paint("X |> Venv Not Found. Initialising a venv. Please wait"));
            run_cmd("python3", &vec!["-m", "venv", "venv"], false, ||{}, ||{});
            run_venv_cmd("pip", &mut vec!["install", "-r", Path::new("..").join("requirements.txt").to_str().unwrap()].into_iter().map(String::from).collect(), RunPy::DontRun, false)?;
            println!("{}", Color::Green.paint("√ |> Initialised a venv, and installed requirements. Please restart the program."));
            process::exit(1);
        }

        run_venv_cmd("python3", &mut args, RunPy::Run, true)?;
    }

    Ok(())
}

fn run_venv_cmd(main_cmd: &str, args: &mut Vec<String>, run: RunPy, should_display_output: bool) -> Result<()> {

    let project_conf = out_commands::is_in_proj(&env::current_dir().unwrap());
    if let Some(conf) = project_conf{
        env::set_current_dir(conf.working_directory)?;
        if let RunPy::Run = run{
            args.insert(0, conf.entry_point);
        }

        #[cfg(windows)]
        let path_win = Path::new("..").join("venv").join("Scripts").join(main_cmd);
        let path_else = Path::new("..").join("venv").join("bin").join(main_cmd);
        let main_cmd = if cfg!(windows) {path_win.to_str().unwrap()} else {path_else.to_str().unwrap()};

        run_cmd(main_cmd, &args, should_display_output, || {}, || {});
    }

    Ok(())
}

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