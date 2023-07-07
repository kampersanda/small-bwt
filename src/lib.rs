//! Implementation of the BWT construction algorithm in small space,
//! described in Algorithm 11.8 of the book:
//! [Compact Data Structures - A Practical Approach](https://users.dcc.uchile.cl/~gnavarro/CDSbook/),
//! Gonzalo Navarro, 2016.
#![deny(missing_docs)]

use std::io::Write;

use anyhow::{anyhow, Result};

/// BWT builder in small space.
///
/// # Specifications
///
/// This assumes that the smallest character appears only at the end of the text.
/// Given an unexpected text, the behavior is undefined.
/// If you want to verify the text, use [`verify_terminal_character`].
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
    /// # Default value
    ///
    /// `ceil(n / log2(n))`, where `n` is the text length.
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
    ///
    /// # Default value
    ///
    /// `false`
    pub const fn verbose(mut self, verbose: bool) -> Self {
        self.progress = Progress::new(verbose);
        self
    }

    /// Builds the BWT and writes it to `wrt`.
    ///
    /// # Specifications
    ///
    /// This assumes that the smallest character appears only at the end of the text.
    /// Given an unexpected text, the behavior is undefined.
    /// If you want to verify the text, use [`verify_terminal_character`].
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
    let mut chunks = vec![];
    for q in 1..=cuts.len() {
        progress.print(&format!("Generating BWT: {}/{}", q, cuts.len()));
        progress.print(&format!("Length of the cut: {:?}", cuts[q - 1].len()));

        let cut_p = cuts[q - 1].as_slice();
        if q < cuts.len() {
            let cut_q = cuts[q].as_slice();
            for j in 0..text.len() {
                let suffix = &text[j..];
                if cut_p < suffix && suffix <= cut_q {
                    chunks.push(j);
                }
            }
        } else {
            for j in 0..text.len() {
                let suffix = &text[j..];
                if cut_p < suffix {
                    chunks.push(j);
                }
            }
        }

        progress.print(&format!("Length of chunks: {:?}", chunks.len()));

        // TODO: Use radix sort.
        chunks.sort_unstable_by(|&a, &b| text[a..].cmp(&text[b..]));
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
    const fn new(verbose: bool) -> Self {
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

/// Verifies that the smallest character appears only at the end of the text.
///
/// # Arguments
///
/// * `text` - The text to be verified.
///
/// # Errors
///
/// An error is returned if the smallest character does not appear only at the end of the text.
///
/// # Examples
///
/// ```
/// use small_bwt::verify_terminal_character;
///
/// let text = "abracadabra$";
/// let result = verify_terminal_character(text.as_bytes());
/// assert!(result.is_ok());
///
/// let text = "abrac$dabra$";
/// let result = verify_terminal_character(text.as_bytes());
/// assert!(result.is_err());
/// ```
pub fn verify_terminal_character(text: &[u8]) -> Result<()> {
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

/// Decodes the original text from a given BWT.
///
/// It runs in `O(n^2)` time and `O(n log s)` bits of space,
/// where `n` is the length of the text and `s` is the size of the alphabet.
///
/// # Arguments
///
/// * `bwt` - The Burrows-Wheeler transform of a text.
///
/// # Errors
///
/// An error is returned if the Burrows-Wheeler transform is invalid.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use small_bwt::decode_bwt;
///
/// let bwt = "ard$rcaaaabb";
/// let decoded = decode_bwt(bwt.as_bytes())?;
/// assert_eq!(decoded, "abracadabra$".as_bytes());
/// # Ok(())
/// # }
/// ```
pub fn decode_bwt(bwt: &[u8]) -> Result<Vec<u8>> {
    let counts = {
        let mut counts = vec![0; 256];
        for &c in bwt {
            counts[c as usize] += 1;
        }
        counts
    };

    let ranks = {
        let mut ranks = vec![0; 256];
        let mut rank = 0;
        for i in 0..256 {
            ranks[i] = rank;
            rank += counts[i];
        }
        ranks
    };

    let terminator = counts.iter().position(|&c| c != 0).unwrap();
    if counts[terminator] != 1 {
        return Err(anyhow!(
            "bwt must have exactly one terminator character, but found {:x} {} times.",
            terminator,
            counts[terminator]
        ));
    }

    let terminator = terminator as u8;

    let mut decoded = Vec::with_capacity(bwt.len());
    decoded.push(terminator);

    let mut i = 0;
    while bwt[i] != terminator {
        decoded.push(bwt[i]);
        i = ranks[bwt[i] as usize] + bwt[..i].iter().filter(|&&c| c == bwt[i]).count();
    }
    decoded.reverse();

    Ok(decoded)
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
