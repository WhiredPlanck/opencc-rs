use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use clap::Parser;
use opencc_rs::{Config, Converter};
use tempfile::NamedTempFile;

#[derive(Parser)]
#[command(version, about = "Open Chinese Convert (OpenCC) Command Line Tool")]
struct Args {
    #[arg(short, long, default_value = "s2t.json", help = "Configuration file")]
    config: String,

    #[arg(short, long, help = "Write converted text to <OUTPUT> file")]
    output: Option<PathBuf>,

    #[arg(short, long, help = "Read original text from <INPUT> file")]
    input: Option<PathBuf>,

    #[arg(long, default_value = "false", help = "Disable flush for every line")]
    noflush: bool,

    #[arg(long, num_args = 0.., help = "Additional paths to locate config and dictionary files")]
    path: Vec<PathBuf>,
}

fn convert<R: BufRead, W: Write>(
    conveter: &Converter,
    reader: &mut R,
    writer: &mut W,
    noflush: bool,
) -> io::Result<()> {
    let mut line = String::new();
    loop {
        line.clear();
    
        let btyes_read = reader.read_line(&mut line)?;
        if btyes_read == 0 {
            break;
        }

        let converted = conveter.convert(&line);
        writer.write_all(converted.as_bytes())?;

        if !noflush {
            writer.flush()?;
        }
    }
    Ok(())
}

pub fn main() -> io::Result<()> {
    let args = Args::parse();

    let converter = Config::new()
        .paths(args.path)
        .argv0(env::args().next())
        .build(&args.config)
        .expect("Failed to build conveter");

    let stdin = io::stdin();
    let mut reader: Box<dyn BufRead> = match args.input {
        Some(path) => Box::new(BufReader::new(File::open(path)?)),
        None => Box::new(stdin.lock()),
    };

    if let Some(path) = args.output {
        let mut parent = path.parent().unwrap_or_else(|| Path::new(""));
        if parent.as_os_str().is_empty() {
            parent = Path::new(".");
        }
    
        let mut temp = NamedTempFile::new_in(parent)?;

        convert(&converter, &mut reader, &mut temp, true)?;

        temp.persist(&path)?;
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        convert(&converter, &mut reader, &mut handle, args.noflush)?;
    }
    Ok(())
}
