use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use runx_contracts::generated_schema_artifacts;

struct Options {
    out_dir: PathBuf,
    check: bool,
}

type CliResult<T> = Result<T, std::io::Error>;

fn main() -> CliResult<()> {
    let options = parse_args()?;
    fs::create_dir_all(&options.out_dir)?;

    let (stale, orphans) = reconcile_schema_artifacts(&options)?;
    if stale.is_empty() && orphans.is_empty() {
        return Ok(());
    }

    report_schema_drift(&stale, &orphans)?;
    Err(schema_drift_error(&stale, &orphans))
}

fn reconcile_schema_artifacts(options: &Options) -> CliResult<(Vec<&'static str>, Vec<String>)> {
    let artifacts = generated_schema_artifacts();
    let expected_file_names = artifacts
        .iter()
        .map(|artifact| artifact.file_name)
        .collect::<BTreeSet<_>>();
    let mut stale = Vec::new();
    for artifact in artifacts {
        let path = options.out_dir.join(artifact.file_name);
        let generated = format!(
            "{}\n",
            serde_json::to_string_pretty(&artifact.schema).map_err(std::io::Error::other)?
        );
        if options.check {
            match fs::read_to_string(&path) {
                Ok(current) if current == generated => {}
                _ => stale.push(artifact.file_name),
            }
        } else {
            fs::write(path, generated)?;
        }
    }

    let orphans = orphan_schema_files(&options.out_dir, &expected_file_names)?;
    if !options.check {
        for file_name in orphans {
            fs::remove_file(options.out_dir.join(file_name))?;
        }
        return Ok((stale, Vec::new()));
    }

    Ok((stale, orphans))
}

fn report_schema_drift(stale: &[&str], orphans: &[String]) -> CliResult<()> {
    let mut stderr = std::io::stderr().lock();
    if !stale.is_empty() {
        writeln!(stderr, "Generated contract schemas are stale:")?;
        for file_name in stale {
            writeln!(stderr, "- {file_name}")?;
        }
    }

    if !orphans.is_empty() {
        writeln!(stderr, "Orphan contract schemas are present:")?;
        for file_name in orphans {
            writeln!(stderr, "- {file_name}")?;
        }
    }
    Ok(())
}

fn schema_drift_error(stale: &[&str], orphans: &[String]) -> std::io::Error {
    if stale.is_empty() {
        std::io::Error::other("orphan contract schemas are present")
    } else if orphans.is_empty() {
        std::io::Error::other("generated contract schemas are stale")
    } else {
        std::io::Error::other("generated contract schemas are stale or orphaned")
    }
}

fn orphan_schema_files(
    out_dir: &Path,
    expected_file_names: &BTreeSet<&'static str>,
) -> CliResult<Vec<String>> {
    let mut orphans = Vec::new();
    for entry in fs::read_dir(out_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if file_name.ends_with(".schema.json") && !expected_file_names.contains(file_name.as_ref())
        {
            orphans.push(file_name.into_owned());
        }
    }
    orphans.sort();
    Ok(orphans)
}

fn parse_args() -> CliResult<Options> {
    let mut out_dir: Option<PathBuf> = None;
    let mut check = false;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => {
                let value = args
                    .next()
                    .ok_or_else(|| std::io::Error::other("--out requires a directory"))?;
                out_dir = Some(PathBuf::from(value));
            }
            "--check" => check = true,
            other => {
                return Err(std::io::Error::other(format!(
                    "unsupported argument: {other}"
                )));
            }
        }
    }

    Ok(Options {
        out_dir: out_dir.ok_or_else(|| std::io::Error::other("--out is required"))?,
        check,
    })
}
