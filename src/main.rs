use std::{env, path, process};
use clap::{Arg};

// template dir ================================================================

fn get_template_dir() -> Result<path::PathBuf, String> {
    let mut p = env::current_exe()
        .map_err(|e| e.to_string())?;
    p.pop();
    p.push("templates");
    
    if !p.exists() {
        Err(format!("template dir `{}` does not exist", p.display()))        
    } else {
        Ok(p)
    }
}

fn template_values(
    template_dir: &path::PathBuf,
    subdir_name: &str,
) -> Result<Vec<String>, String> {
    let dir = template_dir.join(subdir_name)
        .as_path()
        .read_dir()
        .map_err(|e| e.to_string())?;

    let mut templates = Vec::new();

    for dir_entry in dir {
        let path = dir_entry
            .map_err(|e| e.to_string())?
            .path();

        let stem = match path.file_stem() {
            Some(s) => match s.to_os_string().into_string() {
                Ok(s) => s,
                Err(_) => return Err(
                    format!("weird path `{}`", path.display()),
                ),
            },
            None => return Err(
                format!("no file stem for path `{}`", path.display()),
            ),
        };

        templates.push(stem);
    }

    Ok(templates)
}

fn template_path(
    template_dir: &path::PathBuf,
    subdir: &str,
    name: &str,
    ext: &str,
) -> path::PathBuf {
    let mut path = template_dir.clone();
    path.push(subdir);
    path.push(name);
    path.set_extension(ext);

    path
}

// scripts =====================================================================

const SCRIPT_ENV: &str = "/usr/bin/env";
const SCRIPT_RUNNER: &str = "sh";

fn script_path(template_dir: &path::PathBuf, name: &str) -> path::PathBuf {
    template_path(template_dir, "scripts", name, "sh")
}

/// uses /usr/bin/env to run scripts
fn run_script(
    template_dir: &path::PathBuf,
    project_dir: &path::PathBuf,
    name: &str,
) -> Result<(), String> {
    println!("[executing script `{}`]", name);

    let status = process::Command::new(SCRIPT_ENV)
        .current_dir(project_dir)
        .arg(SCRIPT_RUNNER)
        .arg(script_path(template_dir, name))
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .status();

    match status {
        Err(e) => Err(e.to_string()),
        Ok(status) => {
            if !status.success() {
                Err(format!("script `{}` failed", name))
            } else {
                Ok(())
            }
        }
    }
}

// main functionality ==========================================================

#[derive(Debug)]
struct Config {
    flake: String,
    scripts: Vec<String>,
    license: Option<String>,
    path: path::PathBuf,
}

impl Config {
    /// selbstverstaendlich
    fn from_args(template_dir: &path::PathBuf) -> Result<Self, String> {
        // get available flakes and licenses
        let flakes = template_values(template_dir, "flakes")?;
        let scripts = template_values(template_dir, "scripts")?;
        let licenses = template_values(template_dir, "licenses")?;

        // create CLAP
        let matches = clap::Command::new("make-project")
            .version("0.0")
            .author("garrisonhh <garrisonhh@pm.me>")
            .about("a personalized project initializer.")
            .arg(
                Arg::new("license")
                    .short('l')
                    .long("license")
                    .value_parser(licenses)
            )
            .arg(
                Arg::new("scripts")
                    .short('s')
                    .long("scripts")
                    .value_delimiter(',')
                    .value_parser(scripts)
            )
            .arg(
                Arg::new("flake")
                    .required(true)
                    .help("nix flake template")
                    .value_parser(flakes)
            )
            .arg(
                Arg::new("path")
                    .required(true)
                    .help("directory to place project in")
            )
            .get_matches();

        // args -> Config
        let flake = matches.get_one::<String>("flake")
            .unwrap()
            .to_owned(); 
        let scripts = matches.get_many::<String>("scripts")
            .unwrap()
            .map(String::to_owned)
            .collect::<Vec<_>>();
        let license = matches.get_one::<String>("license")
            .map(String::to_owned);
        let path = env::current_dir()
            .map_err(|e| e.to_string())?
            .join(matches.get_one::<String>("path").unwrap());

        Ok(Self {
            flake,
            scripts,
            license,
            path,
        })
    }

    fn sub_path(&self, name: &str) -> path::PathBuf {
        let mut path = self.path.clone();
        path.push(name);

        path
    }

    /// use config to write a project directory
    fn make_project(self, template_dir: &path::PathBuf) -> Result<(), String> {
        // create project folder
        if self.path.exists() {
            return Err("this directory already exists".to_string());
        }

        match std::fs::create_dir_all(&self.path) {
            Ok(()) => (),
            Err(e) => return Err(e.to_string()),
        }

        // copy license
        if let Some(name) = &self.license {
            match std::fs::copy(
                template_path(template_dir, "licenses", &name, "md"),
                self.sub_path("LICENSE.md"),
            ) {
                Ok(_) => (),
                Err(e) => return Err(e.to_string()),
            }
        }

        // copy flake
        match std::fs::copy(
            template_path(template_dir, "flakes", &self.flake, "nix"),
            self.sub_path("flake.nix"),
        ) {
            Ok(_) => (),
            Err(e) => return Err(e.to_string()),
        }

        // run any scripts
        for name in self.scripts {
            run_script(template_dir, &self.path, &name)?;
        }

        Ok(())
    }
}

fn run() -> Result<(), String> {
    let template_dir = get_template_dir()?;
    Config::from_args(&template_dir)?.make_project(&template_dir)
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(msg) => {
            eprintln!("error: {}", msg);
            process::exit(1);
        },
    }
}
