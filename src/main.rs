use clap::{Parser};

mod dotfiles;

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
}

fn main() {

    let cli = Cli::parse();

    println!("Force       = {}", cli.force);
    println!("Headers     = {}", cli.headers);
    println!("File Format = {}", cli.file_format);
    println!("Filename    = {}", cli.filename);

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

