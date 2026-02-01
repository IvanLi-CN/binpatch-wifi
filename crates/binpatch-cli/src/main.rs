use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use binpatch_core::{PatchOptions, Profile};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "binpatch-wifi")]
#[command(about = "Wi‑Fi config patcher for firmware binaries (Rust core + profile TOML).")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Verify {
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        firmware: PathBuf,
    },
    Patch {
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        firmware: PathBuf,
        #[arg(long)]
        ssid: String,
        #[arg(long)]
        psk: String,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        select_index: Option<usize>,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Verify { profile, firmware } => cmd_verify(&profile, &firmware),
        Command::Patch {
            profile,
            firmware,
            ssid,
            psk,
            out,
            select_index,
        } => cmd_patch(
            &profile,
            &firmware,
            &ssid,
            &psk,
            out.as_deref(),
            select_index,
        ),
    }
}

fn cmd_verify(profile_path: &Path, firmware_path: &Path) -> anyhow::Result<()> {
    let profile_toml = fs::read_to_string(profile_path)
        .with_context(|| format!("read profile: {profile_path:?}"))?;
    let profile = Profile::parse_toml(&profile_toml)?;

    let firmware =
        fs::read(firmware_path).with_context(|| format!("read firmware: {firmware_path:?}"))?;

    let report = profile.verify(&firmware).map_err(|e| anyhow!(e))?;
    println!("valid_matches={}", report.valid.len());
    for (idx, c) in report.valid.iter().enumerate() {
        println!("- index={} offset=0x{:X}", idx, c.offset);
    }

    Ok(())
}

fn cmd_patch(
    profile_path: &Path,
    firmware_path: &Path,
    ssid: &str,
    psk: &str,
    out_path: Option<&Path>,
    select_index: Option<usize>,
) -> anyhow::Result<()> {
    let profile_toml = fs::read_to_string(profile_path)
        .with_context(|| format!("read profile: {profile_path:?}"))?;
    let profile = Profile::parse_toml(&profile_toml)?;

    let firmware =
        fs::read(firmware_path).with_context(|| format!("read firmware: {firmware_path:?}"))?;

    let patched = profile
        .patch(
            &firmware,
            PatchOptions {
                ssid: ssid.to_string(),
                psk: psk.to_string(),
                select_index,
            },
        )
        .map_err(|e| anyhow!(e))?;

    let out_path = out_path
        .map(PathBuf::from)
        .unwrap_or_else(|| default_out_path(firmware_path));

    fs::write(&out_path, patched).with_context(|| format!("write output: {out_path:?}"))?;
    println!("wrote {:?}", out_path);
    println!("ssid_bytes={}", ssid.len());
    println!("psk_bytes={}", psk.len());

    Ok(())
}

fn default_out_path(input: &Path) -> PathBuf {
    let Some(file_name) = input.file_name().and_then(|s| s.to_str()) else {
        return input.with_extension("wifi.bin");
    };

    let mut out = String::from(file_name);
    if let Some((stem, ext)) = file_name.rsplit_once('.') {
        out = format!("{stem}.wifi.{ext}");
    } else {
        out.push_str(".wifi");
    }

    input.with_file_name(out)
}
