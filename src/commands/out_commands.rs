use std::{path::{PathBuf, Path}, fs::{self, File}, process, env, result, io::{Write, Seek, Read, self}};
use clap::{Parser, Subcommand};
use pie::{Result, gitignore, run_cmd, input};
use spinach::{Spinach, Spinner};
use ansi_term::Color;
use random_string;
use remove_dir_all;

use walkdir::{DirEntry, WalkDir};
use zip::{write::FileOptions, result::ZipError};

use crate::{config::{MainConfig, ProjectConfig}, commands::in_commands};

pub fn is_in_proj(path: &PathBuf) -> Option<ProjectConfig>{
    let project_conf = path.join("project.json");
    if project_conf.exists(){
        let project_conf_result: result::Result<ProjectConfig, serde_json::Error> = serde_json::from_str(&fs::read_to_string(project_conf).expect("Could not read 'project.json', please try again."));
        if !project_conf_result.is_err() {
            return Some(project_conf_result.unwrap());
        }
    }
    None
}

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
    New{ name: String, short_description: String },

    /// List all the pie projects.
    List,

    /// Deletes a project.
    #[clap(long_about="Deletes a project. The name of the command is long as to not delete the project on accident.")]
    DeleteProject{ name: String },

    /// Packages a project.
    Pkg{ 

        /// The project to package.
        project: String,

        /// Force overwrites existing project file(.pie).
        #[clap(short('F'), long)]
        force: bool
    
    },

    /// Unpackages a packaged project.
    Unpkg{
        
        /// The project to unpackage.
        project: String,

        /// Force overwrites existing project dir.
        #[clap(short('F'), long)]
        force: bool

    }

}

/////////////////////////////////////////////////////////////////////

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        
        if name.to_str().unwrap().contains("venv") || name.to_str().unwrap().contains(".git") {
            continue;
        }

        if path.is_file() {
            zip.start_file(name.to_str().unwrap(), options)?;
            
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_str().unwrap(), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn compress(
    src_dir: &str,
    dst_file: &str,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(&path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method).unwrap();

    Ok(())
}

fn un_compress(filename: &str, dest_dir: &str) -> Result<()> {
    let fname = Path::new(&filename);
    let dest = Path::new(&dest_dir);
    let file = File::open(&fname)?;

    let mut archive = zip::ZipArchive::new(file)?;

    fs::create_dir(dest)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest.join(path.to_owned()),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

pub fn unpkg(project: &str, force: bool) -> Result<()> {
    let project_pie = Path::new(&project);
    let project_folder = project.split_once(".");

    if let Some(project_folder_dir) = project_folder {
        if project_folder_dir.1 != "pie"{
            println!("{}", Color::Red.paint(format!("X |> Project '{}' is not a pie project.", project)));
            process::exit(1);
        }

        if !project_pie.exists() {
            println!("{}", Color::Red.paint("X |> Project does not exist"));
            process::exit(1);
        }

        let project_dir_path  = Path::new(project_folder_dir.0);

        if force{
            remove_dir_all::remove_dir_all(project_dir_path)?;
        }

        if project_dir_path.is_dir() {
            println!("{}", Color::Red.paint(format!("X |> Project Folder '{}' already exists, consider using the '--force' flag.", project_folder_dir.0)));
            process::exit(1);
        }

        println!("{}", Color::Green.paint("|> Unpacking project."));
        let spinner = Spinner::new(vec!["-", "\\", "|", "/"], 130);
        let spinach = Spinach::new_with(spinner, Color::Yellow.paint("Packing project").to_string(), spinach::Color::Ignore);
        let result = un_compress(project, project_folder_dir.0);
        if result.is_err() {
            spinach.stop_with("X |> ", Color::Red.paint("Could not unpackage project.").to_string(), spinach::Color::Ignore);
            process::exit(1);
        }

        spinach_log(&spinach, "Unpackaged Project.", "Initialising venv.", false);

        let result = env::set_current_dir(&project_dir_path);
        if result.is_err(){
            spinach.stop_with("X |> ", Color::Red.paint(format!("Cannot change directory into {}", project)).to_string(), spinach::Color::Ignore);
            process::exit(1);
        }

        #[cfg(windows)]
        run_cmd("python", &vec!["-m", "venv", "venv"], false, ||{
            spinach_log(&spinach, "Could not find the 'python' command. Please check if python is installed, and if it is in your %PATH% environment variable", "Finalising Project Creation!", true);
        }, ||{
            spinach_log(&spinach, "Created a Virtual environment", "Finalising Project Creation!", false);
        });

        #[cfg(not(windows))]
        run_cmd("python3", &vec!["-m", "venv", "venv"], false, ||{
            spinach_log(&spinach, "Could not find the 'python3' command. Please check if python is installed, and if it is in your %PATH% environment variable", "Finalising Project Creation!", true);
        }, ||{
            spinach_log(&spinach, "Created a Virtual environment", "Finalising Project Creation!", false);
        });

        let result = in_commands::reqs(true, false, &None);
        if result.is_err(){
            spinach.stop_with("X |> ", Color::Red.paint("Cannot install requirements from 'requirements.txt'").to_string(), spinach::Color::Ignore);
            process::exit(1);
        }

        spinach_log(&spinach, "Installed requirements from 'requirements.txt'", "Finalising unpackaging", false);

        spinach.stop_with("√ |>", Color::Green.bold().paint(format!("Project '{}' successfully unpackaged!", project)).to_string(), spinach::Color::Ignore);


    } else {
        println!("{}", Color::Red.paint(format!("X |> '{}' is a folder. Please provide a valid '.pie' project.", project)));
        process::exit(1);
    }

    Ok(())
}

pub fn pkg(project : &str, force: bool) -> Result<()> {

    let path_str = project.to_string() + ".pie";
    let project_pie = Path::new(&path_str);

    if !Path::new(project).is_dir(){
        println!("{}", Color::Red.paint(format!("X |> Project folder '{}' not found!", project)));
        process::exit(1);
    }

    if force {
        if project_pie.exists() {
            fs::remove_file(project_pie).expect("Could not delete 'project.pie'");
        }
    }

    if project_pie.exists() {
        println!("{}", Color::Red.paint(format!("X |> Project '{}' already exists in this directory, consider using the '--force' flag.", path_str)));
        process::exit(1);
    }

    println!("{}", Color::Green.paint("|> Packing project."));
    let spinner = Spinner::new(vec!["-", "\\", "|", "/"], 130);
    let spinach = Spinach::new_with(spinner, Color::Yellow.paint("Packing project").to_string(), spinach::Color::Ignore);

    let mut err = false;

    // Go inside project directory, and note down the requirements.
    env::set_current_dir(project)?;
    in_commands::reqs(false, false, &None)?;
    env::set_current_dir("..")?;

    match compress(project, env::current_dir()?.join(format!("{}.pie", project)).to_str().unwrap(), zip::CompressionMethod::Stored){
        Err(_) => {err = true;},
        _ => {}
    }

    if err{
        spinach.stop_with("X |> ", Color::Red.paint("Could not package project.").to_string(), spinach::Color::Ignore);
        process::exit(1);
    }
    spinach.stop_with("√ |>", Color::Green.bold().paint(format!("Project '{}' successfully packaged!", project)).to_string(), spinach::Color::Ignore);
    Ok(())
}

pub fn delete_project(name: &str) -> Result<()> {

    let proj_dir = env::current_dir()?.join(name);

    if let Some(_project_conf) = is_in_proj(&proj_dir) {
        let captcha = name.to_string().to_uppercase() + "-" + &random_string::generate(5, "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        println!("{}{}", Color::Red.paint("|> Please type this captcha to confirm project deletion: "), Color::Green.paint(&captcha));
        let input_captcha = input(Color::Green.paint("|> Enter captcha: ").to_string())?;
        if input_captcha.to_uppercase() == captcha{
            
            let result = remove_dir_all::remove_dir_all(proj_dir);
            if result.is_err(){
                println!("{}", Color::Red.paint("X |> There was an issue while removing the directories. Please try again."));
                return Ok(());
            }
            println!("{}", Color::Green.paint("√ |> Project successfully deleted."));
            return Ok(());

        } else {
            
            println!("{}", Color::Red.paint("X |> Wrong Captcha. Project deletion, aborted."));

        }
    }

    println!("{}", Color::Red.paint("X |> The given folder is not a pie-project."));

    Ok(())
}

pub fn list() -> Result<()> {
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

    println!("{} {}", Color::Green.paint("|> Projects in this directory: "), Color::Green.bold().paint(format!("{:?}", projs)));

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

    spinach_log(&spinach, "Created project files!", "Creating local git repo!", false);

    let result = env::set_current_dir(&relative_path);
    if result.is_err(){
        spinach.stop_with("X |> ", Color::Red.paint(format!("Cannot change directory into {}", name)).to_string(), spinach::Color::Ignore);
        process::exit(1);
    }

    run_cmd("git", &vec!["init"], false, ||{
        spinach_log(&spinach, "Could not find the 'git' command.", "Creating virtual env!", true);
    }, ||{
        spinach_log(&spinach, "Initialised a local Git repo!", "Creating virtual env!", false);
    });

    #[cfg(windows)]
    run_cmd("python", &vec!["-m", "venv", "venv"], false, ||{
        spinach_log(&spinach, "Could not find the 'python' command. Please check if python is installed, and if it is in your %PATH% environment variable", "Finalising Project Creation!", true);
    }, ||{
        spinach_log(&spinach, "Created a Virtual environment", "Finalising Project Creation!", false);
    });

    #[cfg(not(windows))]
    run_cmd("python3", &vec!["-m", "venv", "venv"], false, ||{
        spinach_log(&spinach, "Could not find the 'python3' command. Please check if python is installed, and if it is in your %PATH% environment variable", "Finalising Project Creation!", true);
    }, ||{
        spinach_log(&spinach, "Created a Virtual environment", "Finalising Project Creation!", false);
    });

    spinach.stop_with("√ |>", Color::Green.bold().paint(format!("Project '{}' successfully created!", name)).to_string(), spinach::Color::Ignore);

    Ok(())
}

fn create_project_files(relative_path: &Path, name: &str, description: &str) -> Result<()> {
    fs::create_dir_all(relative_path.join("src"))?;
    fs::write(relative_path.join("src").join(name.to_string() + ".py"), "#!/usr/bin/env python3\n\ndef main():\n    print(\"Hello World!\")\n\nif __name__ == \"__main__\":\n    main()")?;
    fs::write(relative_path.join("README.md"), format!("# {}\n\n{}", name, description))?;
    fs::write(relative_path.join(".gitignore"), gitignore())?;
    fs::write(relative_path.join("requirements.txt"), "")?;
    let config = MainConfig::from_file().expect(&Color::Red.paint("Could not read main config file. Please run the 'cfg' command to rewrite the config file.").to_string());
    fs::write(relative_path.join("project.json"), 
    serde_json::to_string_pretty(&ProjectConfig::new(name, description, &config))?)?;

    Ok(())
}