use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'i', long)]
    input_file: String,

    #[arg(short = 'o', long)]
    output_file: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let bwt = read_text(&args.input_file)?;
    let text = small_bwt::decode_bwt(&bwt)?;

    let mut writer = File::create(&args.output_file)?;
    writer.write_all(&text)?;

    Ok(())
}

fn read_text(input_file: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file: File = File::open(input_file)?;
    let mut text = Vec::new();
    file.read_to_end(&mut text)?;
    Ok(text)
}
