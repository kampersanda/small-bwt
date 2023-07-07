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

    #[arg(short = 't', long)]
    teriminator: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let text = read_text(&args.input_file, args.teriminator)?;
    small_bwt::verify_terminal_character(&text)?;

    let writer = BufWriter::new(File::create(&args.output_file)?);
    let builder = if let Some(chunk_size) = args.chunk_size {
        BwtBuilder::new(&text)?.chunk_size(chunk_size)?
    } else {
        BwtBuilder::new(&text)?
    };
    builder.verbose(true).build(writer)?;
    Ok(())
}

fn read_text(input_file: &str, teriminator: bool) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file: File = File::open(input_file)?;
    let n_bytes = file.metadata()?.len();
    let mut text = Vec::with_capacity(n_bytes as usize + if teriminator { 1 } else { 0 });
    file.read_to_end(&mut text)?;
    if teriminator {
        text.push(b'\0');
    }
    Ok(text)
}
