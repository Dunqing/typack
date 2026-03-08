//! Warning collection for the link stage.

use oxc_diagnostics::OxcDiagnostic;

use crate::scan_stage::ScanResult;

use super::types::RenamePlan;

pub fn collect_link_warnings(
    rename_plan: &RenamePlan,
    scan_result: &ScanResult<'_>,
) -> Vec<OxcDiagnostic> {
    let mut warnings = Vec::new();
    for ((module_idx, original_name), renamed_name) in &rename_plan.fallback_name_renames {
        let module_path = &scan_result.modules[*module_idx].path;
        warnings.push(OxcDiagnostic::warn(format!(
            "typack/rename-fallback: used fallback rename for \"{original_name}\" in {} -> \"{renamed_name}\"",
            module_path.display()
        )));
    }
    warnings
}
