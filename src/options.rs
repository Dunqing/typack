//! Configuration options for the DTS bundler.

use std::path::PathBuf;

/// Options for the DTS bundler.
pub struct TypackOptions {
    /// Entry module ids/paths to bundle.
    pub input: Vec<String>,
    /// Module specifiers to keep as external imports.
    /// Unresolvable specifiers that are not in this list will produce errors.
    pub external: Vec<String>,
    /// Working directory. Region marker paths are relative to this.
    pub cwd: PathBuf,
    /// Generate source map (`.d.ts.map`).
    /// When true, maps each declaration in the bundled output back to its
    /// original position in the source `.d.ts` file.
    pub sourcemap: bool,
    /// Emit `export =` for single default export output.
    pub cjs_default: bool,
    /// Path to a `tsconfig.json` file. When set, IsolatedDeclarations config
    /// (`isolatedDeclarations`, `stripInternal`) is read from this file instead
    /// of auto-discovering by walking up from `cwd`.
    pub tsconfig: Option<PathBuf>,
}

impl Default for TypackOptions {
    fn default() -> Self {
        Self {
            input: Vec::new(),
            external: Vec::new(),
            cwd: PathBuf::from("."),
            sourcemap: false,
            cjs_default: false,
            tsconfig: None,
        }
    }
}

/// Options for the generate stage of the bundling pipeline.
///
/// This is a subset of [`TypackOptions`] containing only the fields needed
/// by [`Bundle::generate`](crate::Bundle::generate).
pub struct GenerateOptions {
    /// Working directory. Region marker paths are relative to this.
    pub cwd: PathBuf,
    /// Generate source map (`.d.ts.map`).
    pub sourcemap: bool,
    /// Emit `export =` for single default export output.
    pub cjs_default: bool,
}

impl GenerateOptions {
    /// Create `GenerateOptions` from the corresponding fields in [`TypackOptions`].
    pub fn from_typack_options(options: &TypackOptions) -> Self {
        Self {
            cwd: options.cwd.clone(),
            sourcemap: options.sourcemap,
            cjs_default: options.cjs_default,
        }
    }
}
