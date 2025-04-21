use std::fmt;
use std::fs;
use std::path::Path;

use dirs::home_dir;

#[cfg(target_os = "linux")]
use std::os::unix::fs::symlink;

#[cfg(target_os = "windows")]
use std::os::windows::fs::symlink;
#[derive(Debug)]
enum Op {
    Symfile,
    Symdir,
    Invalid,
}

#[derive(Debug)]
enum FormatFile {
    OrgTable,
    Csv,
}

impl FormatFile {
    fn parse(format_str: &str) -> Result<Self, String> {
        match format_str {
            "org" => Ok(FormatFile::OrgTable),
            "csv" => Ok(FormatFile::Csv),
            _ => Err(format!("Invalid format {format_str}.")),
        }
    }
}
#[derive(Debug)]
struct Dot {
    line: usize,
    name: String,
    source: String,
    dest: String,
    operation: Op,
}

#[allow(dead_code)]
pub struct Flags {
    file_format: FormatFile,
    headers: bool,
    force: bool,
}

impl Flags {
    pub fn build(file_format: &str, headers: bool, force: bool) -> Self {
        let file_format = FormatFile::parse(file_format).unwrap();
        Self {
            file_format,
            headers,
            force,
        }
    }
}

pub struct Dots {
    filename: String,
    flags: Flags,
    dotfiles: Vec<Dot>,
}

impl fmt::Display for Dot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "[{}:{}] {} -> {} | {:?}",
            self.name, self.line, self.source, self.dest, self.operation,
        )
    }
}

impl fmt::Display for Dots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut display = String::new();
        self.dotfiles
            .iter()
            .for_each(|dot| display.push_str(&format!("{dot}\n")));
        write!(f, "{display}")
    }
}

impl Dots {
    pub fn build(flags: Flags) -> Self {
        Self {
            filename: String::new(),
            flags,
            dotfiles: vec![],
        }
    }

    pub fn parse_file(&mut self, filename: &str) -> Result<(), String> {
        let filepath = Path::new(filename).canonicalize().unwrap();
        let file_contents = fs::read_to_string(filepath)
            .unwrap_or_else(|_| panic!("File {filename} does not exist"));
        match self.flags.file_format {
            FormatFile::OrgTable => {
                let skip_headers = if self.flags.headers { 2 } else { 0 };
                for (num, line) in file_contents.lines().skip(skip_headers).enumerate() {
                    let values = line.split('|').collect::<Vec<_>>();
                    if values.len() < 5 {
                        continue;
                    };

                    let dot = Dot::new(
                        num,       // Line number
                        values[1], // Name
                        values[2], // Source
                        values[3], // Dest
                        values[4], // Operation
                    );
                    self.dotfiles.push(dot);
                }
                self.filename = filename.to_string();
                Ok(())
            }
            FormatFile::Csv => Err(String::from("ERROR: FormatFile::Csv is not implemented yet.")),
            // _ => Err(format!("Invalid file type dotfiles declaration: {:?}", self.file_format)),
        }
    }

    fn format_error(filename: &String, line: usize, name: &String, message: &String) -> String {
        format!(
            "./{filename}:{line}:0 -> Dotfile `{name}`\n\t{message}"
        )
    }

    pub fn verify_dotfiles(&self) -> Result<(), String> {
        let errors: Vec<Result<(), String>> = self.dotfiles.iter()
            .map(|dot|{
                let mut err_str = String::new();
                let mut has_errors = false;
                let source_path = Path::new(&*(dot.source));
                let dest_path = Path::new(&*(dot.dest));
                let line = dot.line + {if self.flags.headers {2 + 1} else {1}} ;
                // Source path
                match dot.operation {
                    Op::Symfile => {
                        if !source_path.is_file() {
                            has_errors = true;
                            let error_msg = format!("Path to the source file {} is not valid", dot.source);
                            err_str.push_str(
                                Dots::format_error(&self.filename, line, &dot.name, &error_msg).as_str());
                        }
                    },
                    Op::Symdir => {
                        if !source_path.is_dir() {
                            has_errors = true;
                            let error_msg = format!("Path to the source directory {} is not valid", dot.source);
                            err_str.push_str(
                                Dots::format_error(&self.filename, line, &dot.name, &error_msg).as_str()
                            );
                        }
                    },
                    Op::Invalid => {
                        has_errors = true;
                        let error_msg = "Invalid operation. Only allowed `Symfile` or `Symdir` for files or directories respectively".to_string();
                        err_str.push_str(
                            Dots::format_error(&self.filename, line, &dot.name, &error_msg).as_str()
                        );
                    }
                }

                // Destination path
                if dest_path.parent().is_none() {
                    has_errors = true;
                    let error_msg = format!("Path to destination {} is not a valid", dot.dest);
                    err_str.push_str(
                        Dots::format_error(&self.filename, line, &dot.name, &error_msg).as_str()
                    );
                }
                if has_errors { Err(err_str) } else {Ok(())}

            }).collect::<Vec<_>>();

        let mut has_errors = false;
        for d in errors {
            if let Err(err) = d {
                has_errors = true;
                eprintln!("ERROR: {err}");
            }
        }
        if has_errors {
            Err("Dotfiles contains errors.".to_string())
        } else {
            Ok(())
        }
    }

    pub fn execute(&self) {
        self.dotfiles.iter().for_each(|dot| {
            let _ = dot.execute(&self.flags);
        });
    }
}

impl Dot {
    fn new(line: usize, name: &str, source: &str, dest: &str, op: &str) -> Self {
        let name = name.trim().to_string();
        let source = source.trim().to_string();
        let dest = dest.trim().to_string();
        let op = op.trim().to_string();

        let dest = if Path::new(dest.as_str()).is_absolute() {
            Dot::strip_dir(dest)
        }
        else {
            let dest = Dot::strip_dir(dest);
            format!("{}/{}", home_dir().unwrap().display(), dest)
        };
        

        let operation = match op.to_lowercase().as_str() {
            "symfile" => Op::Symfile,
            "symdir" => Op::Symdir,
            _ => Op::Invalid,
        };

        Self {
            line,
            name,
            source,
            dest,
            operation,
        }
    }

    fn execute(&self, flags: &Flags) -> Result<(), String> {
        // If the directory does not exist,
        let source_path = Path::new(&self.source).canonicalize().unwrap();
        let dest_path = Path::new(&self.dest);
        // Don't care about errors.
        // I just want the dirs to be created
        let _ = fs::create_dir_all(dest_path.parent().unwrap());
        if flags.force && dest_path.exists() {
            if let Ok(meta) = dest_path.metadata() {
                assert!(dest_path.exists(), "IMPOSSIBLE: Dest path exists here");
                if meta.is_dir() {
                    fs::remove_dir_all(dest_path).unwrap();
                } else if meta.is_file() || meta.is_symlink() {
                    fs::remove_file(dest_path).unwrap();
                } else {
                    panic!("WTF is this file {}", dest_path.display());
                }
            }
        }
        match symlink(source_path, dest_path) {
            Ok(()) => {
                println!("[LOG] Source {} to dest {}", &self.source, &self.dest,);
                Ok(())
            }
            Err(err) => {
                let err_msg = format!(
                    "Error {} while symlinking {} to {} ({:?})",
                    err, &self.source, &self.dest, self.operation
                );
                eprintln!(
                    "{}",
                    Dots::format_error(&String::new(), self.line, &self.name, &err_msg)
                );
                
                Err("Error while symlinking".to_string())
            }
        }
    }

    fn strip_dir(dir: String) -> String {
        let dir = if let Some(d) = &dir.strip_suffix("/") {
            if dir.len() > 1 {
                (*d).to_string()
            } else {
                dir
            }
        } else {
            dir
        };
        dir
    }
}
