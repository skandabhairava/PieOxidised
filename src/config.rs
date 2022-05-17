use std::fs;
use std::process;
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

use pie::{input, Result};

use ansi_term::Color;

////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug)]
pub struct MainConfig {
    pub dev: String,
    pub email: String,
    pub github: String,
}
    impl MainConfig {
        pub fn new(dev: &str, email: &str, github: &str) -> MainConfig {
            MainConfig { dev: dev.to_string(), email: email.to_string(), github: github.to_string() }
        }
        pub fn from_file() -> Result<MainConfig> {
            let path = MainConfig::get_file_loc()?;

            if !path.exists(){

                println!("{}", Color::Green.bold().paint("Welcome to Pie Project Manager!"));
                let dev = input(&(Color::Green.paint("Please enter your ").to_string() + &Color::Green.bold().paint("dev/brand name: ").to_string()))?;
                let email = input(&(Color::Green.paint("Please enter your ").to_string() + &Color::Green.bold().paint("email address: ").to_string()))?;
                let github = input(&(Color::Green.paint("Please enter your ").to_string() + &Color::Green.bold().paint("github profile url: ").to_string()))?;

                let conf = MainConfig::new(&dev, &email, &github);
                conf.write_json()?;
                println!("{}", Color::Green.paint(format!("Config File Successfully Created in {}!", path.to_str().unwrap_or("path-not-found"))));

                return Ok(conf);
            }

            let deserialized = fs::read_to_string(MainConfig::get_file_loc()?)?;
            Ok(serde_json::from_str(&deserialized)?)
        }

        fn get_path() -> Result<PathBuf> {
            if let Some(project_dir) = ProjectDirs::from("com", "terroid", "pie"){
                return Ok(project_dir.config_dir().to_owned());
            }
            Err(Box::new(io::Error::new(ErrorKind::InvalidData, "No path found")))
        }
        pub fn get_file_loc() -> Result<PathBuf> {
            Ok(MainConfig::get_path()?.join("config.json"))
        }
        pub fn write_json(&self) -> Result<()> {
            let path = MainConfig::get_path()?;

            if !path.exists(){
                fs::create_dir_all(&path)?;
            }

            let serialized = serde_json::to_string_pretty(&self)?;
            fs::write(MainConfig::get_file_loc()?, serialized)?;
            Ok(())
        }
    }
/////////////////////////////////////////////////////

pub fn start_config_if_not(config_loc: &PathBuf) -> Result<()>{
    if !config_loc.exists() {
        MainConfig::from_file()?;
        process::exit(0);
    }
    Ok(())
}

/////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectConfig{
    name: String,
    short_description: String,
    pub version: String,
    author: String,
    email: String,
    author_github: String,
    entry_point: String,
    working_directory: String,
    github: String,
    license: String
}
    impl ProjectConfig {
        pub fn new(name: &str, description: &str, config: &MainConfig) -> ProjectConfig{
            ProjectConfig { name: name.to_string(),
                short_description: description.to_string(),
                version: String::from("0.0.1"),
                author: config.dev.to_string(),
                email: config.email.to_string(),
                author_github: config.github.to_string(),
                entry_point: name.to_string() + ".py",
                working_directory: String::from("src"),
                github: String::from(""),
                license: String::from("MIT")
            }
        }
    }
////////////////////////////////////////////////////

/*

{
    "name": name,
    "short_description": description,
    "version": "0.0.1",
    "author": config.dev,
    "email": config.email,
    "author_github": config.github,
    "entry_point": (name.to_string() + ".py"),
    "working_directory": "src",
    "github": "",
    "license": "MIT"
}

*/