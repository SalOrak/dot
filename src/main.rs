use clap::{Parser};
use dirs::home_dir;
use std::env::current_dir;
use std::process::exit;

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
        help="Overrides files and directories.",
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
        default_value = "DOTS",
        help="Specify the dots declaration file.",
    )]
    filename: String,
    
    #[arg(
        required=false,
        short='u',
        long = "url",
        default_value = "",
        help="[EXPERIMENTAL].Specify the github url.",
    )]
    url: String,

}


fn main() {

    let cli = Cli::parse();

    if !cli.url.is_empty() {
        simplegit::clone_repo(cli.url.as_str());
    }

    println!("[DEBUG]: Force status: {}", cli.force);
            
    let flags: dotfiles::Flags = dotfiles::Flags::build(
        cli.file_format.as_str(),
        cli.headers,
        cli.force,
        cli.source_prefix,
        cli.destination_prefix,
    );
    
    let mut dots: dotfiles::Dots = dotfiles::Dots::build(flags);
    
    match dots.parse_file(cli.filename.as_str()) {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        },
    };
    
    if let Err(error) = dots.verify_dotfiles() {
        eprintln!("{error}");
        exit(1);
    }

    dots.execute();
}

