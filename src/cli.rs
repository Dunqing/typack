//! CLI entry-point logic, shared between the native binary and the napi binding.

#![expect(clippy::print_stdout, clippy::print_stderr, clippy::exit)]

use std::fs;
use std::path::PathBuf;

use bpaf::{Args, Bpaf};

use crate::{TypackBundler, TypackOptions};

/// A native TypeScript declaration (.d.ts) bundler built on Oxc crates.
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version)]
struct Cli {
    /// Module specifiers to keep external (repeatable)
    #[bpaf(long("external"), argument("SPEC"), many)]
    external: Vec<String>,

    /// Working directory (default: current directory)
    #[bpaf(long("cwd"), argument("DIR"), optional)]
    cwd: Option<PathBuf>,

    /// Generate source map (.d.ts.map)
    #[bpaf(long("sourcemap"), switch)]
    sourcemap: bool,

    /// Emit `export =` for single default export
    #[bpaf(long("cjs-default"), switch)]
    cjs_default: bool,

    /// Write output to file instead of stdout (single entry only)
    #[bpaf(short('o'), long("outfile"), argument("PATH"), optional)]
    outfile: Option<PathBuf>,

    /// Write per-entry outputs to directory (multiple entries)
    #[bpaf(long("outdir"), argument("DIR"), optional)]
    outdir: Option<PathBuf>,

    /// Entry .d.ts files to bundle
    #[bpaf(positional("ENTRY"), some("at least one entry file is required"))]
    input: Vec<String>,
}

/// Run the CLI with the given arguments (excluding argv[0] / the program name).
///
/// This function handles all output and exits the process when done.
///
/// # Panics
///
/// Panics if the current working directory cannot be determined.
pub fn run_cli(args: &[String]) -> ! {
    let cli = cli().run_inner(Args::from(args)).unwrap_or_else(|err| {
        err.print_message(100);
        std::process::exit(err.exit_code());
    });

    let cwd = cli
        .cwd
        .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));
    let cwd = cwd.canonicalize().unwrap_or_else(|e| {
        eprintln!("error: cannot resolve --cwd {}: {e}", cwd.display());
        std::process::exit(1);
    });

    let input: Vec<String> = cli
        .input
        .iter()
        .map(|entry| {
            let path = PathBuf::from(entry);
            let canonical = path.canonicalize().unwrap_or_else(|e| {
                eprintln!("error: cannot find entry file {entry}: {e}");
                std::process::exit(1);
            });
            canonical.to_string_lossy().to_string()
        })
        .collect();

    // Validate option combinations
    if input.len() > 1 && cli.outfile.is_some() {
        eprintln!("error: --outfile cannot be used with multiple entries; use --outdir instead");
        std::process::exit(1);
    }
    if cli.outfile.is_some() && cli.outdir.is_some() {
        eprintln!("error: --outfile and --outdir cannot be used together");
        std::process::exit(1);
    }

    let result = TypackBundler::bundle(&TypackOptions {
        input: input.clone(),
        external: cli.external,
        cwd: cwd.clone(),
        sourcemap: cli.sourcemap,
        cjs_default: cli.cjs_default,
    });

    match result {
        Ok(bundle) => {
            for warning in &bundle.warnings {
                eprintln!("warning: {warning}");
            }

            if let Some(outfile) = &cli.outfile {
                // Single entry with --outfile
                let output = &bundle.outputs[0];
                write_output_file(outfile, &output.code, output.map.as_ref());
            } else if let Some(outdir) = &cli.outdir {
                // Multiple entries with --outdir
                fs::create_dir_all(outdir).unwrap_or_else(|e| {
                    eprintln!("error: cannot create directory {}: {e}", outdir.display());
                    std::process::exit(1);
                });
                for (entry, output) in input.iter().zip(&bundle.outputs) {
                    let entry_path = PathBuf::from(entry);
                    // Preserve relative directory structure under outdir
                    let relative = entry_path
                        .strip_prefix(&cwd)
                        .unwrap_or(&entry_path);
                    let stem = relative
                        .file_stem()
                        .unwrap_or_else(|| relative.as_os_str())
                        .to_string_lossy();
                    // Strip .d from stems like "index.d" (from "index.d.ts")
                    let stem = stem.strip_suffix(".d").unwrap_or(&stem);
                    let out_path = if let Some(parent) = relative.parent() {
                        outdir.join(parent).join(format!("{stem}.d.ts"))
                    } else {
                        outdir.join(format!("{stem}.d.ts"))
                    };
                    write_output_file(&out_path, &output.code, output.map.as_ref());
                }
            } else {
                // Print to stdout
                for output in &bundle.outputs {
                    println!("{}", output.code);
                }
                if bundle.outputs.iter().any(|o| o.map.is_some()) {
                    eprintln!(
                        "warning: --sourcemap without --outfile/--outdir; source map not written"
                    );
                }
            }

            std::process::exit(0);
        }
        Err(diagnostics) => {
            for diag in &diagnostics {
                eprintln!("error: {diag}");
            }
            std::process::exit(1);
        }
    }
}

fn write_output_file(path: &PathBuf, code: &str, map: Option<&oxc_sourcemap::SourceMap>) {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).unwrap_or_else(|e| {
            eprintln!("error: cannot create directory {}: {e}", parent.display());
            std::process::exit(1);
        });
    }
    fs::write(path, code).unwrap_or_else(|e| {
        eprintln!("error: cannot write {}: {e}", path.display());
        std::process::exit(1);
    });

    if let Some(map) = map {
        let map_path = PathBuf::from(format!("{}.map", path.display()));
        let json = map.to_json_string();
        fs::write(&map_path, json).unwrap_or_else(|e| {
            eprintln!("error: cannot write {}: {e}", map_path.display());
            std::process::exit(1);
        });
    }
}
