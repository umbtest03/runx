use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use runx_contracts::generated_schema_artifacts;

struct Options {
    out_dir: PathBuf,
    check: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = parse_args()?;
    fs::create_dir_all(&options.out_dir)?;

    let mut stale = Vec::new();
    for artifact in generated_schema_artifacts() {
        let path = options.out_dir.join(artifact.file_name);
        let generated = format!("{}\n", serde_json::to_string_pretty(&artifact.schema)?);
        if options.check {
            match fs::read_to_string(&path) {
                Ok(current) if current == generated => {}
                _ => stale.push(artifact.file_name),
            }
        } else {
            fs::write(path, generated)?;
        }
    }

    if !stale.is_empty() {
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "Generated contract schemas are stale:")?;
        for file_name in stale {
            writeln!(stderr, "- {file_name}")?;
        }
        return Err("generated contract schemas are stale".into());
    }

    Ok(())
}

fn parse_args() -> Result<Options, Box<dyn Error>> {
    let mut out_dir: Option<PathBuf> = None;
    let mut check = false;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--out requires a directory".to_owned())?;
                out_dir = Some(PathBuf::from(value));
            }
            "--check" => check = true,
            other => return Err(format!("unsupported argument: {other}").into()),
        }
    }

    Ok(Options {
        out_dir: out_dir.ok_or_else(|| "--out is required".to_owned())?,
        check,
    })
}
