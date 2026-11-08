//! # CLSAT / CLSUNK / NOPRV class file checks
//!
//! File-level checks about the class definition itself:
//!
//! - **CLSAT** — class attributes must be specified before the name of the
//!   class. The tree-sitter grammar enforces `classdef (attrs) Name`, so this
//!   only fires if a malformed ordering ever reaches the AST.
//! - **CLSUNK** — a superclass cannot be resolved on MATLAB's path. Since the
//!   linter cannot resolve the path, this is a narrow heuristic: it fires for
//!   bare superclass names that are not known MATLAB base classes and for
//!   dotted names whose first (lowercase) segment is not a known package.
//! - **NOPRV** — a class definition cannot live inside a `private` directory.
//!
//! See the parent module `super` for the shared engine and helpers.

use super::*;

/// Known MATLAB base classes that never need path resolution (CLSUNK).
const KNOWN_BASE_CLASSES: &[&str] = &[
    "handle",
    "value",
    "dynamicprops",
    "matlab.System",
    "matlab.mixin.Copyable",
    "matlab.mixin.SetGet",
    "matlab.mixin.Heterogeneous",
    "matlab.mixin.CustomDisplay",
    "matlab.mixin.Scalar",
    "matlab.mixin.indexing.RedefinesParen",
    "matlab.mixin.indexing.RedefinesBrace",
    "matlab.mixin.indexing.RedefinesDot",
    "matlab.mixin.indexing.RedefinesParen",
];

/// Known top-level packages whose classes are always resolvable (CLSUNK).
const KNOWN_PACKAGES: &[&str] = &[
    "matlab",
    "simulink",
    "coder",
    "parallel",
    "dsp",
    "vision",
    "stateflow",
    "fixedpoint",
    "sldv",
    "polyspace",
    "rf",
    "mpt",
];

impl LanguageSpecEngine {
    /// Run the class-file checks (CLSAT, CLSUNK, NOPRV).
    pub(crate) fn check_class_file_rules(
        &self,
        meta: &FileMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(class) = meta.class.as_ref() else {
            return;
        };

        self.check_clsat(ctx, diagnostics);
        self.check_clsunk(class, diagnostics);
        self.check_noprv(class, ctx, diagnostics);
    }

    /// CLSAT: class attributes must be specified before the name of the class.
    ///
    /// The grammar always places the `attributes` node before the class-name
    /// `identifier`, so this is a structural guard that fires only for a
    /// malformed ordering (which tree-sitter normally cannot produce).
    fn check_clsat(&self, ctx: &FileContext, diagnostics: &mut Vec<Diagnostic>) {
        let Some(class_node) = Self::find_class_definition_node(ctx.tree.root_node()) else {
            return;
        };
        let Some(name_node) = class_node.child_by_field_name("name") else {
            return;
        };
        let Some(attrs_node) = find_child_kind(class_node, "attributes") else {
            return;
        };
        if name_node.start_byte() < attrs_node.start_byte() {
            self.push_diag(
                attrs_node,
                "CLSAT",
                "Specify class attributes before the name of the class.",
                diagnostics,
            );
        }
    }

    /// CLSUNK: this class, or one of its superclasses, could not be found on
    /// MATLAB's path.
    ///
    /// Full path resolution is impossible from a static linter, so this is a
    /// deliberately narrow heuristic: it fires for superclass names that are
    /// not known MATLAB base classes and either have no package prefix (bare
    /// names) or start with a lowercase segment that is not a known package.
    fn check_clsunk(&self, class: &ClassMeta, diagnostics: &mut Vec<Diagnostic>) {
        for sc in &class.superclasses {
            if self.superclass_is_resolvable(sc) {
                continue;
            }
            if self.is_check_enabled("CLSUNK") {
                diagnostics.push(Diagnostic {
                    rule_id: "CLSUNK",
                    message: "This class, or one of its superclasses, could not be found on MATLAB's path."
                        .to_string(),
                    severity: Severity::Error,
                    byte_range: class.byte_range.clone(),
                    line: class.line,
                    column: 1,
                    fix: None,
                });
            }
        }
    }

    /// Whether a superclass name can be treated as resolvable without path
    /// information.
    fn superclass_is_resolvable(&self, name: &str) -> bool {
        if KNOWN_BASE_CLASSES.contains(&name) {
            return true;
        }
        let first = name.split('.').next().unwrap_or(name);
        if KNOWN_PACKAGES.contains(&first) {
            return true;
        }
        // Dotted names whose first segment does not start lowercase are
        // treated as resolvable (user classes conventionally start uppercase).
        if name.contains('.') && !first.starts_with(|c: char| c.is_lowercase()) {
            return true;
        }
        false
    }

    /// NOPRV: a class definition cannot be inside a private directory.
    fn check_noprv(&self, class: &ClassMeta, ctx: &FileContext, diagnostics: &mut Vec<Diagnostic>) {
        let in_private = ctx
            .file_path
            .components()
            .any(|c| c.as_os_str() == "private");
        if in_private && self.is_check_enabled("NOPRV") {
            diagnostics.push(Diagnostic {
                rule_id: "NOPRV",
                message: "A class definition cannot be inside a private directory.".to_string(),
                severity: Severity::Error,
                byte_range: class.byte_range.clone(),
                line: class.line,
                column: 1,
                fix: None,
            });
        }
    }

    /// Find the first `class_definition` node in the tree.
    fn find_class_definition_node(node: tree_sitter::Node) -> Option<tree_sitter::Node> {
        if node.kind() == "class_definition" {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = Self::find_class_definition_node(child) {
                return Some(found);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {

    use crate::language_spec::tests::{check_source, filter_by_id};
    use malt_core::Rule;

    // -- CLSAT ---------------------------------------------------------------

    #[test]
    fn test_clsat_no_fire_attributes_before_name() {
        let source = "\
classdef (Sealed, Abstract) Foo
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "CLSAT").is_empty(),
            "CLSAT should NOT fire when attributes precede the class name"
        );
    }

    #[test]
    fn test_clsat_no_fire_no_attributes() {
        let source = "\
classdef Foo
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "CLSAT").is_empty(),
            "CLSAT should NOT fire when the class has no attributes"
        );
    }

    // -- CLSUNK --------------------------------------------------------------

    #[test]
    fn test_clsunk_fires_unknown_bare_superclass() {
        let source = "\
classdef Foo < NotARealClass
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "CLSUNK").is_empty(),
            "CLSUNK should fire for an unknown bare superclass name"
        );
    }

    #[test]
    fn test_clsunk_fires_unknown_package_superclass() {
        let source = "\
classdef Foo < nonexistent.pkg.Base
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            !filter_by_id(&diags, "CLSUNK").is_empty(),
            "CLSUNK should fire for an unknown lowercase package prefix"
        );
    }

    #[test]
    fn test_clsunk_no_fire_known_base_classes() {
        let source = "\
classdef Foo < handle
end

classdef Bar < matlab.mixin.Copyable
end

classdef Baz < matlab.System
end

classdef Qux < value
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "CLSUNK").is_empty(),
            "CLSUNK should NOT fire for known base classes"
        );
    }

    // -- NOPRV ---------------------------------------------------------------

    #[test]
    fn test_noprv_fires_class_in_private_dir() {
        let source = "\
classdef Foo
end
";
        let diags = check_source(source, "private/Foo.m");
        assert!(
            !filter_by_id(&diags, "NOPRV").is_empty(),
            "NOPRV should fire for a class in a private directory"
        );
    }

    #[test]
    fn test_noprv_no_fire_public_dir() {
        let source = "\
classdef Foo
end
";
        let diags = check_source(source, "Foo.m");
        assert!(
            filter_by_id(&diags, "NOPRV").is_empty(),
            "NOPRV should NOT fire for a class outside private directories"
        );
    }

    #[test]
    fn test_noprv_no_fire_function_in_private_dir() {
        let source = "\
function y = f(x)
    y = x;
end
";
        let diags = check_source(source, "private/f.m");
        assert!(
            filter_by_id(&diags, "NOPRV").is_empty(),
            "NOPRV should NOT fire for a function file in a private directory"
        );
    }

    // -- Config respect ------------------------------------------------------

    #[test]
    fn test_clsunk_respects_disabled_config() {
        let mut rule_config = crate::language_spec::LanguageSpecConfig::default();
        rule_config.disabled_checks.push("CLSUNK".to_string());
        let engine = crate::language_spec::LanguageSpecEngine {
            config: rule_config,
        };
        let source = "\
classdef Foo < NotARealClass
end
";
        let tree = crate::language_spec::tests::parse_matlab(source);
        let path = std::path::Path::new("Foo.m");
        let ctx = malt_core::FileContext {
            tree: &tree,
            source,
            file_path: path,
        };
        let diags = engine.check_file(&ctx);
        assert!(
            filter_by_id(&diags, "CLSUNK").is_empty(),
            "CLSUNK should be disabled via config disabled_checks"
        );
    }
}
