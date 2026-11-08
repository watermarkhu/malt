//! SODEPPROP: Access to deprecated System object properties.

use super::*;

impl SystemObjectsEngine {
    /// Check SODEPPROP: access to deprecated System object properties.
    pub(crate) fn check_sodeprop(
        &self,
        node: tree_sitter::Node,
        func_name: &str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // SODEPPROP: Detect access to deprecated properties
        // Pattern: obj.SampleRate, obj.FrameLength, etc.
        if self.is_enabled("SODEPPROP") && func_name.contains('.') {
            if let Some(prop_name) = func_name.rsplit('.').next() {
                if DEPRECATED_PROPERTIES.iter().any(|(p, _)| *p == prop_name) {
                    diags.push(make_diag_named("SODEPPROP", node, prop_name));
                }
            }
        }

        diags
    }
}
