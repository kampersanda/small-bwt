use std::time::Duration;

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion, SamplingMode,
};

const SAMPLE_SIZE: usize = 10;
const WARM_UP_TIME: Duration = Duration::from_secs(5);
const MEASURE_TIME: Duration = Duration::from_secs(10);
const ENGLISH_10MB_ZST: &[u8] = include_bytes!("english.10MB.zst");

fn english_10mb_txt() -> Vec<u8> {
    zstd::decode_all(ENGLISH_10MB_ZST).unwrap()
}

fn criterion_bwt_english(c: &mut Criterion) {
    let mut group = c.benchmark_group("bwt_english");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(WARM_UP_TIME);
    group.measurement_time(MEASURE_TIME);
    group.sampling_mode(SamplingMode::Flat);
    let text = english_10mb_txt();
    let mut n = 1000;
    while n <= text.len() {
        perform_bwt(&mut group, &text[..n]);
        n *= 10;
    }
}

fn perform_bwt(group: &mut BenchmarkGroup<WallTime>, text: &[u8]) {
    let text = [text, &[0x00]].concat();
    let group_id = format!("small_bwt/n={}", text.len());
    group.bench_function(group_id, |b| {
        b.iter(|| {
            small_bwt::BwtBuilder::new(&text)
                .unwrap()
                .build(NullWriter)
                .unwrap()
        });
    });
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

criterion_group!(benches, criterion_bwt_english);
criterion_main!(benches);
