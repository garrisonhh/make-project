use std::{env, path, process::exit};
use clap::{Command, Arg};

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

#[derive(Debug)]
struct Config {
    flake: String,
    license: Option<String>,
    path: path::PathBuf,
}

impl Config {
    /// selbstverstaendlich
    fn from_args(template_dir: &path::PathBuf) -> Result<Self, String> {
        // get available flakes and licenses
        let flakes = template_values(template_dir, "flakes")?;
        let licenses = template_values(template_dir, "licenses")?;

        // create CLAP
        let matches = Command::new("make-project")
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
        let license = matches.get_one::<String>("license")
            .map(String::to_owned);
        let path = matches.get_one::<String>("path")
            .map(path::PathBuf::from)
            .unwrap();

        Ok(Self {
            flake,
            license,
            path,
        })
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

    fn dest_path(base: &path::PathBuf, name: &str, ext: &str) -> path::PathBuf {
        let mut path = base.clone();
        path.push(name);
        path.set_extension(ext);

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
        
        // copy flake and license
        if let Some(name) = self.license {
            match std::fs::copy(
                Self::template_path(template_dir, "licenses", &name, "md"),
                Self::dest_path(&self.path, "LICENSE", "md"),
            ) {
                Ok(_) => (),
                Err(e) => return Err(e.to_string()),
            }
        }

        match std::fs::copy(
            Self::template_path(template_dir, "flakes", &self.flake, "nix"),
            Self::dest_path(&self.path, "flake", "nix"),
        ) {
            Ok(_) => (),
            Err(e) => return Err(e.to_string()),
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
            exit(1);
        },
    }
}
