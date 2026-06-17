mod download;

use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Instant;

use clap::Parser;
use colored::{Color, Colorize};
use eyre::{Context, Result, bail};
use zip::ZipArchive;

use download::download_sb3;
use sb2gs_decompiler::{DiskFS, FS, decompile_assets, decompile_code, extract_asset_files};

#[derive(Parser)]
#[command(about = "Decompile Scratch projects into goboscript projects.")]
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
    #[arg(long)]
    overwrite: bool,
    #[arg(long)]
    id: Option<String>,
    #[arg(long)]
    verify: bool,
}

fn main() -> ExitCode {
    if let Err(error) = color_eyre::install() {
        eprintln!("{error:?}");
        return ExitCode::FAILURE;
    }
    std::panic::set_hook(Box::new(|info| {
        eprintln!(
            "{info}\n{}\nopen an issue at {}",
            "sb2gs is cooked".red().bold(),
            "https://github.com/aspizu/sb2gs/issues".cyan()
        );
    }));
    let begin = Instant::now();
    let result = run();
    let color = if result.is_ok() {
        Color::Green
    } else {
        Color::Red
    };
    eprintln!(
        "{} in {:?}",
        "Finished".color(color).bold(),
        begin.elapsed()
    );
    if let Err(error) = result {
        eprintln!("{error:?}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<()> {
    let args = Args::parse();
    if args
        .input
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("sb3")
    {
        bail!("input must be a `.sb3` file.");
    }
    let output = determine_output_path(&args.input, args.output.as_deref(), args.overwrite)?;
    if let Some(id) = args.id.as_deref()
        && !args.input.exists()
    {
        download_sb3(id, &args.input)?;
    }
    let file = File::open(&args.input).wrap_err("failed to open input")?;
    let mut archive = ZipArchive::new(file).wrap_err("failed to read sb3 zip")?;
    prepare_output_dir(&output)?;
    let assets = decompile_assets(&mut archive)?;
    let mut fs = DiskFS::new(&output);
    for (path, bytes) in decompile_code(&assets)? {
        let text = String::from_utf8(bytes).wrap_err("failed to decode generated code")?;
        fs.write_text_file(&path, &text)
            .wrap_err("failed to write generated code")?;
    }
    extract_asset_files(&mut archive, &assets, &output.join("assets"))?;
    if args.verify {
        verify(&output)?;
    }
    Ok(())
}

fn determine_output_path(input: &Path, output: Option<&Path>, overwrite: bool) -> Result<PathBuf> {
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| input.with_extension(""));
    if output.exists() && !overwrite {
        bail!("output directory already exists. (use --overwrite to overwrite)");
    }
    Ok(output)
}

fn prepare_output_dir(output: &Path) -> Result<()> {
    match fs::remove_dir_all(output) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).wrap_err("failed to remove output directory"),
    }
    fs::create_dir_all(output).wrap_err("failed to create output directory")
}

fn verify(project: &Path) -> Result<()> {
    let status = Command::new("goboscript").arg("b").arg(project).status();
    let status = status.wrap_err("goboscript executable not found. Installation instructions: https://aspiz.uk/goboscript/docs/install.html")?;
    if !status.success() {
        bail!("goboscript failed to compile the decompiled code.");
    }
    Ok(())
}
