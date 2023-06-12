use std::{env, path, process::exit};
use clap::{Command, Arg};

fn error_exit(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    std::process::exit(1);
}

trait UnwrapOrExit<T> {
    fn unwrap_or_exit(self, msg: &str) -> T;
}

impl<T, E> UnwrapOrExit<T> for Result<T, E> {
    fn unwrap_or_exit(self, msg: &str) -> T {
        match self {
            Err(_) => error_exit(msg),
            Ok(x) => x,
        }
    }
}

impl<T> UnwrapOrExit<T> for Option<T> {
    fn unwrap_or_exit(self, msg: &str) -> T {
        match self {
            None => error_exit(msg),
            Some(x) => x,
        }
    }
}

// TODO this is dumb, this should be called once and used as a constant
fn get_template_dir() -> path::PathBuf {
    env::current_dir()
        .unwrap_or_exit("failed to retrieve path to cwd")
        .join("templates")
}

fn template_values(
    template_dir: &path::PathBuf,
    subdir_name: &str,
) -> Vec<String> {
    let p = template_dir.join(subdir_name);

    p
        .read_dir()
        .unwrap_or_exit(
            format!("failed to read template subdirectory '{:?}'", p)
                .as_str()
        )
        .map(|entry|
            entry
                .unwrap_or_exit(format!(
                    "failed to read entry in template subdirectory '{}'",
                    subdir_name,
                ).as_str())
                .path()
                .file_stem()
                .unwrap_or_exit("failed to get file stem of template")
                .to_os_string()
                .into_string()
                .unwrap_or_exit("filename could not be converted to unicode")
        )
        .collect()
}

#[derive(Debug)]
struct Config {
    flake: String,
    license: Option<String>,
    path: path::PathBuf,
}

impl Config {
    /// selbstverstaendlich
    fn from_args() -> Self {
        // get available flakes and licenses
        let template_dir = get_template_dir();
        let flakes = template_values(&template_dir, "flakes");
        let licenses = template_values(&template_dir, "licenses");

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

        Self {
            flake,
            license,
            path,
        }
    }

    fn template_path(subdir: &str, name: &str, ext: &str)
        -> path::PathBuf
    {
        let mut path = get_template_dir();
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
    fn make_project(self) -> Result<(), String> {
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
                Self::template_path("licenses", &name, "md"),
                Self::dest_path(&self.path, "LICENSE", "md"),
            ) {
                Ok(_) => (),
                Err(e) => return Err(e.to_string()),
            }
        }

        match std::fs::copy(
            Self::template_path("flakes", &self.flake, "nix"),
            Self::dest_path(&self.path, "flake", "nix"),
        ) {
            Ok(_) => (),
            Err(e) => return Err(e.to_string()),
        }

        Ok(())
    }
}

fn main() {
    let res = Config::from_args().make_project();
    if let Err(msg) = res {
        eprintln!("error: {}", msg);
        exit(1);
    }
}
