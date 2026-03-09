//! Warning collection for the link stage.

use oxc_diagnostics::OxcDiagnostic;

use crate::scan_stage::ScanStageOutput;

use super::types::CanonicalNames;

pub fn collect_link_warnings(
    canonical_names: &CanonicalNames,
    scan_result: &ScanStageOutput<'_>,
) -> Vec<OxcDiagnostic> {
    let mut warnings = Vec::new();
    for ((module_idx, original_name), renamed_name) in &canonical_names.fallback_name_renames {
        let module_path = &scan_result.module_table[*module_idx].path;
        warnings.push(OxcDiagnostic::warn(format!(
            "typack/rename-fallback: used fallback rename for \"{original_name}\" in {} -> \"{renamed_name}\"",
            module_path.display()
        )));
    }
    warnings
}
