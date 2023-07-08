pub struct MsdRadixSorter<'a> {
    text: &'a [u8],
    suffixes: Vec<usize>,
    threshold: usize,
}

impl<'a> MsdRadixSorter<'a> {
    pub fn sort(text: &'a [u8], suffixes: Vec<usize>, threshold: usize) -> Vec<usize> {
        let n_suffixes = suffixes.len();
        let threshold = threshold.max(1);
        let mut sorter = Self {
            text,
            suffixes,
            threshold,
        };
        sorter.sort_range(0, n_suffixes, 0);
        sorter.suffixes
    }

    fn sort_range(&mut self, start: usize, end: usize, level: usize) {
        if end - start <= self.threshold {
            // Sorts small ranges with comparison sort.
            self.suffixes[start..end].sort_unstable_by(|&a, &b| {
                self.text[a..].cmp(&self.text[b..]).then_with(|| a.cmp(&b))
            });
            return;
        }

        {
            // Counts occurrences at this level.
            let mut counts = vec![0; 256];
            for i in start..end {
                let c = self.text[self.suffixes[i] + level];
                counts[c as usize] += 1;
            }

            // Computes cumulative sums
            for i in 1..256 {
                counts[i] += counts[i - 1];
            }

            // Bucket sort.
            let mut sorted = vec![0; end - start];
            for i in (start..end).rev() {
                let c = self.text[self.suffixes[i] + level];
                counts[c as usize] -= 1;
                sorted[counts[c as usize]] = self.suffixes[i];
            }
            for i in start..end {
                self.suffixes[i] = sorted[i - start];
            }
        }

        // Recursively sort each bucket.
        let mut i = start;
        while i < end {
            let c = self.text[self.suffixes[i] + level];
            let mut j = i + 1;
            while j < end && self.text[self.suffixes[j] + level] == c {
                j += 1;
            }
            self.sort_range(i, j, level + 1);
            i = j;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msd_radix_sorter_1() {
        let text = b"abracadabra$";
        let suffixes = (0..text.len()).collect();
        let suffixes = MsdRadixSorter::sort(text, suffixes, 1);
        assert_eq!(suffixes, vec![11, 10, 7, 0, 3, 5, 8, 1, 4, 6, 9, 2]);
    }

    #[test]
    fn test_msd_radix_sorter_2() {
        let text = b"abracadabra$";
        let suffixes = (0..text.len()).collect();
        let suffixes = MsdRadixSorter::sort(text, suffixes, 2);
        assert_eq!(suffixes, vec![11, 10, 7, 0, 3, 5, 8, 1, 4, 6, 9, 2]);
    }

    #[test]
    fn test_msd_radix_sorter_4() {
        let text = b"abracadabra$";
        let suffixes = (0..text.len()).collect();
        let suffixes = MsdRadixSorter::sort(text, suffixes, 4);
        assert_eq!(suffixes, vec![11, 10, 7, 0, 3, 5, 8, 1, 4, 6, 9, 2]);
    }

    #[test]
    fn test_msd_radix_sorter_part_1() {
        let text = b"abracadabra$";
        let suffixes = vec![1, 3, 4, 7, 10];
        let suffixes = MsdRadixSorter::sort(text, suffixes, 1);
        assert_eq!(suffixes, vec![10, 7, 3, 1, 4]);
    }

    #[test]
    fn test_msd_radix_sorter_part_2() {
        let text = b"abracadabra$";
        let suffixes = vec![1, 3, 4, 7, 10];
        let suffixes = MsdRadixSorter::sort(text, suffixes, 2);
        assert_eq!(suffixes, vec![10, 7, 3, 1, 4]);
    }
}
