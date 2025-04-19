use std::fs;
use std::path::Path;
use std::fmt;

use dirs::home_dir;


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

#[derive(Debug)]
struct Dots {
    filename: String,
    headers: bool,
    file_type: FormatFile,
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
    fn new(headers: bool, file_type: &str) -> Result<Self, String> {
        let file_type = FormatFile::parse(file_type)?;
        Ok(Self {
            filename: "".to_string(),
            headers,
            file_type,
            dots: vec![],
        })
    }

    fn parse_file(&mut self, filename: &str) -> Result<(), String> {
        let filepath = Path::new(filename).canonicalize().unwrap();
        let file_contents = fs::read_to_string(filepath).expect(&*format!("File {} does not exist", filename));
        match self.file_type {
            FormatFile::OrgTable => {
                let skip_headers = if self.headers { 2 } else { 0 };
                for (num,line) in file_contents.lines().skip(skip_headers).enumerate() {
                    let values = line.split("|").collect::<Vec<_>>();
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
            // _ => Err(format!("Invalid file type dotfiles declaration: {:?}", self.file_type)),
        }
    }

    fn format_error(filename: &String, line: usize, name: &String, message: &String) -> String {
        format!("./{}:{}:0 -> Dotfile `{}`\n\t{}",
            filename,
            line,
            name,
            message)
    }

    fn verify_dotfiles(&self) {
        let errors: Vec<Result<(), String>> = self.dots.iter()
            .map(|dot|{
                let mut err_str = String::from("");
                let mut has_errors = false;
                let source_path = Path::new(&*(dot.source));
                let dest_path = Path::new(&*(dot.dest));
                let line = dot.line + {if self.headers {2 + 1} else {0 + 1}} ;
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
        for d in errors {
            if let Err(err) = d {
                eprintln!("ERROR: {}", err)
            }
        };
       
    }
}

impl Dot {
    fn new(line: usize,  name: &str, source: &str, dest: &str, op: &str) -> Self {
        let name = name.trim().to_string();
        let source = source.trim().to_string();
        let dest = dest.trim().to_string();
        let op = op.trim().to_string();

        let dest = match Path::new(dest.as_str()).is_absolute() {
            true => dest,
            false => {
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
}

fn main() {
    let filename = "DOTS";


    let _ = std::env::set_current_dir(Path::new("/home/hector/personal/dotfiles")).is_ok(); // TODO: Delete

    let mut dots: Dots = Dots::new(true, "org").unwrap();

    dots.parse_file(filename).unwrap();
    dots.verify_dotfiles();
    
}

