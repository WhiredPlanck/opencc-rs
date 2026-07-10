use std::{path::{Path, PathBuf}, sync::Arc};

use clap::{Parser, ValueEnum};
use opencc_rs::{AnyDict, Dict, MarisaDict, SerializableDict, TextDict};

#[derive(Parser)]
#[clap(version, about = "Open Chinese Convert (OpenCC) Dictionary Tool")]
struct Args {
    #[arg(short, long, help = "Output format")]
    to: DictFormat,

    #[arg(short, long, help = "Input format")]
    from: DictFormat,

    #[arg(short, long, help = "Path to output dictionary")]
    output: PathBuf,

    #[arg(short, long, help = "Path to input dictionary")]
    input: PathBuf
}

#[derive(Debug, Clone, ValueEnum)]
enum DictFormat {
    Text,
    Ocd2
}

fn load_dictionary(format: DictFormat, input_path: &Path) -> AnyDict {
    match format {
        DictFormat::Text => TextDict::new_from_path(input_path).unwrap(),
        DictFormat::Ocd2 => MarisaDict::new_from_path(input_path).unwrap()
    }
}

fn convert_dict(format: DictFormat, dict: &AnyDict) -> Box<dyn SerializableDict> {
    match format {
        DictFormat::Text => Box::new(TextDict::from_dict(dict)),
        DictFormat::Ocd2 => Box::new(MarisaDict::from_dict(dict))
    }
}

fn main() {
    let args = Args::parse();
    let dict_from = load_dictionary(args.from, &args.input);
    let dict_to = convert_dict(args.to, &dict_from);
    dict_to.serialize_to_path(&args.output).unwrap();
}
