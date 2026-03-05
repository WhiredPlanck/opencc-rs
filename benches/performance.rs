use std::{
    env,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use opencc_rs::SimpleConverter;

fn initialize(config_name: &str) -> SimpleConverter {
    SimpleConverter::build(format!("{}.json", config_name)).unwrap()
}

fn read_text(filename: &str) -> String {
    let benchmark_data_dir = PathBuf::from("/home/panda/Projects/Rust/opencc-rs/test/benchmark");
    let data_path = benchmark_data_dir.join(filename);
    let file = File::open(data_path).unwrap();
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).unwrap();
    buffer
}

fn initialization(c: &mut Criterion) {
    env::set_current_dir("/usr/share/opencc").unwrap();

    let mut group = c.benchmark_group("BM_Initialization");
    for config_name in [
        "hk2s", "hk2t", "jp2t", "s2hk", "s2t", "s2tw", "s2twp", "t2hk", "t2jp", "t2s", "tw2s",
        "tw2sp", "tw2t",
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::from_parameter(config_name),
            config_name,
            |b, &config_name| {
                b.iter(|| initialize(config_name));
            },
        );
    }
}

fn convert_2m(c: &mut Criterion) {
    env::set_current_dir("/usr/share/opencc").unwrap();
    let converter = initialize("s2t");
    let text = read_text("zuozhuan.txt");
    c.bench_function("BM_Convert2M", |b| {
        b.iter(|| converter.convert(&text));
    });
}

fn convert(c: &mut Criterion) {
    env::set_current_dir("/usr/share/opencc").unwrap();

    let mut group = c.benchmark_group("BM_Convert");
    for iteration in [100, 1000, 10000, 100000].iter() {
        let text: String = (0..*iteration)
            .map(|i| format!("Open Chinese Convert 開放中文轉換{}\n", i))
            .collect();
        let converter = initialize("s2t");
        group.bench_with_input(BenchmarkId::from_parameter(iteration), iteration, |b, _| {
            b.iter(|| converter.convert(&text));
        });
    }
}

criterion_group!(benches, initialization, convert_2m, convert);
criterion_main!(benches);
