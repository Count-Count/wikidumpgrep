// wikidumpgrep
//
// (C) 2020 Count Count
//
// Distributed under the terms of the MIT license.

use criterion::*;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("file-io");
    group
        .sample_size(10)
        .warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(140))
        .throughput(Throughput::Bytes(fs::metadata(get_dump_path()).unwrap().len()));

    static KB: usize = 1024;
    static MB: usize = KB * 1024;
    for buf_size in [8 * KB, 128 * KB, 1 * MB, 8 * MB, 128 * MB].iter() {
        group.bench_with_input(BenchmarkId::new("file-reading", buf_size), &buf_size, |b, &buf_size| {
            b.iter(|| test_dump_reading(*buf_size));
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn get_dump_path() -> PathBuf {
    let env_var =
        env::var("WIKIPEDIA_DUMPS_DIRECTORY").expect("WIKIPEDIA_DUMPS_DIRECTORY environment variable not set.");
    let dump_path = Path::new(env_var.as_str()).join(Path::new("dewiki-20200620-pages-articles-multistream.xml"));
    fs::metadata(&dump_path).expect("Dump file not found or inaccessible.");
    dump_path
}

fn test_dump_reading(buf_size: usize) {
    let dump_path = get_dump_path();
    let file = File::open(&dump_path).unwrap();
    let mut reader = BufReader::with_capacity(buf_size, file);
    loop {
        let read_buf = reader.fill_buf().unwrap();
        let length = read_buf.len();
        if length == 0 {
            break;
        }
        reader.consume(length);
    }
}
