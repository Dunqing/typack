//! CLI binary for the DTS bundler.
//!
//! Provides a `bpaf`-based command-line interface that wraps [`TypackBundler::bundle`]
//! with options for external specifiers, source maps, CJS default exports, and
//! file output.

#![expect(clippy::print_stdout, clippy::print_stderr)]

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use bpaf::Bpaf;
use typack::{TypackBundler, TypackOptions};

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

    /// Write output to file instead of stdout
    #[bpaf(short('o'), long("outfile"), argument("PATH"), optional)]
    outfile: Option<PathBuf>,

    /// Entry .d.ts files to bundle
    #[bpaf(positional("ENTRY"), some("at least one entry file is required"))]
    input: Vec<String>,
}

fn main() -> ExitCode {
    let cli = cli().run();

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

    let result = TypackBundler::bundle(&TypackOptions {
        input,
        external: cli.external,
        cwd,
        sourcemap: cli.sourcemap,
        cjs_default: cli.cjs_default,
    });

    match result {
        Ok(bundle) => {
            for warning in &bundle.warnings {
                eprintln!("warning: {warning}");
            }

            if let Some(outfile) = &cli.outfile {
                if let Some(parent) = outfile.parent()
                    && !parent.as_os_str().is_empty()
                {
                    fs::create_dir_all(parent).unwrap_or_else(|e| {
                        eprintln!("error: cannot create directory {}: {e}", parent.display());
                        std::process::exit(1);
                    });
                }
                fs::write(outfile, &bundle.code).unwrap_or_else(|e| {
                    eprintln!("error: cannot write {}: {e}", outfile.display());
                    std::process::exit(1);
                });

                if let Some(map) = &bundle.map {
                    let map_path = PathBuf::from(format!("{}.map", outfile.display()));
                    let json = map.to_json_string();
                    fs::write(&map_path, json).unwrap_or_else(|e| {
                        eprintln!("error: cannot write {}: {e}", map_path.display());
                        std::process::exit(1);
                    });
                }
            } else {
                println!("{}", bundle.code);
                if bundle.map.is_some() {
                    eprintln!("warning: --sourcemap without --outfile; source map not written");
                }
            }

            ExitCode::SUCCESS
        }
        Err(diagnostics) => {
            for diag in &diagnostics {
                eprintln!("error: {diag}");
            }
            ExitCode::FAILURE
        }
    }
}
