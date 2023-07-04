//!
//!
use anyhow::{anyhow, Result};

///
pub struct BwtBuilder<'a> {
    text: &'a [u8],
    chunk_size: usize,
    verbose: bool,
}

impl<'a> BwtBuilder<'a> {
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
            verbose: false,
        })
    }

    pub fn chunk_size(mut self, chunk_size: usize) -> Result<Self> {
        if chunk_size == 0 {
            return Err(anyhow!("chunk_size must be positive."));
        }
        self.chunk_size = chunk_size;
        Ok(self)
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn build(&self) -> Vec<u8> {
        assert!(!self.text.is_empty());
        assert_ne!(self.chunk_size, 0);

        let text = self.text;
        let chunk_size = self.chunk_size;
        let n_expected_cuts = text.len() / chunk_size;

        eprintln!("Text length: {:?} MiB", to_mib(text.len()));
        eprintln!("Chunk size: {:?} M", to_mb(chunk_size));
        eprintln!("Expected number of cuts: {:?}", n_expected_cuts);

        eprintln!("Generating cuts...");
        let cuts = CutGenerator::generate(text, chunk_size);
        eprintln!("Generating BWT...");
        bwt_from_cuts(text, &cuts)
    }
}

/// # Arguments
///
/// * `text` - The text to be transformed.
/// * `cuts` - Minimal set of prefixes that each prefix starts no more than b suffixes of `text`.
fn bwt_from_cuts(text: &[u8], cuts: &[Vec<u8>]) -> Vec<u8> {
    assert!(cuts[0].is_empty());

    let text = text.as_ref();
    let mut bwt = Vec::with_capacity(text.len());
    let mut chunks = vec![];

    for q in 1..=cuts.len() {
        eprintln!("Generating BWT: {}/{}", q, cuts.len());
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
            bwt.push(if j == 0 {
                *text.last().unwrap()
            } else {
                text[j - 1]
            });
        }
        chunks.clear();
    }
    bwt
}

///
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
                    eprintln!("Generating cuts: {:?}", self.cuts.len());
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
        let bwt = BwtBuilder::new(text.as_bytes()).unwrap().build();
        let bwt_str = String::from_utf8_lossy(&bwt);
        assert_eq!(bwt_str, "ard$rcaaaabb");
    }

    #[test]
    fn test_bwt_builder_3() {
        let text = "abracadabra$";
        let bwt = BwtBuilder::new(text.as_bytes())
            .unwrap()
            .chunk_size(3)
            .unwrap()
            .build();
        let bwt_str = String::from_utf8_lossy(&bwt);
        assert_eq!(bwt_str, "ard$rcaaaabb");
    }

    #[test]
    fn test_bwt_builder_4() {
        let text = "abracadabra$";
        let bwt = BwtBuilder::new(text.as_bytes())
            .unwrap()
            .chunk_size(4)
            .unwrap()
            .build();
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
        let bwt = bwt_from_cuts(text, cuts);
        let bwt_str = String::from_utf8_lossy(&bwt);
        assert_eq!(bwt_str, "ard$rcaaaabb");
    }

    #[test]
    fn test_bwt_from_cuts_4() {
        let text = b"abracadabra$";
        let cuts = &[b"".to_vec(), b"ab".to_vec(), b"b".to_vec(), b"r".to_vec()];
        let bwt = bwt_from_cuts(text, cuts);
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
