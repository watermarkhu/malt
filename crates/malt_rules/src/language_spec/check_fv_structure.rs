//! See the parent module `super` for the shared engine and helpers.

use super::*;

impl LanguageSpecEngine {
    /// Block-level structural checks for arguments blocks.
    pub(crate) fn check_fv_structure(
        &self,
        blocks: &[ArgumentsBlockMeta],
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.check_fvdrep(blocks, diagnostics);
        self.check_fvioa(blocks, diagnostics);
        self.check_fvrepo(blocks, diagnostics);
        self.check_fvvrep(blocks, diagnostics);
        self.check_fvovrep(blocks, diagnostics);
        self.check_fvnrep_fvrepd(blocks, source, diagnostics);
        self.check_fvood_fvooi_fvoon(blocks, source, diagnostics);
        self.check_fvorm(blocks, source, diagnostics);
        self.check_fvobi(blocks, diagnostics);
        self.check_fvatf(blocks, source, diagnostics);
        self.check_fvmcl(blocks, source, diagnostics);
        self.check_fvnde_fvnvl(blocks, source, diagnostics);
    }
}
