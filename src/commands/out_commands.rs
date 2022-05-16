use std::{path::{PathBuf, Path}, fs, process, env, result};
use clap::{Parser, Subcommand};
use pie::{Result, gitignore, run_cmd};
use spinach::{Spinach, Spinner};
use ansi_term::Color;

use crate::config::{MainConfig, ProjectConfig};

/////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[clap(
    author = "Terroid", 
    version = "0.0.1", 
    about = "A Loose structured and easy to use python project manager.", 
    long_about = "A Loose structured and easy to use python project manager. Different sets of commands can be accessed by using the cli inside a project directory or outside of it."
    )
]
pub struct OutArgs {
    #[clap(subcommand)]
    pub command: OutSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum OutSubCommands{

    /// Edits the config file.
    Cfg,

    /// Creates a new python project.
    New{ name:String, short_description:String },

    /// List all the pie projects.
    List,

}

/////////////////////////////////////////////////////////////////////

pub fn list() -> Result<()>{
    let path_buf = env::current_dir()?;
    let paths = fs::read_dir(path_buf.as_path())?;

    let mut projs = vec![];

    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() && path.join("project.json").exists() {
            let project_conf_result: result::Result<ProjectConfig, serde_json::Error> = serde_json::from_str(&fs::read_to_string(path.join("project.json")).expect("Could not read 'project.json', please try again."));
            if !project_conf_result.is_err() {
                projs.push(path.file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned()
                    );
            }
        }
    }

    println!("> Projects in this directory: {:?}", projs);

    Ok(())
}

pub fn config<T>(config_loc: &PathBuf, func: &T) -> Result<()>
where
    T: Fn(&PathBuf) -> Result<()>
{
    if config_loc.exists() {
        fs::remove_file(&config_loc).unwrap();
    }
    func(&config_loc).unwrap();
    Ok(())
}

fn spinach_log(spinach: &Spinach, frozen_msg: &str, new_message: &str, err: bool){
    if err{
        spinach.freeze("X |> ", Color::Red.paint(frozen_msg).to_string(), spinach::Color::Ignore, Color::Yellow.paint(new_message).to_string())
    } else {
        spinach.freeze("√ |> ", Color::Green.paint(frozen_msg).to_string(), spinach::Color::Ignore, Color::Yellow.paint(new_message).to_string())
    }
}

pub fn new(name: &str, description: &str) -> Result<()> {
    
    let relative_path = Path::new(name);
    if relative_path.is_dir() {
        println!("{}", Color::Red.paint("X |> A Folder with that name already exists"));
        process::exit(0);
    }

    println!("{}", Color::Green.paint("|> Creating Project..."));
    let spinner = Spinner::new(vec!["-", "\\", "|", "/"], 130);
    let spinach = Spinach::new_with(spinner, "Creating Project", spinach::Color::Ignore);

    let result = create_project_files(relative_path, name, description);
    if result.is_err(){
        spinach.stop_with("X |> ", Color::Red.paint("Could not create project files.").to_string(), spinach::Color::Ignore);
        process::exit(1);
    }

    spinach.freeze("√ |> ", Color::Green.paint("Created project files!").to_string(), spinach::Color::Ignore, Color::Yellow.paint("Creating local git repo!").to_string());

    let result = env::set_current_dir(&relative_path);
    if result.is_err(){
        spinach.stop_with("X |> ", Color::Red.paint(format!("Cannot change directory into {}", name)).to_string(), spinach::Color::Ignore);
        process::exit(1);
    }

    run_cmd("git", vec!["init"], false, ||{
        spinach_log(&spinach, "Could not find the 'git' command.", "Creating virtual env!", true);
    }, ||{
        spinach_log(&spinach, "Initialised a local Git repo!", "Creating virtual env!", false);
    });

    run_cmd("python", vec!["-m", "venv", "venv"], false, ||{
        spinach_log(&spinach, "Could not find the 'python' command. Please check if python is installed, and if it is in your %PATH% environment variable", "Finalising Project Creation!", true);
    }, ||{
        spinach_log(&spinach, "Created a Virtual environment", "Finalising Project Creation!", false);
    });

    spinach.stop_with("√ |>", Color::Green.bold().paint(format!("Project '{}' successfully created!", name)).to_string(), spinach::Color::Ignore);

    Ok(())
}

fn create_project_files(relative_path: &Path, name: &str, description: &str) -> Result<()> {
    fs::create_dir_all(relative_path.join("src"))?;
    fs::write(relative_path.join("src").join(name.to_string() + ".py"), "#! /usr/bin/python\n\ndef main():\n    print(\"Hello World!\")\n\nif __name__ == \"__main__\":\n    main()")?;
    fs::write(relative_path.join("README.md"), format!("# {}\n\n{}", name, description))?;
    fs::write(relative_path.join(".gitignore"), gitignore())?;
    fs::write(relative_path.join("requirements.txt"), "")?;
    let config = MainConfig::from_file().expect(&Color::Red.paint("Could not read config file. Please run the 'cfg' command to rewrite the config file.").to_string());
    fs::write(relative_path.join("project.json"), 
    serde_json::to_string_pretty(&ProjectConfig::new(name, description, &config))?)?;

    Ok(())
}