//! Implementation of the BWT construction algorithm in small space,
//! described in Algorithm 11.8 of the book:
//! [Compact Data Structures - A Practical Approach](https://users.dcc.uchile.cl/~gnavarro/CDSbook/),
//! Gonzalo Navarro, 2016.
use std::io::Write;

use anyhow::{anyhow, Result};

/// Verifies that the text ends with the smallest special character.
///
/// # Arguments
///
/// * `text` - The text to be verified.
///
/// # Errors
///
/// An error is returned if `text` is empty or does not end with the smallest special character.
pub fn verify_terminal_symbol(text: &[u8]) -> Result<()> {
    if text.is_empty() {
        return Err(anyhow!("text must not be empty."));
    }
    let smallest = *text.last().unwrap();
    for (i, &c) in text[..text.len() - 1].iter().enumerate() {
        if c <= smallest {
            return Err(anyhow!(
                "text must have the smallest special character only at the end, but found {c:?} at position {i}."
            ));
        }
    }
    Ok(())
}

/// BWT builder in small space.
///
/// # Specifications
///
/// This assumes that the text ends with the smallest special character (e.g., `\0`).
/// Given an unexpected text, the behavior is undefined.
/// If you want to verify the text, use [`verify_terminal_symbol`].
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use small_bwt::BwtBuilder;
///
/// let text = "abracadabra$";
/// let mut bwt = vec![];
/// BwtBuilder::new(text.as_bytes())?.build(&mut bwt)?;
/// let bwt_str = String::from_utf8_lossy(&bwt);
/// assert_eq!(bwt_str, "ard$rcaaaabb");
/// # Ok(())
/// # }
/// ```
pub struct BwtBuilder<'a> {
    text: &'a [u8],
    chunk_size: usize,
    progress: Progress,
}

impl<'a> BwtBuilder<'a> {
    /// Creates a new builder.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to be transformed.
    ///
    /// # Errors
    ///
    /// An error is returned if `text` is empty.
    pub fn new(text: &'a [u8]) -> Result<Self> {
        if text.is_empty() {
            return Err(anyhow!("text must not be empty."));
        }
        let n = text.len() as f64;
        let chunk_size = (n / n.log2()).ceil() as usize;
        let chunk_size = chunk_size.max(1);
        Ok(Self {
            text,
            chunk_size,
            progress: Progress::new(false),
        })
    }

    /// Sets the chunk size.
    ///
    /// # Arguments
    ///
    /// * `chunk_size` - The chunk size.
    ///
    /// # Errors
    ///
    /// An error is returned if `chunk_size` is zero.
    pub fn chunk_size(mut self, chunk_size: usize) -> Result<Self> {
        if chunk_size == 0 {
            return Err(anyhow!("chunk_size must be positive."));
        }
        self.chunk_size = chunk_size;
        Ok(self)
    }

    /// Sets the verbosity.
    /// If `verbose` is `true`, the progress is printed to stderr.
    ///
    /// # Arguments
    ///
    /// * `verbose` - The verbosity.
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.progress = Progress::new(verbose);
        self
    }

    /// Builds the BWT and writes it to `wrt`.
    ///
    /// # Arguments
    ///
    /// * `wrt` - The writer to write the BWT.
    ///
    /// # Errors
    ///
    /// An error is returned if `wrt` returns an error.
    pub fn build<W: Write>(&self, wrt: W) -> Result<()> {
        assert!(!self.text.is_empty());
        assert_ne!(self.chunk_size, 0);

        let text = self.text;
        let chunk_size = self.chunk_size;
        let n_expected_cuts = text.len() / chunk_size;

        self.progress
            .print(&format!("Text length: {:?} MiB", to_mib(text.len())));
        self.progress
            .print(&format!("Chunk size: {:?} M", to_mb(chunk_size)));
        self.progress
            .print(&format!("Expected number of cuts: {:?}", n_expected_cuts));

        self.progress.print("Generating cuts...");
        let cuts = CutGenerator::generate(text, chunk_size);
        self.progress
            .print(&format!("Actual number of cuts: {:?}", cuts.len()));

        bwt_from_cuts(text, &cuts, wrt, &self.progress)
    }
}

fn bwt_from_cuts<W: Write>(
    text: &[u8],
    cuts: &[Vec<u8>],
    mut wrt: W,
    progress: &Progress,
) -> Result<()> {
    assert!(cuts[0].is_empty());

    let text = text.as_ref();
    let mut chunks = vec![];

    for q in 1..=cuts.len() {
        progress.print(&format!("Generating BWT: {}/{}", q, cuts.len()));
        if q < cuts.len() {
            let cut_p = cuts[q - 1].as_slice();
            let cut_q = cuts[q].as_slice();
            for j in 0..text.len() {
                let suffix = &text[j..];
                if cut_p < suffix && suffix <= cut_q {
                    chunks.push(j);
                }
            }
        } else {
            let cut_p = cuts[q - 1].as_slice();
            for j in 0..text.len() {
                let suffix = &text[j..];
                if cut_p < suffix {
                    chunks.push(j);
                }
            }
        }
        // TODO: Use radix sort.
        chunks.sort_by(|&a, &b| text[a..].cmp(&text[b..]));
        for &j in &chunks {
            let c = if j == 0 {
                *text.last().unwrap()
            } else {
                text[j - 1]
            };
            wrt.write_all(&[c])?;
        }
        chunks.clear();
    }
    Ok(())
}

struct CutGenerator<'a> {
    text: &'a [u8],
    chunk_size: usize,
    cuts: Vec<Vec<u8>>,
    lens: Vec<usize>,
}

impl<'a> CutGenerator<'a> {
    fn generate(text: &'a [u8], chunk_size: usize) -> Vec<Vec<u8>> {
        let mut builder = Self {
            text,
            chunk_size,
            cuts: vec![vec![]],
            lens: vec![],
        };
        builder.expand(vec![]);
        builder.cuts
    }

    fn expand(&mut self, mut cut: Vec<u8>) {
        let freqs = symbol_freqs(self.text, &cut);
        cut.push(0); // dummy last symbol
        for (symbol, &freq) in freqs.iter().enumerate() {
            if freq == 0 {
                continue;
            }
            *cut.last_mut().unwrap() = symbol as u8;
            if freq <= self.chunk_size {
                if self.lens.is_empty() || *self.lens.last().unwrap() + freq > self.chunk_size {
                    self.cuts.push(vec![]);
                    self.lens.push(0);
                }
                *self.cuts.last_mut().unwrap() = cut.clone();
                *self.lens.last_mut().unwrap() += freq;
            } else {
                self.expand(cut.clone());
            }
        }
    }
}

/// Computes the frequencies of symbols following cut in text.
fn symbol_freqs(text: &[u8], cut: &[u8]) -> Vec<usize> {
    let cut = cut.as_ref();
    let mut freqs = vec![0; 256];
    for j in cut.len()..text.len() {
        let i = j - cut.len();
        if cut == &text[i..j] {
            freqs[text[j] as usize] += 1;
        }
    }
    freqs
}

struct Progress {
    verbose: bool,
}

impl Progress {
    fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    fn print(&self, msg: &str) {
        if self.verbose {
            eprintln!("{}", msg);
        }
    }
}

fn to_mb(bytes: usize) -> f64 {
    bytes as f64 / 1000.0 / 1000.0
}

fn to_mib(bytes: usize) -> f64 {
    bytes as f64 / 1024.0 / 1024.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bwt_builder() {
        let text = "abracadabra$";
        let mut bwt = vec![];
        BwtBuilder::new(text.as_bytes())
            .unwrap()
            .build(&mut bwt)
            .unwrap();
        let bwt_str = String::from_utf8_lossy(&bwt);
        assert_eq!(bwt_str, "ard$rcaaaabb");
    }

    #[test]
    fn test_bwt_builder_3() {
        let text = "abracadabra$";
        let mut bwt = vec![];
        BwtBuilder::new(text.as_bytes())
            .unwrap()
            .chunk_size(3)
            .unwrap()
            .build(&mut bwt)
            .unwrap();
        let bwt_str = String::from_utf8_lossy(&bwt);
        assert_eq!(bwt_str, "ard$rcaaaabb");
    }

    #[test]
    fn test_bwt_builder_4() {
        let text = "abracadabra$";
        let mut bwt = vec![];
        BwtBuilder::new(text.as_bytes())
            .unwrap()
            .chunk_size(4)
            .unwrap()
            .build(&mut bwt)
            .unwrap();
        let bwt_str = String::from_utf8_lossy(&bwt);
        assert_eq!(bwt_str, "ard$rcaaaabb");
    }

    #[test]
    fn test_bwt_from_cuts_3() {
        let text = b"abracadabra$";
        let cuts = &[
            b"".to_vec(),
            b"a$".to_vec(),
            b"ac".to_vec(),
            b"b".to_vec(),
            b"d".to_vec(),
            b"r".to_vec(),
        ];
        let mut bwt = vec![];
        bwt_from_cuts(text, cuts, &mut bwt, &Progress::new(false)).unwrap();
        let bwt_str = String::from_utf8_lossy(&bwt);
        assert_eq!(bwt_str, "ard$rcaaaabb");
    }

    #[test]
    fn test_bwt_from_cuts_4() {
        let text = b"abracadabra$";
        let cuts = &[b"".to_vec(), b"ab".to_vec(), b"b".to_vec(), b"r".to_vec()];
        let mut bwt = vec![];
        bwt_from_cuts(text, cuts, &mut bwt, &Progress::new(false)).unwrap();
        let bwt_str = String::from_utf8_lossy(&bwt);
        assert_eq!(bwt_str, "ard$rcaaaabb");
    }

    #[test]
    fn test_symbol_freqs() {
        let text = b"abracadabra$";
        let cut = b"ra";
        let freqs = symbol_freqs(text, cut);
        let mut expected = vec![0; 256];
        expected[b'$' as usize] = 1;
        expected[b'c' as usize] = 1;
        assert_eq!(freqs, expected);
    }

    #[test]
    fn test_symbol_freqs_empty() {
        let text = b"abracadabra$";
        let cut = b"";
        let freqs = symbol_freqs(text, cut);
        let mut expected = vec![0; 256];
        expected[b'$' as usize] = 1;
        expected[b'a' as usize] = 5;
        expected[b'b' as usize] = 2;
        expected[b'c' as usize] = 1;
        expected[b'd' as usize] = 1;
        expected[b'r' as usize] = 2;
        assert_eq!(freqs, expected);
    }
}
