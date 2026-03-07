use std::path::PathBuf;

use napi::Result;
use napi_derive::napi;
use serde::Serialize;
use typack::{TypackBundler, TypackOptions};

#[napi(object)]
pub struct BundleDtsOptions {
    pub input: Vec<String>,
    pub external: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub sourcemap: Option<bool>,
    pub cjs_default: Option<bool>,
}

#[napi(object)]
#[derive(Serialize)]
pub struct BundleDtsDiagnostic {
    pub message: String,
    pub file: Option<String>,
    pub span: Option<Vec<u32>>,
    pub severity: String,
}

#[napi(object)]
pub struct BundleDtsOutput {
    pub code: String,
    pub map: Option<String>,
}

#[napi(object)]
pub struct BundleDtsResult {
    pub outputs: Vec<BundleDtsOutput>,
    pub warnings: Vec<BundleDtsDiagnostic>,
}

fn bundle_impl(options: BundleDtsOptions) -> Result<BundleDtsResult> {
    let result = TypackBundler::bundle(&TypackOptions {
        input: options.input,
        external: options.external.unwrap_or_default(),
        cwd: options.cwd.map_or_else(|| PathBuf::from("."), PathBuf::from),
        sourcemap: options.sourcemap.unwrap_or(false),
        cjs_default: options.cjs_default.unwrap_or(false),
    });

    match result {
        Ok(bundle) => Ok(BundleDtsResult {
            outputs: bundle
                .outputs
                .into_iter()
                .map(|output| BundleDtsOutput {
                    code: output.code,
                    map: output.map.map(|map| map.to_json_string()),
                })
                .collect(),
            warnings: bundle
                .warnings
                .into_iter()
                .map(|warning| BundleDtsDiagnostic {
                    message: warning.to_string(),
                    file: None,
                    span: None,
                    severity: "warning".to_string(),
                })
                .collect(),
        }),
        Err(diagnostics) => {
            let errors = diagnostics
                .into_iter()
                .map(|diagnostic| BundleDtsDiagnostic {
                    message: diagnostic.to_string(),
                    file: None,
                    span: None,
                    severity: "error".to_string(),
                })
                .collect::<Vec<_>>();
            let reason = serde_json::to_string(&errors)
                .unwrap_or_else(|_| "[{\"message\":\"unknown native error\"}]".to_string());
            Err(napi::Error::from_reason(reason))
        }
    }
}

#[napi]
pub fn bundle(options: BundleDtsOptions) -> Result<BundleDtsResult> {
    bundle_impl(options)
}

/// Run the CLI with the given argv (pass `process.argv`; argv[0] and argv[1] are skipped).
///
/// All output is written to stdout/stderr and the process exits when done.
#[cfg(feature = "cli")]
#[napi]
pub fn cli(argv: Vec<String>) {
    let args: Vec<String> = argv.into_iter().skip(2).collect();
    typack::cli::run_cli(&args);
}
