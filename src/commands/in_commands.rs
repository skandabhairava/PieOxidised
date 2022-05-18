use std::{env, fs, path::Path, process};

use ansi_term::Color;
use clap::{Parser, Subcommand};
use pie::{Result, run_cmd};
use spinach::{Spinach, Spinner};

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

    /// Runs the python project
    Run{
        #[clap(allow_hyphen_values = true)] 
        args: Vec<String> 
    },

    /// Runs `pip show` inside the venv
    Show{
        #[clap(allow_hyphen_values = true)]
        args: Vec<String>
    },

    /// Runs `pip` inside the venv
    Pip{
        #[clap(allow_hyphen_values = true)]
        args: Vec<String>
    },

    /// Runs `pip list` inside the venv
    List{
        #[clap(allow_hyphen_values = true)]
        args: Vec<String>
    },

    /// Runs `pip install` inside the venv
    Install{
        #[clap(allow_hyphen_values = true)]
        args: Vec<String>
    },

    /// Runs `pip uninstall` inside the venv
    Uninstall{
        #[clap(allow_hyphen_values = true)]
        args: Vec<String>
    },

    /// Generates a requirements.txt file for the project.
    #[clap(long_about("Generates a requirements.txt file for the project. If --install is passed, it will install the packages in requirements.txt file."))]
    Reqs{

        // Install the packages in requirements.txt file.
        #[clap(long)]        
        install: bool
    },

    /// Automatically installs all modules used in the project, to the venv.
    AutoInstall,

    Push{
        /// The Commit message.
        commit_msg: String,

        /// Remote to push repository to.
        #[clap(short('R'), default_value("origin"))]
        remote: String,

        #[clap(short('B'), default_value("main"))]
        /// Branch to push repository to.
        branch: String
    }

}

/////////////////////////////////////////////////////////////////////

fn spinach_log(spinach: &Spinach, frozen_msg: &str, new_message: &str, err: bool){
    if err{
        spinach.freeze("X |> ", Color::Red.paint(frozen_msg).to_string(), spinach::Color::Ignore, Color::Yellow.paint(new_message).to_string())
    } else {
        spinach.freeze("√ |> ", Color::Green.paint(frozen_msg).to_string(), spinach::Color::Ignore, Color::Yellow.paint(new_message).to_string())
    }
}

pub fn push(commit_msg: String, remote: String, branch: String) -> Result<()>{
    println!("{}", Color::Green.paint("|> Pushing to Github..."));
    let spinner = Spinner::new(vec!["-", "\\", "|", "/"], 130);
    let spinach = Spinach::new_with(spinner, "Pushing to Github", spinach::Color::Ignore);

    let mut err = false;

    run_cmd("git", &vec!["add", "."], false, ||{
        err = true;
    }, ||{
        spinach_log(&spinach, "Added files to git index.", "Committing files to local repo", false);
    });
    if err{
        spinach.stop_with("X |> ", Color::Red.paint("Could not add files to git index.").to_string(), spinach::Color::Ignore);
        process::exit(1);
    }

    run_cmd("git", &vec!["commit", "-m", &commit_msg], false, ||{
        err = true;
    }, ||{
        spinach_log(&spinach, "Committed files to local repo.", "Pushing files to github", false);
    });
    if err {
        spinach.stop_with("X |> ", Color::Red.paint("Could not commit files to local repo.").to_string(), spinach::Color::Ignore);
        process::exit(1);
    }

    run_cmd("git", &vec!["push", &remote, &branch], false, ||{
        err = true;
    }, || {
    });
    if err{
        spinach.stop_with("X |> ", Color::Red.paint("Could not push files to github.").to_string(), spinach::Color::Ignore);
        process::exit(1);
    }
    spinach.stop_with("√ |> ", Color::Green.paint("Pushed files to github.").to_string(), spinach::Color::Ignore);

    Ok(())
}

pub fn auto_install() -> Result<()> {
    reqs(false, true)?;
    reqs(true, true)?;
    Ok(())
}

pub fn reqs(install: bool, display_progress: bool) -> Result<()> {
    if install{
        let req_txt = Path::new("requirements.txt");
        if req_txt.exists(){
            run_pip("install", &mut vec!["-r", Path::new("..").join("requirements.txt").to_str().unwrap()].into_iter().map(String::from).collect(), false)?;
            
            if display_progress{ println!("{}", Color::Green.bold().paint("√ |> Installed packages in 'requirements.txt'")); }

            return Ok(());
        }
        
        if display_progress{ println!("{}", Color::Red.bold().paint("X |> Could not find 'requirements.txt'")); }
        return Ok(());
        
    }

    run_cmd("pipreqs", &vec![String::from("--force")], false, ||{
        if display_progress{ println!("{}", Color::Red.bold().paint("X |> Command 'pipreqs' failed. Please check if 'pipreqs' is installed, if not, install it from pip/pypi. If pipreqs is installed, please check your project for correct import statements")); }
    }, || {
        if display_progress{ println!("{}", Color::Green.bold().paint("√ |> Written requirements in 'requirements.txt'")); }
    });

    Ok(())
}

pub fn run_pip(cmd: &str, args: &mut Vec<String>, should_display_output: bool) -> Result<()> {

    if cmd != ""{
        args.insert(0, cmd.to_string());
    }

    #[cfg(windows)]
    run_venv_cmd("pip", args, RunPy::DontRun, should_display_output)?;

    #[cfg(not(windows))]
    run_venv_cmd("pip3", args, RunPy::DontRun, should_display_output)?;

    Ok(())
}

enum RunPy{
    Run,
    DontRun
}

pub fn run(mut args: Vec<String>) -> Result<()> {
    #[cfg(windows)]
    run_venv_cmd("python", &mut args, RunPy::Run, true)?;

    #[cfg(not(windows))]
    run_venv_cmd("python3", &mut args, RunPy::Run, true)?;

    Ok(())
}

fn run_venv_cmd(main_cmd: &str, args: &mut Vec<String>, run: RunPy, should_display_output: bool) -> Result<()> {

    let venv_path = Path::new("venv");

    if !venv_path.exists(){
        #[cfg(windows)]
        {
            println!("{}", Color::Red.paint("X |> Venv Not Found. Initialising a venv. Please wait"));
            run_cmd("python", &vec!["-m", "venv", "venv"], false, ||{}, ||{});
            run_venv_cmd("pip", &mut vec!["install", "-r", Path::new("..").join("requirements.txt").to_str().unwrap()].into_iter().map(String::from).collect(), RunPy::DontRun, false)?;
            println!("{}", Color::Green.paint("√ |> Initialised a venv, and installed requirements from 'requirements.txt'. Please restart the program."));
            process::exit(1);
        }

        #[cfg(not(windows))]
        {
            println!("{}", Color::Red.paint("X |> Venv Not Found. Initialising a venv. Please wait"));
            run_cmd("python3", &vec!["-m", "venv", "venv"], false, ||{}, ||{});
            run_venv_cmd("pip3", &mut vec!["install", "-r", Path::new("..").join("requirements.txt").to_str().unwrap()].into_iter().map(String::from).collect(), RunPy::DontRun, false)?;
            println!("{}", Color::Green.paint("√ |> Initialised a venv, and installed requirements from 'requirements.txt'. Please restart the program."));
            process::exit(1);
        }
    }

    let project_conf = out_commands::is_in_proj(&env::current_dir().unwrap());
    
    if let Some(conf) = project_conf{
    
        env::set_current_dir(conf.working_directory)?;
        if let RunPy::Run = run{
            args.insert(0, conf.entry_point);
        }

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