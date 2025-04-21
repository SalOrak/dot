use clap::{Parser};
use dirs::home_dir;
use std::env::current_dir;

mod dotfiles;
mod simplegit;

#[derive(Parser)]
#[command(version)]
#[command(about)]
#[command(long_about)]
struct Cli {

    #[arg(
        required=false,
        short='f',
        long = "force",
        default_value_t = false,
        help="Overrides files and directories. Default false",
    )]
    force: bool,

    #[arg(
        required=false,
        short='j',
        long = "headers",
        default_value_t = false,
        help="Indicates whether the dots file includes headers or not.",
    )]
    headers: bool,
    
    #[arg(
        required=false,
        short= None,
        long = "source-prefix",
        default_value_t = {format!("{}",current_dir().unwrap().display())},
        help="Specify the prefix path for source",
    )]
    source_prefix: String,
    
    #[arg(
        required=false,
        short= None,
        long = "dest-prefix",
        default_value_t = {format!("{}",home_dir().unwrap().display())},
        help="Specify the prefix path for destination",
    )]
    destination_prefix: String,

    #[arg(
        required=false,
        short='t',
        long = "file-format",
        default_value = "org",
        value_parser= ["org", "csv"],
        help="Specify the file format for dots declaration file.",
    )]
    file_format: String,

    #[arg(
        required=false,
        short='d',
        long = "dots",
        default_value = "./DOTS",
        help="Specify the dots declaration file.",
    )]
    filename: String,
    
    #[arg(
        required=false,
        short='u',
        long = "url",
        default_value = "",
        help="Specify the github url.",
    )]
    url: String,
}

fn main() {

    let cli = Cli::parse();

    if !cli.url.is_empty() {
        simplegit::clone_repo(cli.url.clone());
    }
            
    let flags: dotfiles::Flags = dotfiles::Flags::build(
        cli.file_format,
        cli.headers,
        cli.force,
    );
    
    let mut dots: dotfiles::Dots = dotfiles::Dots::build(flags).unwrap();
    
    dots.parse_file(cli.filename.as_str()).unwrap();
    if let Err(error) = dots.verify_dotfiles() {
        eprintln!("{}", error);
        return;
    }

    dots.execute();
}

