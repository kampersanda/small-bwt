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

    #[arg(
        short = 'o',
        long,
        help = "Path to an output bwt file (if none, verification mode will be activated)"
    )]
    output_file: Option<String>,

    #[arg(short = 't', long, help = "Flag to add a special teriminator \\0")]
    teriminator: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let text = read_text(&args.input_file, args.teriminator)?;
    small_bwt::verify_terminator(&text).map_err(|e| {
        format!("Got error while verifying terminal character: {e} Consider using -t option.")
    })?;

    let builder = BwtBuilder::new(&text)?.verbose(true);
    let elapsed_ms = if let Some(output_file) = args.output_file.as_ref() {
        let now = Instant::now();
        let writer = BufWriter::new(File::create(output_file)?);
        builder.build(writer)?;
        now.elapsed().as_millis()
    } else {
        eprintln!("VERIFICATION MODE: The BWT will not be saved.");
        let now = Instant::now();
        let mut bwt = Vec::with_capacity(text.len());
        builder.build(&mut bwt)?;
        let elapsed_ms = now.elapsed().as_millis();
        let decoded = small_bwt::decode_bwt(&bwt)?;
        if decoded != text {
            eprintln!("ERROR: The decoded text is different from the original text. The system will be broken.");
        } else {
            eprintln!("NO PROBLEM: The decoded text is the same as the original text. The system will be fine.");
        }
        elapsed_ms
    };
    println!("Elapsed sec: {}", elapsed_ms as f64 / 1000.0);

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
