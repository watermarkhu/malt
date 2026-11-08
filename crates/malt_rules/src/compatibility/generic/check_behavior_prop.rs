//! # 4.5-D: Behavior-change figure/axes properties and settings
//!
//! These checks detect name-value property/setting arguments passed to
//! figure/axes functions (and `set`/`get`/`feature`) that MATLAB flags as
//! behavior changes or future removals:
//!
//! - **SMTHG / SMTHF / INVHCRM** — removed figure properties
//!   (`'GraphicsSmoothing'`, `'FontSmoothing'`, `'InvertHardCopy'`) used as
//!   name-value pairs on `figure`, `set`, `get`, or chart-creation calls.
//! - **SMTHGF / SMTHFA / SMTHFT / DINVHCRM** — removed groot settings
//!   (`'DefaultFigureGraphicsSmoothing'`, `'DefaultAxesFontSmoothing'`,
//!   `'DefaultTextFontSmoothing'`, `'defaultFigureInvertHardCopy'`) passed to
//!   `set`/`get`/`feature`.
//! - **PTCLO / PTDLO** — `'LineStyleOrder'`/`'ColorOrder'` name-value args on
//!   `set`/`axes`/plot calls (PTCLO), and `'LineStyleOrder'` values that are
//!   cell arrays with more than one style string (PTDLO).
//!
//! A string (or `Name = value` identifier) matching one of the removed names is
//! only reported when it appears in a name context: either as an argument to
//! `set`/`get`/`feature`, or as a name followed by a value in a call's argument
//! list. The diagnostic fires on the name node itself.

use super::super::CompatibilityEngine;
use malt_core::{Diagnostic, Severity};
use tree_sitter::Node;

/// PTCLO: changing axes LineStyleOrder/ColorOrder now affects existing charts.
const MSG_PTCLO: &str = "Changing the axes LineStyleOrder or ColorOrder properties of an existing chart now affects the chart immediately. To revert to the old behavior, set either the axes LineStyleOrderIndex or ColorOrderIndex to any value (such as its current value) before changing LineStyleOrder or ColorOrder.";

/// PTDLO: multiple line styles in the axes LineStyleOrder render differently.
const MSG_PTDLO: &str = "Specifying multiple line styles in the axes LineStyleOrder might result in charts that render differently than in the previous releases. MATLAB uses a new indexing scheme to select colors and line styles. To revert to the old behavior, set either the axes LineStyleOrderIndex or ColorOrderIndex to any value (such as its current value) and call 'hold on' before creating your chart.";

/// SMTHG: 'GraphicsSmoothing' property removal notice.
const MSG_SMTHG: &str = "'GraphicsSmoothing' property will be removed in a future release. Graphics smoothing behavior has been enabled by default. Editing this property does not have an effect.";

/// SMTHGF: 'DefaultFigureGraphicsSmoothing' setting removal notice.
const MSG_SMTHGF: &str = "'DefaultFigureGraphicsSmoothing' setting will be removed in a future release. Graphics smoothing behavior has been enabled by default. Editing this setting does not have an effect.";

/// SMTHF: 'FontSmoothing' property removal notice.
const MSG_SMTHF: &str = "'FontSmoothing' property will be removed in a future release. Font smoothing behavior has been enabled by default. Editing this property does not have an effect.";

/// SMTHFA: 'DefaultAxesFontSmoothing' setting removal notice.
const MSG_SMTHFA: &str = "'DefaultAxesFontSmoothing' setting will be removed in a future release. Font smoothing behavior has been enabled by default. Editing this setting does not have an effect.";

/// SMTHFT: 'DefaultTextFontSmoothing' setting removal notice.
const MSG_SMTHFT: &str = "'DefaultTextFontSmoothing' setting will be removed in a future release. Font smoothing behavior has been enabled by default. Editing this setting does not have an effect.";

/// INVHCRM: 'InvertHardCopy' property removal notice.
const MSG_INVHCRM: &str = "The 'InvertHardCopy' property will be removed in a future release and currently has no effect. With appropriate code changes, set the 'Color' property of the figure before printing or call the 'exportgraphics' function to control the color for exporting.";

/// DINVHCRM: 'defaultFigureInvertHardCopy' setting removal notice.
const MSG_DINVHCRM: &str = "The 'defaultFigureInvertHardCopy' setting will be removed in a future release and currently has no effect. With appropriate code changes, set the 'Color' property of the figure before printing or call the 'exportgraphics' function to control the color for exporting.";

/// Functions on which a `'LineStyleOrder'`/`'ColorOrder'` name-value argument
/// triggers PTCLO: `set`, `axes`, and common chart-creation functions.
const PTCLO_FUNCS: &[&str] = &[
    "set", "axes", "plot", "plot3", "semilogx", "semilogy", "loglog", "bar", "barh", "scatter",
    "stairs", "stem", "area", "line", "errorbar",
];

/// Dispatch a single node to every behavior-change property/setting check.
pub(crate) fn collect_checks(
    engine: &CompatibilityEngine,
    node: Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() != "function_call" {
        return;
    }
    engine.check_smthg(node, source, diagnostics);
    engine.check_smthgf(node, source, diagnostics);
    engine.check_smthf(node, source, diagnostics);
    engine.check_smthfa(node, source, diagnostics);
    engine.check_smthft(node, source, diagnostics);
    engine.check_invhcrm(node, source, diagnostics);
    engine.check_dinvhcrm(node, source, diagnostics);
    engine.check_ptclo(node, source, diagnostics);
    engine.check_ptdlo(node, source, diagnostics);
}

impl CompatibilityEngine {
    /// SMTHG: `'GraphicsSmoothing'` figure property used as a name-value pair.
    pub(crate) fn check_smthg(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        scan_name_args(
            node,
            source,
            "GraphicsSmoothing",
            "SMTHG",
            MSG_SMTHG,
            diagnostics,
        );
    }

    /// SMTHGF: `'DefaultFigureGraphicsSmoothing'` groot setting.
    pub(crate) fn check_smthgf(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        scan_name_args(
            node,
            source,
            "DefaultFigureGraphicsSmoothing",
            "SMTHGF",
            MSG_SMTHGF,
            diagnostics,
        );
    }

    /// SMTHF: `'FontSmoothing'` figure property used as a name-value pair.
    pub(crate) fn check_smthf(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        scan_name_args(
            node,
            source,
            "FontSmoothing",
            "SMTHF",
            MSG_SMTHF,
            diagnostics,
        );
    }

    /// SMTHFA: `'DefaultAxesFontSmoothing'` groot setting.
    pub(crate) fn check_smthfa(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        scan_name_args(
            node,
            source,
            "DefaultAxesFontSmoothing",
            "SMTHFA",
            MSG_SMTHFA,
            diagnostics,
        );
    }

    /// SMTHFT: `'DefaultTextFontSmoothing'` groot setting.
    pub(crate) fn check_smthft(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        scan_name_args(
            node,
            source,
            "DefaultTextFontSmoothing",
            "SMTHFT",
            MSG_SMTHFT,
            diagnostics,
        );
    }

    /// INVHCRM: `'InvertHardCopy'` figure property used as a name-value pair.
    pub(crate) fn check_invhcrm(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        scan_name_args(
            node,
            source,
            "InvertHardCopy",
            "INVHCRM",
            MSG_INVHCRM,
            diagnostics,
        );
    }

    /// DINVHCRM: `'defaultFigureInvertHardCopy'` groot setting.
    pub(crate) fn check_dinvhcrm(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        scan_name_args(
            node,
            source,
            "defaultFigureInvertHardCopy",
            "DINVHCRM",
            MSG_DINVHCRM,
            diagnostics,
        );
    }

    /// PTCLO: `'LineStyleOrder'`/`'ColorOrder'` name-value args on `set`,
    /// `axes`, or chart-creation functions.
    pub(crate) fn check_ptclo(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if !is_ptclo_func(call_name(node, source)) {
            return;
        }
        let args = named_args(node);
        let total = args.len();
        for (index, (arg, is_positional)) in args.iter().enumerate() {
            let Some(text) = arg_text(*arg, source) else {
                continue;
            };
            if (text == "LineStyleOrder" || text == "ColorOrder")
                && is_name_value_name(*arg, *is_positional, index, total)
            {
                fire(*arg, "PTCLO", MSG_PTCLO, diagnostics);
            }
        }
    }

    /// PTDLO: `'LineStyleOrder'` name-value whose value is a cell array with
    /// more than one style string.
    pub(crate) fn check_ptdlo(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        for (arg, _) in named_args(node) {
            if arg_text(arg, source) != Some("LineStyleOrder") {
                continue;
            }
            // The value is the next named argument (works for both
            // `'LineStyleOrder', {...}` and `LineStyleOrder={...}` forms).
            if let Some(value) = arg.next_named_sibling() {
                if cell_style_count(value, source) >= 2 {
                    fire(arg, "PTDLO", MSG_PTDLO, diagnostics);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The callee name of a `function_call` node, if any.
fn call_name<'a>(call: Node<'a>, source: &'a str) -> Option<&'a str> {
    let name = call.child_by_field_name("name")?;
    Some(&source[name.start_byte()..name.end_byte()])
}

/// Named children of a call's argument list, paired with a flag that is true
/// for positional arguments (those carrying the `argument:` field). `Name =
/// value` names/values are unnamed and get `false`.
fn named_args<'a>(call: Node<'a>) -> Vec<(Node<'a>, bool)> {
    let mut result = Vec::new();
    let mut cursor = call.walk();
    let args = call
        .children(&mut cursor)
        .find(|child| child.kind() == "arguments");
    let Some(args) = args else {
        return result;
    };
    let mut args_cursor = args.walk();
    for (index, child) in args.children(&mut args_cursor).enumerate() {
        if child.is_named() {
            let is_positional = args.field_name_for_child(index as u32) == Some("argument");
            result.push((child, is_positional));
        }
    }
    result
}

/// The string/identifier text of an argument, with surrounding quotes removed
/// for string literals.
fn arg_text<'a>(arg: Node<'a>, source: &'a str) -> Option<&'a str> {
    match arg.kind() {
        "string" => Some(unquoted(arg, source)),
        "identifier" => Some(&source[arg.start_byte()..arg.end_byte()]),
        _ => None,
    }
}

/// Strip the surrounding quotes from a string-literal node's text.
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

/// True when the callee is `set`, `get`, or `feature`.
fn is_set_get_or_feature(func_name: Option<&str>) -> bool {
    matches!(func_name, Some("set") | Some("get") | Some("feature"))
}

/// True when the argument is used as the *name* of a name-value pair: either a
/// `Name = value` identifier (immediately followed by `=`), or a positional
/// argument that is followed by another argument (its value).
fn is_name_value_name(arg: Node, is_positional: bool, index: usize, total: usize) -> bool {
    if arg.kind() == "identifier" {
        arg.next_sibling()
            .is_some_and(|sibling| sibling.kind() == "=")
    } else {
        is_positional && index + 1 < total
    }
}

/// True when a removed property/setting name appears in a name context: as an
/// argument to `set`/`get`/`feature`, or as the name of a name-value pair.
fn in_name_context(
    func_name: Option<&str>,
    arg: Node,
    is_positional: bool,
    index: usize,
    total: usize,
) -> bool {
    if is_set_get_or_feature(func_name) && arg.kind() == "string" {
        return true;
    }
    is_name_value_name(arg, is_positional, index, total)
}

/// True when the callee is a function PTCLO applies to.
fn is_ptclo_func(func_name: Option<&str>) -> bool {
    match func_name {
        Some(name) => PTCLO_FUNCS.contains(&name),
        None => false,
    }
}

/// Number of non-empty string literals inside a cell array (PTDLO).
fn cell_style_count(value: Node, source: &str) -> usize {
    if value.kind() != "cell" {
        return 0;
    }
    let mut count = 0;
    let mut cursor = value.walk();
    for child in value.children(&mut cursor) {
        count += count_strings(child, source);
    }
    count
}

/// Count non-empty string literals under `node`, recursing through containers
/// (e.g. the `row` child of a cell).
fn count_strings(node: Node, source: &str) -> usize {
    if node.kind() == "string" {
        return if unquoted(node, source).is_empty() {
            0
        } else {
            1
        };
    }
    let mut count = 0;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        count += count_strings(child, source);
    }
    count
}

/// Scan one call for the given property/setting name and emit the check when
/// the name appears in a name context. Fires on the name node itself.
fn scan_name_args(
    call: Node,
    source: &str,
    property_name: &str,
    rule_id: &'static str,
    message: &'static str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let func_name = call_name(call, source);
    let args = named_args(call);
    let total = args.len();
    for (index, (arg, is_positional)) in args.iter().enumerate() {
        let Some(text) = arg_text(*arg, source) else {
            continue;
        };
        if text == property_name && in_name_context(func_name, *arg, *is_positional, index, total) {
            fire(*arg, rule_id, message, diagnostics);
        }
    }
}

/// Emit a warning diagnostic on the given node.
fn fire(
    node: Node,
    rule_id: &'static str,
    message: &'static str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let start = node.start_position();
    diagnostics.push(Diagnostic {
        rule_id,
        message: message.to_string(),
        severity: Severity::Warning,
        byte_range: node.start_byte()..node.end_byte(),
        line: start.row + 1,
        column: start.column + 1,
        fix: None,
    });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{has_id, lint_file};

    fn diags(source: &str) -> Vec<Diagnostic> {
        lint_file(&CompatibilityEngine, source)
    }

    fn fires(source: &str, id: &str) {
        let got = diags(source);
        assert!(
            has_id(&got, id),
            "expected {id} to fire for {source:?}; got: {got:?}"
        );
    }

    fn not_fires(source: &str, id: &str) {
        let got = diags(source);
        assert!(
            !has_id(&got, id),
            "did not expect {id} to fire for {source:?}; got: {got:?}"
        );
    }

    // -- SMTHG: 'GraphicsSmoothing' figure property -------------------------

    #[test]
    fn smthg_figure_name_value_fires() {
        fires("figure('GraphicsSmoothing', 'on');", "SMTHG");
    }

    #[test]
    fn smthg_set_name_value_fires() {
        fires("set(gcf, 'GraphicsSmoothing', 'on');", "SMTHG");
    }

    #[test]
    fn smthg_get_fires() {
        fires("get(gcf, 'GraphicsSmoothing');", "SMTHG");
    }

    #[test]
    fn smthg_name_equals_value_fires() {
        fires("figure(GraphicsSmoothing=\"on\");", "SMTHG");
    }

    #[test]
    fn smthg_disp_not_fires() {
        not_fires("disp('GraphicsSmoothing');", "SMTHG");
    }

    #[test]
    fn smthg_assignment_not_fires() {
        not_fires("x = 'GraphicsSmoothing';", "SMTHG");
    }

    // -- SMTHGF: 'DefaultFigureGraphicsSmoothing' setting -------------------

    #[test]
    fn smthgf_set_fires() {
        fires(
            "set(groot, 'DefaultFigureGraphicsSmoothing', 'off');",
            "SMTHGF",
        );
    }

    #[test]
    fn smthgf_get_fires() {
        fires("get(groot, 'DefaultFigureGraphicsSmoothing');", "SMTHGF");
    }

    #[test]
    fn smthgf_disp_not_fires() {
        not_fires("disp('DefaultFigureGraphicsSmoothing');", "SMTHGF");
    }

    // -- SMTHF: 'FontSmoothing' figure property -----------------------------

    #[test]
    fn smthf_figure_name_value_fires() {
        fires("figure('FontSmoothing', 'off');", "SMTHF");
    }

    #[test]
    fn smthf_get_fires() {
        fires("get(gcf, 'FontSmoothing');", "SMTHF");
    }

    #[test]
    fn smthf_disp_not_fires() {
        not_fires("disp('FontSmoothing');", "SMTHF");
    }

    #[test]
    fn smthf_variable_positional_not_fires() {
        not_fires("plot(FontSmoothing, x);", "SMTHF");
    }

    // -- SMTHFA: 'DefaultAxesFontSmoothing' setting -------------------------

    #[test]
    fn smthfa_set_fires() {
        fires("set(groot, 'DefaultAxesFontSmoothing', 'off');", "SMTHFA");
    }

    #[test]
    fn smthfa_disp_not_fires() {
        not_fires("disp('DefaultAxesFontSmoothing');", "SMTHFA");
    }

    // -- SMTHFT: 'DefaultTextFontSmoothing' setting -------------------------

    #[test]
    fn smthft_feature_fires() {
        fires("feature('DefaultTextFontSmoothing', 'off');", "SMTHFT");
    }

    #[test]
    fn smthft_get_fires() {
        fires("get(groot, 'DefaultTextFontSmoothing');", "SMTHFT");
    }

    #[test]
    fn smthft_disp_not_fires() {
        not_fires("disp('DefaultTextFontSmoothing');", "SMTHFT");
    }

    // -- INVHCRM: 'InvertHardCopy' figure property --------------------------

    #[test]
    fn invhcrm_set_fires() {
        fires("set(f, 'InvertHardCopy', 'off');", "INVHCRM");
    }

    #[test]
    fn invhcrm_figure_name_value_fires() {
        fires("figure('InvertHardCopy', 'off');", "INVHCRM");
    }

    #[test]
    fn invhcrm_disp_not_fires() {
        not_fires("disp('InvertHardCopy');", "INVHCRM");
    }

    // -- DINVHCRM: 'defaultFigureInvertHardCopy' setting --------------------

    #[test]
    fn dinvhcrm_set_fires() {
        fires(
            "set(groot, 'defaultFigureInvertHardCopy', 'off');",
            "DINVHCRM",
        );
    }

    #[test]
    fn dinvhcrm_get_fires() {
        fires("get(groot, 'defaultFigureInvertHardCopy');", "DINVHCRM");
    }

    #[test]
    fn dinvhcrm_disp_not_fires() {
        not_fires("disp('defaultFigureInvertHardCopy');", "DINVHCRM");
    }

    // -- PTCLO: LineStyleOrder/ColorOrder on set/axes/plot calls ------------

    #[test]
    fn ptclo_set_linestyleorder_fires() {
        fires("set(ax, 'LineStyleOrder', {'-', '--'});", "PTCLO");
    }

    #[test]
    fn ptclo_axes_colororder_fires() {
        fires("axes('ColorOrder', c);", "PTCLO");
    }

    #[test]
    fn ptclo_plot_colororder_fires() {
        fires("plot(x, y, 'ColorOrder', c);", "PTCLO");
    }

    #[test]
    fn ptclo_bar_linestyleorder_fires() {
        fires("bar(x, y, 'LineStyleOrder', {'-', '--', ':'});", "PTCLO");
    }

    #[test]
    fn ptclo_get_colororder_not_fires() {
        not_fires("get(ax, 'ColorOrder');", "PTCLO");
    }

    #[test]
    fn ptclo_plain_plot_not_fires() {
        not_fires("plot(x, y);", "PTCLO");
    }

    #[test]
    fn ptclo_disp_not_fires() {
        not_fires("disp('LineStyleOrder');", "PTCLO");
    }

    // -- PTDLO: LineStyleOrder with multiple line styles --------------------

    #[test]
    fn ptdlo_two_styles_fires() {
        fires("set(ax, 'LineStyleOrder', {'-.', '--'});", "PTDLO");
    }

    #[test]
    fn ptdlo_three_styles_fires() {
        fires("set(ax, 'LineStyleOrder', {'-', '--', ':'});", "PTDLO");
    }

    #[test]
    fn ptdlo_name_equals_value_fires() {
        fires("set(ax, LineStyleOrder={'-','--'});", "PTDLO");
    }

    #[test]
    fn ptdlo_single_string_value_not_fires() {
        not_fires("set(ax, 'LineStyleOrder', '-');", "PTDLO");
    }

    #[test]
    fn ptdlo_single_style_cell_not_fires() {
        not_fires("set(ax, 'LineStyleOrder', {'-'});", "PTDLO");
    }

    #[test]
    fn ptdlo_get_not_fires() {
        not_fires("get(ax, 'LineStyleOrder');", "PTDLO");
    }

    // -- cross-check: clean code produces no diagnostics --------------------

    #[test]
    fn clean_plot_not_flagged() {
        // `x` and `y` are defined before use; `plot` is a plain callee with
        // no variable definition, so the Phase 7 unset-var checks stay quiet.
        let got = diags("x = 1;\ny = 2;\nplot(x, y);\n");
        assert!(got.is_empty(), "got: {got:?}");
    }
}
