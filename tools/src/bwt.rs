use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::io::Read;
use std::time::Instant;

use clap::Parser;
use small_bwt::BwtBuilder;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Constructs the BWT of the given text."
)]
struct Args {
    #[arg(short = 'i', long, help = "Path to an input text file")]
    input_file: String,

    #[arg(short = 'o', long, help = "Path to an output bwt file")]
    output_file: Option<String>,

    #[arg(short = 'c', long, help = "Optional parameter for chunk size")]
    chunk_size: Option<usize>,

    #[arg(
        short = 't',
        long,
        help = "Flag to add a special terminal character \\0"
    )]
    teriminator: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let text = read_text(&args.input_file, args.teriminator)?;
    small_bwt::verify_terminal_character(&text)?;

    let builder = if let Some(chunk_size) = args.chunk_size {
        BwtBuilder::new(&text)?.chunk_size(chunk_size)?
    } else {
        BwtBuilder::new(&text)?
    };
    let builder = builder.verbose(true);

    let now = Instant::now();
    if let Some(output_file) = args.output_file.as_ref() {
        let writer = BufWriter::new(File::create(output_file)?);
        builder.build(writer)?;
    } else {
        let writer = NullWriter;
        builder.build(writer)?;
    }
    println!("Elapsed sec: {}", now.elapsed().as_millis() as f64 / 1000.0);

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

struct NullWriter;

impl std::io::Write for NullWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
