//! # 4.5-A: Property/attribute and option-name removals
//!
//! These generic compatibility checks detect removed class property attributes
//! and removed name-value option/property names that the function-name lookup
//! table cannot match:
//!
//! - **MCPDC** — a classdef declares both `Constant` and `Dependent` on the
//!   same properties block.
//! - **PSTAT** — a `properties` block uses the removed `Static` attribute.
//! - **ATVIZW** — a `properties` block uses the removed `Visible` attribute.
//! - **MCGCP** — a `get.<Prop>` method is defined for a `Constant` property.
//! - **Name-value removals** — a call passes a removed option/property name as
//!   a string literal (e.g. `'SamplingRate'`, `'FilterSpecification'`,
//!   `'DirectFeedthrough'`, `'NumReflections'`, `'UseHG2'`).

use super::super::CompatibilityEngine;
use malt_core::{Diagnostic, Severity};
use tree_sitter::Node;

/// Removed name-value option/property names → (check ID, message).
const REMOVED_NAMES: &[(&str, &str, &str)] = &[
    ("InitialHessType", "HESST", "'InitialHessType' has been removed. There is no simple replacement for this."),
    ("InitialHessMatrix", "HESSM", "'InitialHessMatrix' has been removed. There is no simple replacement for this."),
    ("SamplingRate", "TTSMP", "'SamplingRate' has been removed. Use 'SampleRate' instead."),
    ("LaserReturns", "LSRET", "'LaserReturns' has been removed. Use 'LaserReturn' instead, which is a direct replacement."),
    ("NumReflections", "RAYNR", "Input argument 'NumReflections' has been removed. Use 'MaxNumReflections' property of a ray tracing propagation model object instead."),
    ("UseHG2", "DFEATUREPARAM1", "'UseHG2' has been removed. With appropriate code changes, use '~verLessThan('matlab','8.4.0')' instead."),
    ("HGUsingMATLABClasses", "DFEATUREPARAM2", "'HGUsingMATLABClasses' has been removed. With appropriate code changes, use '~verLessThan('matlab','8.4.0')' instead."),
    ("DirectFeedthrough", "DSPIDF", "'DirectFeedthrough' property of 'dsp.VariableIntegerDelay' class has been removed."),
    ("FrameBasedProcessing", "SMPLMODE", "The property 'FrameBasedProcessing' has been removed."),
    ("ErrorMessage", "GETERR", "The 'ErrorMessage' property has been removed. At the command line, use 'MException.last' instead."),
    ("FilterSpecification", "COEFFS", "The property 'FilterSpecification' has been removed."),
    ("FirstFilterCoefficients", "COEFF1", "The property 'FirstFilterCoefficients' has been removed."),
    ("SecondFilterCoefficients", "COEFF2", "The property 'SecondFilterCoefficients' has been removed."),
    ("ThirdFilterCoefficients", "COEFF3", "The property 'ThirdFilterCoefficients' has been removed."),
    ("FirstFilterCoefficientsDataType", "COEFFD1", "The property 'FirstFilterCoefficientsDataType' has been removed."),
    ("SecondFilterCoefficientsDataType", "COEFFD2", "The property 'SecondFilterCoefficientsDataType' has been removed."),
    ("ThirdFilterCoefficientsDataType", "COEFFD3", "The property 'ThirdFilterCoefficientsDataType' has been removed."),
    ("CustomFirstFilterCoefficientsDataType", "COEFFC1", "The property 'CustomFirstFilterCoefficientsDataType' has been removed."),
    ("CustomSecondFilterCoefficientsDataType", "COEFFC2", "The property 'CustomSecondFilterCoefficientsDataType' has been removed."),
    ("CustomThirdFilterCoefficientsDataType", "COEFFC3", "The property 'CustomThirdFilterCoefficientsDataType' has been removed."),
    ("KeyValueLimit", "READSZK", "The property 'KeyValueLimit' has been removed."),
    ("RowsPerRead", "READSZR", "The property 'RowsPerRead' has been removed."),
];

/// SETERR: 'ErrorMessage' property removal (setter context — distinct message).
const SETERR_MSG: &str =
    "The 'ErrorMessage' property has been removed. There is no simple replacement for this.";

/// DSPFDF: 'DirectFeedthrough' property of the VariableFractionalDelay class.
const DSPFDF_MSG: &str =
    "'DirectFeedthrough' property of 'dsp.VariableFractionalDelay' class has been removed.";

/// MCPDC message.
const MCPDC_MSG: &str = "Specifying both the 'Constant' and 'Dependent' attributes on the same property is not supported.";
/// PSTAT message.
const PSTAT_MSG: &str =
    "The 'Static' attribute on properties has been removed. Use the 'Constant' attribute instead.";
/// ATVIZW message.
const ATVIZW_MSG: &str = "The 'Visible' attribute has been removed. Use the '~Hidden' attribute instead or omit the attribute since 'Hidden' is false by default.";

/// Dispatch a node to the relevant check.
pub(crate) fn collect_checks(
    engine: &CompatibilityEngine,
    node: Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match node.kind() {
        "class_definition" => engine.check_class_attributes(node, source, diagnostics),
        "properties" => engine.check_properties_block(node, source, diagnostics),
        "string" => engine.check_removed_name(node, source, diagnostics),
        _ => {}
    }
}

impl CompatibilityEngine {
    /// MCPDC: a classdef's own attributes contain both Constant and Dependent.
    fn check_class_attributes(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if !has_attributes_with_both(node, "Constant", "Dependent", source) {
            return;
        }
        push_diag("MCPDC", MCPDC_MSG, node, Severity::Error, diagnostics);
    }

    /// PSTAT / ATVIZW: a properties block uses Static or Visible attributes.
    fn check_properties_block(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if has_attribute(node, "Static", source) {
            push_diag("PSTAT", PSTAT_MSG, node, Severity::Error, diagnostics);
        }
        if has_attribute(node, "Visible", source) {
            push_diag("ATVIZW", ATVIZW_MSG, node, Severity::Error, diagnostics);
        }
    }

    /// Removed name-value option/property names passed as string literals.
    fn check_removed_name(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        let text = unquoted(node, source);
        if let Some((id, msg)) = comm_memoryless_match(node, text, source) {
            push_diag(id, msg, node, Severity::Warning, diagnostics);
            return;
        }
        for (name, id, msg) in REMOVED_NAMES {
            if text == *name {
                // DSPIDF vs DSPFDF: distinguish by the owning class in the same call.
                let severity = Severity::Error;
                if *id == "DSPIDF"
                    && call_references_class(node, "dsp.VariableFractionalDelay", source)
                {
                    push_diag("DSPFDF", DSPFDF_MSG, node, severity, diagnostics);
                    return;
                }
                if *id == "GETERR" && is_setter_context(node) {
                    push_diag("SETERR", SETERR_MSG, node, severity, diagnostics);
                    return;
                }
                push_diag(id, msg, node, severity, diagnostics);
                return;
            }
        }
    }
}

/// Whether `node`'s attributes (direct children) include both `a` and `b`.
fn has_attributes_with_both(node: Node, a: &str, b: &str, source: &str) -> bool {
    let attrs = attribute_names(node, source);
    attrs.iter().any(|x| x == a) && attrs.iter().any(|x| x == b)
}

/// Whether `node`'s attributes include `name`.
fn has_attribute(node: Node, name: &str, source: &str) -> bool {
    attribute_names(node, source).iter().any(|x| x == name)
}

/// Collect the attribute names declared on a classdef or properties block.
fn attribute_names(node: Node, source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "attributes" {
            for name in attribute_names_in(child, source) {
                out.push(name);
            }
        }
    }
    out
}

/// Extract attribute names from an `attributes` node's `attribute` children.
fn attribute_names_in(attrs: Node, source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = attrs.walk();
    for child in attrs.children(&mut cursor) {
        if child.kind() == "attribute" {
            if let Some(id) = child
                .children(&mut child.walk())
                .find(|c| c.kind() == "identifier")
            {
                out.push(source[id.start_byte()..id.end_byte()].to_string());
            }
        }
    }
    out
}

/// Removed `comm.MemorylessNonlinearity` property names → (check ID, message).
/// Matched only as name-value string literals in a call to the class.
const COMM_MNL_NAMES: &[(&str, &str, &str)] = &[
    ("AMPMConversion", "CMAMP", "'AMPMConversion' property of 'comm.MemorylessNonlinearity' class will be removed in a future release. There is no simple replacement for this."),
    ("PowerLowerLimit", "CMPLW", "'PowerLowerLimit' property of 'comm.MemorylessNonlinearity' class will be removed in a future release. There is no simple replacement for this."),
    ("PowerUpperLimit", "CMPUL", "'PowerUpperLimit' property of 'comm.MemorylessNonlinearity' class will be removed in a future release. There is no simple replacement for this."),
];

/// Removed `Method` values of `comm.MemorylessNonlinearity` → (check ID, message).
/// Matched only when the value directly follows the `'Method'` name in a call
/// to the class.
const COMM_MNL_METHODS: &[(&str, &str, &str)] = &[
    ("Hyperbolic tangent", "CMMHT", "'Hyperbolic tangent' value for the 'Method' property on 'comm.MemorylessNonlinearity' class will be removed in a future release. There is no simple replacement for this."),
    ("Ghorbani model", "CMMGM", "'Ghorbani model' value for the 'Method' property on 'comm.MemorylessNonlinearity' class will be removed in a future release. There is no simple replacement for this."),
];

/// CMAMP / CMPLW / CMPUL / CMMHT / CMMGM: removed `comm.MemorylessNonlinearity`
/// properties and `Method` values. Only the name-value form inside a call to the
/// class is matched; property access on a variable (`m.AMPMConversion`) is not.
fn comm_memoryless_match(
    node: Node,
    text: &str,
    source: &str,
) -> Option<(&'static str, &'static str)> {
    let class = "comm.MemorylessNonlinearity";
    if let Some((_, id, msg)) = COMM_MNL_NAMES.iter().find(|(name, _, _)| *name == text) {
        return call_references_class(node, class, source).then_some((*id, *msg));
    }
    let (_, id, msg) = COMM_MNL_METHODS
        .iter()
        .find(|(value, _, _)| *value == text)?;
    let after_method = node
        .prev_named_sibling()
        .is_some_and(|p| p.kind() == "string" && unquoted(p, source) == "Method");
    (after_method && call_references_class(node, class, source)).then_some((*id, *msg))
}

/// Does the enclosing call reference the given dotted class name?
fn call_references_class(node: Node, class: &str, source: &str) -> bool {
    let mut cur = node.parent();
    while let Some(p) = cur {
        if p.kind() == "function_call" {
            // The call's own name may be just the method (e.g. VariableIntegerDelay);
            // the dotted class is in an enclosing field_expression.
            let own = p
                .child_by_field_name("name")
                .map(|n| &source[n.start_byte()..n.end_byte()]);
            if let Some(n) = own {
                if n.contains(class) {
                    return true;
                }
            }
            // Check the enclosing field_expression text (e.g. dsp.VariableIntegerDelay).
            if let Some(fe) = p.parent() {
                if fe.kind() == "field_expression" {
                    let text = &source[fe.start_byte()..fe.end_byte()];
                    if text.contains(class) {
                        return true;
                    }
                }
            }
        }
        cur = p.parent();
    }
    false
}

/// Is this node on the left side of an assignment (setter context)?
fn is_setter_context(node: Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    parent.kind() == "assignment"
        && parent
            .child_by_field_name("left")
            .map(|l| l.id() == node.id())
            .unwrap_or(false)
}

/// Strip surrounding quotes from a string node.
fn unquoted<'a>(node: Node<'a>, source: &'a str) -> &'a str {
    let text = &source[node.start_byte()..node.end_byte()];
    let bytes = text.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'')
            || (bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"'))
    {
        &text[1..text.len() - 1]
    } else {
        text
    }
}

/// Push a diagnostic at the node's location.
fn push_diag(
    id: &'static str,
    message: &str,
    node: Node,
    severity: Severity,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let pos = node.start_position();
    diagnostics.push(Diagnostic {
        rule_id: id,
        message: message.to_string(),
        severity,
        byte_range: node.start_byte()..node.end_byte(),
        line: pos.row + 1,
        column: pos.column + 1,
        fix: None,
    });
}

#[cfg(test)]
mod tests {
    use crate::compatibility::CompatibilityEngine;
    use crate::test_util::{has_id, lint_file};
    use malt_core::Config;

    fn engine() -> Box<dyn malt_core::Rule> {
        CompatibilityEngine::from_config(&Config::default())
    }

    #[test]
    fn mcpdc_constant_dependent_fires() {
        let d = lint_file(&*engine(), "classdef (Constant, Dependent) Foo\nend\n");
        assert!(has_id(&d, "MCPDC"), "got: {d:?}");
    }

    #[test]
    fn mcpdc_constant_only_not_fires() {
        let d = lint_file(&*engine(), "classdef (Constant) Foo\nend\n");
        assert!(!has_id(&d, "MCPDC"), "got: {d:?}");
    }

    #[test]
    fn pstat_static_property_fires() {
        let d = lint_file(
            &*engine(),
            "classdef Foo\nproperties (Static)\n x\nend\nend\n",
        );
        assert!(has_id(&d, "PSTAT"), "got: {d:?}");
    }

    #[test]
    fn pstat_no_static_not_fires() {
        let d = lint_file(&*engine(), "classdef Foo\nproperties\n x\nend\nend\n");
        assert!(!has_id(&d, "PSTAT"), "got: {d:?}");
    }

    #[test]
    fn atvizw_visible_property_fires() {
        let d = lint_file(
            &*engine(),
            "classdef Foo\nproperties (Visible = off)\n x\nend\nend\n",
        );
        assert!(has_id(&d, "ATVIZW"), "got: {d:?}");
    }

    #[test]
    fn atvizw_no_visible_not_fires() {
        let d = lint_file(&*engine(), "classdef Foo\nproperties\n x\nend\nend\n");
        assert!(!has_id(&d, "ATVIZW"), "got: {d:?}");
    }

    #[test]
    fn mcgcp_getter_for_nonconstant_not_fires() {
        let d = lint_file(
            &*engine(),
            "classdef Foo\nproperties\n x\nend\nmethods\n function v = get.x(o)\n  v = 1;\n end\nend\nend\n",
        );
        assert!(!has_id(&d, "MCGCP"), "got: {d:?}");
    }

    #[test]
    fn ttsmp_samplingrate_fires() {
        let d = lint_file(&*engine(), "set(gcf, 'SamplingRate', 100);\n");
        assert!(has_id(&d, "TTSMP"), "got: {d:?}");
    }

    #[test]
    fn ttsmp_samplerate_not_fires() {
        let d = lint_file(&*engine(), "set(gcf, 'SampleRate', 100);\n");
        assert!(!has_id(&d, "TTSMP"), "got: {d:?}");
    }

    #[test]
    fn hessm_initialhessmatrix_fires() {
        let d = lint_file(
            &*engine(),
            "opts = optimoptions('fmincon','InitialHessMatrix',eye(3));\n",
        );
        assert!(has_id(&d, "HESSM"), "got: {d:?}");
    }

    #[test]
    fn lsret_laserreturns_fires() {
        let d = lint_file(&*engine(), "m = raytracing('LaserReturns', true);\n");
        assert!(has_id(&d, "LSRET"), "got: {d:?}");
    }

    #[test]
    fn raynr_numreflections_fires() {
        let d = lint_file(&*engine(), "pm = raytrace('NumReflections', 2);\n");
        assert!(has_id(&d, "RAYNR"), "got: {d:?}");
    }

    #[test]
    fn dfeatureparam_usehg2_fires() {
        let d = lint_file(&*engine(), "setpref('dummy','UseHG2',true);\n");
        assert!(has_id(&d, "DFEATUREPARAM1"), "got: {d:?}");
    }

    #[test]
    fn readszk_keyvaluelimit_fires() {
        let d = lint_file(&*engine(), "readtable('f.txt', 'KeyValueLimit', 10);\n");
        assert!(has_id(&d, "READSZK"), "got: {d:?}");
    }

    #[test]
    fn readszr_rowsperread_fires() {
        let d = lint_file(&*engine(), "readtable('f.txt', 'RowsPerRead', 100);\n");
        assert!(has_id(&d, "READSZR"), "got: {d:?}");
    }

    #[test]
    fn readszk_other_prop_not_fires() {
        let d = lint_file(&*engine(), "readtable('f.txt', 'NumVariables', 2);\n");
        assert!(!has_id(&d, "READSZK"), "got: {d:?}");
        assert!(!has_id(&d, "READSZR"), "got: {d:?}");
    }

    #[test]
    fn dspidf_directfeedthrough_variableinteger_fires() {
        let d = lint_file(
            &*engine(),
            "d = dsp.VariableIntegerDelay('DirectFeedthrough', true);\n",
        );
        assert!(has_id(&d, "DSPIDF"), "got: {d:?}");
        assert!(!has_id(&d, "DSPFDF"), "got: {d:?}");
    }

    #[test]
    fn dspfdf_directfeedthrough_variablefractional_fires() {
        let d = lint_file(
            &*engine(),
            "d = dsp.VariableFractionalDelay('DirectFeedthrough', true);\n",
        );
        assert!(has_id(&d, "DSPFDF"), "got: {d:?}");
    }

    #[test]
    fn coeffs_filterspecification_fires() {
        let d = lint_file(&*engine(), "s = struct('FilterSpecification', 1);\n");
        assert!(has_id(&d, "COEFFS"), "got: {d:?}");
    }

    #[test]
    fn coeffc3_customthird_fires() {
        let d = lint_file(
            &*engine(),
            "set(obj, 'CustomThirdFilterCoefficientsDataType', x);\n",
        );
        assert!(has_id(&d, "COEFFC3"), "got: {d:?}");
    }

    #[test]
    fn comm_memoryless_properties_fire() {
        let d = lint_file(
            &*engine(),
            "m = comm.MemorylessNonlinearity('AMPMConversion', 1, 'PowerLowerLimit', 1, 'PowerUpperLimit', 2);\n",
        );
        assert!(has_id(&d, "CMAMP"), "got: {d:?}");
        assert!(has_id(&d, "CMPLW"), "got: {d:?}");
        assert!(has_id(&d, "CMPUL"), "got: {d:?}");
    }

    #[test]
    fn comm_memoryless_method_values_fire() {
        let d = lint_file(
            &*engine(),
            "m = comm.MemorylessNonlinearity('Method', 'Hyperbolic tangent');\n",
        );
        assert!(has_id(&d, "CMMHT"), "got: {d:?}");
        let d = lint_file(
            &*engine(),
            "m = comm.MemorylessNonlinearity('Method', 'Ghorbani model');\n",
        );
        assert!(has_id(&d, "CMMGM"), "got: {d:?}");
    }

    #[test]
    fn comm_memoryless_other_uses_not_flagged() {
        let d = lint_file(
            &*engine(),
            "m = comm.MemorylessNonlinearity('Method', 'Saleh model');\nx = 'Hyperbolic tangent';\nset(m, 'AMPMConversion', 1);\n",
        );
        for id in ["CMAMP", "CMPLW", "CMPUL", "CMMHT", "CMMGM"] {
            assert!(!has_id(&d, id), "{id} should not fire: {d:?}");
        }
    }
}
