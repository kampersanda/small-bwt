use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::io::Read;

use clap::Parser;
use small_bwt::BwtBuilder;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'i', long)]
    input_file: String,

    #[arg(short = 'o', long)]
    output_file: String,

    #[arg(short = 'c', long)]
    chunk_size: Option<usize>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let text = read_text(&args.input_file)?;
    let writer = BufWriter::new(File::create(&args.output_file)?);
    let builder = if let Some(chunk_size) = args.chunk_size {
        BwtBuilder::new(&text)?.chunk_size(chunk_size)?
    } else {
        BwtBuilder::new(&text)?
    };
    builder.build(writer)?;
    Ok(())
}

fn read_text(input_file: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file: File = File::open(input_file)?;
    let n_bytes = file.metadata()?.len();
    let mut text = Vec::with_capacity(n_bytes as usize + 1);
    file.read_to_end(&mut text)?;
    text.push(b'\0');
    Ok(text)
}
