use std::fs;
use std::path::Path;
use std::fmt;

use dirs::home_dir;

#[cfg(target_os = "linux")]
use std::os::unix::fs::symlink as symlink;

#[cfg(target_os = "windows")]
use std::os::windows::fs::symlink as symlink;
#[derive(Debug)]
enum Op {
    SYMFILE,
    SYMDIR,
    INVALID,
}

#[derive(Debug)]
enum FormatFile {
    OrgTable,
    CSV,
}

impl FormatFile {
    fn parse(format_str: &str) -> Result<Self, String> {
        match format_str {
            "org" => Ok(FormatFile::OrgTable),
            "csv" => Ok(FormatFile::CSV),
            _ => Err(format!("Invalid format {}.", format_str)),
        }
    }

}
#[derive(Debug)]
struct Dot {
    line: usize,
    name: String,
    source: String,
    dest: String,
    operation: Op
}

#[allow(dead_code)]
pub struct Flags {
    headers: bool,
    force: bool,
    file_format: FormatFile,    
}

impl Flags {
    pub fn build(file_format: String, headers: bool, force: bool) -> Self {
        let file_format = FormatFile::parse(file_format.as_str()).unwrap();
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
    dots: Vec<Dot>,
}

impl fmt::Display for Dot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "[{}:{}] {} -> {} | {:?}",
            self.name,
            self.line,
            self.source,
            self.dest,
            self.operation,
        )
    }
}

impl fmt::Display for Dots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut display = String::from("");
        self.dots.iter().for_each(|dot| display.push_str(&*format!("{}\n", dot)));
        write!(f, "{}",  display)
    }
}

impl Dots {
    pub fn build(flags: Flags) -> Result<Self, String> {
        Ok(Self {
            filename: "".to_string(),
            flags,
            dots: vec![],
        })
    }

    pub fn parse_file(&mut self, filename: &str) -> Result<(), String> {
        let filepath = Path::new(filename).canonicalize().unwrap();
        let file_contents = fs::read_to_string(filepath).expect(&*format!("File {} does not exist", filename));
        match self.flags.file_format {
            FormatFile::OrgTable => {
                let skip_headers = if self.flags.headers { 2 } else { 0 };
                for (num,line) in file_contents.lines().skip(skip_headers).enumerate() {
                    let values = line.split("|").collect::<Vec<_>>();
                    if values.len() < 5 {continue};

                    let dot = Dot::new(
                        num,       // Line number
                        values[1], // Name
                        values[2], // Source
                        values[3], // Dest
                        values[4], // Operation
                    );
                    self.dots.push(dot);
                }
                self.filename = filename.to_string();
                Ok(())
            },
            FormatFile::CSV => todo!("ERROR: FormatFile::CSV is not implemented yet."),
            // _ => Err(format!("Invalid file type dotfiles declaration: {:?}", self.file_format)),
        }
    }

    fn format_error(filename: &String, line: usize, name: &String, message: &String) -> String {
        format!("./{}:{}:0 -> Dotfile `{}`\n\t{}",
            filename,
            line,
            name,
            message)
    }

    pub fn verify_dotfiles(&self) -> Result<(), String> {
        let errors: Vec<Result<(), String>> = self.dots.iter()
            .map(|dot|{
                let mut err_str = String::from("");
                let mut has_errors = false;
                let source_path = Path::new(&*(dot.source));
                let dest_path = Path::new(&*(dot.dest));
                let line = dot.line + {if self.flags.headers {2 + 1} else {0 + 1}} ;
                // Source path
                match dot.operation {
                    Op::SYMFILE => {
                        if !source_path.is_file() {
                            has_errors = true;
                            let error_msg = format!("Path to the source file {} is not valid", dot.source);
                            err_str.push_str(
                                Dots::format_error(&self.filename, line, &dot.name, &error_msg).as_str())
                        }
                    },
                    Op::SYMDIR => {
                        if !source_path.is_dir() {
                            has_errors = true;
                            let error_msg = format!("Path to the source directory {} is not valid", dot.source);
                            err_str.push_str(
                                Dots::format_error(&self.filename, line, &dot.name, &error_msg).as_str()
                            )
                        }
                    },
                    Op::INVALID => {
                        has_errors = true;
                        let error_msg = format!("Invalid operation. Only allowed `SYMFILE` or `SYMDIR` for files or directories respectively");
                        err_str.push_str(
                            Dots::format_error(&self.filename, line, &dot.name, &error_msg).as_str()
                        )
                    }
                    
                }

                // Destination path
                if dest_path.parent().is_none() {
                    has_errors = true;
                    let error_msg = format!("Path to destination {} is not a valid", dot.dest);
                    err_str.push_str(
                        Dots::format_error(&self.filename, line, &dot.name, &error_msg).as_str()
                    )
                }
                
                if !has_errors { Ok(()) } else {Err(err_str)}

            }).collect::<Vec<_>>();
        
        let mut has_errors = false;
        for d in errors {
            if let Err(err) = d {
                has_errors = true;
                eprintln!("ERROR: {}", err)
            }
        };
        
        if has_errors { Err("Dotfiles contains errors.".to_string())} else {Ok(())}
    }

    pub fn execute(&self) {
        self.dots.iter().for_each(|dot| {
            let _ = dot.execute(&self.flags);
        })
    }
}

impl Dot {
    fn new(line: usize,  name: &str, source: &str, dest: &str, op: &str) -> Self {
        let name = name.trim().to_string();
        let source = source.trim().to_string();
        let dest = dest.trim().to_string();
        let op = op.trim().to_string();

        let dest = match Path::new(dest.as_str()).is_absolute() {
            true => {
                Dot::strip_dir(dest)
            },
            false => {
                let dest = Dot::strip_dir(dest);
                format!("{}/{}", home_dir().unwrap().display(), dest)
            }
        };
        
        let operation = match op.to_lowercase().as_str() {
            "symfile" => Op::SYMFILE,
            "symdir"  => Op::SYMDIR,
            _ => Op::INVALID,
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
                assert!(dest_path.exists(), "IMPOSSIBLE: Dest path exists here");                if meta.is_dir() {
                    fs::remove_dir_all(dest_path).unwrap();
                } else if meta.is_file() || meta.is_symlink() {
                    fs::remove_file(dest_path).unwrap();
                }else {
                    panic!("WTF is this file {}", dest_path.display());
                }
            }
            
        }
        match symlink(source_path, dest_path) {
            Ok(_) => {
                println!("[LOG] Source {} to dest {}",
                    &self.source,
                    &self.dest,
                );
                Ok(())
            }
,
            Err(err) => {
                let err_msg = format!("Error {} while symlinking {} to {} ({:?})",
                    err, &self.source, &self.dest,
                    self.operation);
                eprintln!("{}", Dots::format_error(&"".to_string(), self.line, &self.name, &err_msg));
                Err("Error while symlinking".to_string())
            },
        }

    }

    fn strip_dir(dir: String) -> String {
        let dir = if let Some(d) = &dir.strip_suffix("/"){
            if dir.len() > 1 {d.to_string()} else {dir}
        } else {dir};
        dir
    }
}


