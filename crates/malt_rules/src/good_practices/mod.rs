//! # GOOD_PRACTICES_ENGINE: Good Practices
//!
//! ```malt
//! id = "GOOD_PRACTICES_ENGINE"
//! title = "Good Practices"
//! category = "good-practices"
//! severity = "warning"
//! fix = true
//! icon = "lucide/check-circle"
//! slug = "good-practices"
//! ```
//!
//! ## Rule
//!
//! Encourages recommended coding practices for MATLAB code. All 106 checks are
//! handled by a single hybrid engine (`GoodPracticesEngine`). Node-level checks
//! cover simple pattern matches (error handling, `eval` usage, string
//! comparisons, parfor/spmd usage, redundant comparisons), while file-level
//! checks use metadata extraction and the symbol table for context-aware
//! analysis (last-statement detection, variable shadowing, class and property
//! validation). Each diagnostic carries the specific check ID (e.g. `TRYNC`,
//! `EVLCS`, `PFRNI`).
//!
//! The checks span error handling, string and comparison idioms, `eval`/dynamic
//! code, parfor/spmd parallel practices, structure and field access, OOP and
//! class properties, logical-usage patterns, shared variables, arity, and App
//! Designer methods.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | TRYNC | warning | no | TRY statement should have a CATCH statement to check for unexpected errors. |
//! | CTCH | warning | no | Best practice is for CATCH to be followed by an identifier that gets the error information. |
//! | WLAST | warning | no | WARNING('') does not reset the warning state. Use LASTWARN('') instead. |
//! | WNTAG | warning | no | The first argument of WARNING should be a message identifier. Using a message identifier allows users better control over the message. |
//! | ERTAG | warning | no | The first argument of ERROR should be a message identifier. |
//! | MEXCEP | warning | no | To report an MException as a warning, use a format specifier to ensure the message is printed correctly. For example, 'warning(E.identifier, "%s", E.message)'. |
//! | STCMP | warning | no | Use STRCMP instead of == or ~= to compare character vectors, or convert character vectors to string scalars for direct comparison. |
//! | STCI | warning | no | Use STRCMPI(str1,str2) instead of using UPPER/LOWER in a call to STRCMP. |
//! | STISA | warning | no | Consider using ISA instead of comparing the class name. |
//! | STRNU | warning | no | This variable, apparently a structure, is changed but the value might be unused. |
//! | EVLCS | warning | no | 'eval' is inefficient and makes code less clear. Call the statement directly. |
//! | EVLDOT | warning | no | 'eval' is inefficient and makes code less clear. Use dynamic field names to access structure fields or object properties instead. |
//! | EVLEQ | warning | no | 'eval' is inefficient and makes code less clear. Assign to the variable directly. |
//! | EVLSYS | warning | no | 'eval' is inefficient and makes code less clear. To make calls to the operating system use the system function instead. |
//! | EVLDUAL | warning | no | This use of 'eval' is unnecessary and can be removed. Call the evaluated function directly using parentheses. For example, use 'load(filename)' instead of 'eval(['load ' filename])'. |
//! | EVLSEQVAR | warning | no | Using 'eval' to dynamically assign variables is not recommended. |
//! | NOANS | warning | no | Using ANS as a variable is not recommended as ANS is frequently overwritten by MATLAB. |
//! | LOAD | warning | no | To avoid conflicts with functions on the path, specify variables to load from file. |
//! | SEPEX | info | no | Consider using newline, semicolon, or comma before this statement for readability. |
//! | NBRAK1 | info | yes | If you intend to specify expression precedence, use parentheses () instead of brackets []. |
//! | LNGNM | warning | no | Names longer than VAR_NUMBER characters are not supported. This name has been truncated to VAR_NUMBER characters. |
//! | CHAIN | info | no | Expressions like a < b < c are interpreted as (a < b) < c. Use (a < b) && (b < c) instead. |
//! | DISPLAY | warning | no | Overloading DISPLAY is not recommended. |
//! | FNDEF | warning | no | Function name VAR_NAME is known to MATLAB by its file name: VAR_FILE. |
//! | NOIN | info | no | Method VAR_NAME should either be a static method or have at least one input argument. |
//! | VALST | info | no | VAR_NAME must be the last argument in the argument list. |
//! | PROP | info | no | VAR_NAME is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename this variable to improve readability. |
//! | CPROP | info | no | VAR_NAME is also the name of a property, which may be confusing. Use obj.PropertyName syntax to reference the property, or rename the property to improve readability. |
//! | FVAL | warning | no | Calling functions using 'feval' is usually not necessary. Call the function directly instead. |
//! | FNCOLND | warning | no | Consider explicitly defining the array, and then using the END operator to index into it. |
//! | COMNC | info | yes | Comment with percent (%) following comma acts as a row separator. Replace the comma with a semicolon to make the row separation clearer. Alternatively, replace the percent (%) with an ellipsis (...) to add a comment inside a row. |
//! | ITERS | warning | no | The Code Analyzer type analysis may be incorrect here. |
//! | LOGPROD | warning | no | Using 'prod' on a logical expression is hard to understand and might be incorrect. Consider using 'all' instead. |
//! | LOGMIN | warning | no | Using 'min' on a logical expression is hard to understand and might be incorrect. Consider using 'all' instead. |
//! | LOGMAX | warning | no | Using 'max' on a logical expression is hard to understand and might be incorrect. Consider using 'any' instead. |
//! | ELARLOG | warning | no | The VAR_NAME operator in the expression VAR_NAME(A VAR_NAME B) is unexpected. Should this be VAR_NAME(A) VAR_NAME B? |
//! | SHOCIRAA | warning | no | Using the VAR_NAME operator in the expression VAR_NAME(A VAR_NAME B) is probably unintended. |
//! | UNRPWR | warning | no | Consider using parentheses to explicitly specify operator precedence. |
//! | ADAPPREF | warning | no | Use app as the first argument for VAR_NAME. |
//! | KEYBOARDFUN | warning | no | Consider removing 'keyboard' function once you have finished debugging. This function may have security implications. |
//! | GVMIS | warning | no | Global variables are inefficient and make errors difficult to diagnose. Use a function with input variables instead. |
//! | PFEVB | warning | no | Using EVALIN('base') or ASSIGNIN('base') inside a PARFOR loop refers to the worker machines' base workspaces. |
//! | PFGP | warning | no | Avoid assigning to GLOBAL or PERSISTENT variable VAR_NAME inside a PARFOR loop. |
//! | PFGV | warning | no | Avoid using GLOBAL variable VAR_NAME in a PARFOR loop. |
//! | PFIIN | warning | no | The input variable VAR_NAME should be initialized before the PARFOR loop. |
//! | PFOUS | warning | no | The output variable VAR_NAME might not be used after the PARFOR loop. |
//! | PFRNI | warning | yes | The parfor loop can only use a step size of 1 or -1. |
//! | PFRIN | warning | no | The reduction variable VAR_NAME might not be set before the PARFOR loop. |
//! | PFRUS | warning | no | The reduction variable VAR_NAME might not be used after the PARFOR loop. |
//! | PFTUSW | warning | no | The temporary variable VAR_NAME might be used after the PARFOR loop on line VAR_NUMBER. The value set on this line is not available after the loop. |
//! | PFUIXW | warning | no | The index variable VAR_NAME might be used after the PARFOR loop on line VAR_NUMBER. The value set on this line is not available after the loop. |
//! | SPEVB | warning | no | Using EVALIN('base') or ASSIGNIN('base') inside an SPMD block refers to the worker machines' base workspaces. |
//! | SPGV | warning | no | Using the GLOBAL or PERSISTENT variable VAR_NAME in an SPMD block might fail because it is accessed on a worker machine. |
//! | DSPMDA | warning | no | Distributed array must be created outside of an SPMD block. |
//! | DUALC | warning | no | Command might be prematurely ended by comma. |
//! | RMFLD | warning | no | RMFIELD output must be assigned back to the structure. |
//! | RMWRN | warning | no | The warning with tag VAR_NAME has been removed from MATLAB, so this statement has no effect. |
//! | STFLD | warning | no | SETFIELD output must be assigned back to the structure. |
//! | STRSZ | warning | no | Use STRCMP to compare character vectors that can have different sizes. |
//! | ATTF | warning | no | The Code Analyzer is unable to determine if the expression assigned to the VAR_NAME attribute evaluates to true or false. |
//! | ATTOF | info | no | Setting the class attribute Abstract to false is not recommended. |
//! | MCPO | warning | no | VAR_NAME property has no effect in a value class. |
//! | MCSAC | warning | no | SetAccess cannot be set on Constant properties. |
//! | MOBSRV | info | no | Using SetObservable or GetObservable on a Constant property has no effect. |
//! | MDEPIN | warning | no | Default values should not be assigned to dependent properties because dependent properties do not store the values. |
//! | MCCPI | warning | no | Initialize the Constant property or make it an Abstract Constant property. |
//! | MGMD | warning | no | 'get' method should be implemented for each dependent property that does not also have private 'GetAccess' attribute. |
//! | MCCPE | warning | no | Attempting to call a property or event VAR_NAME as a function. |
//! | MTHANS | info | no | Using ANS as a method name is not recommended as ANS is frequently overwritten by MATLAB. |
//! | MHERM | info | no | Parenthesize the multiplication of VAR_NAME and its transpose to ensure the result is Hermitian. |
//! | MNUML | warning | no | To create a square matrix, use VAR_NAME(numel(...), numel(...)). Alternatively, use VAR_NAME(size(...)) to create an array with same size as input array. |
//! | COMPNOP | warning | yes | This logical comparison simplifies to VAR_NAME(...). Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)? |
//! | COMPNOT | warning | yes | This logical comparison simplifies to ~VAR_NAME(...). Did you mean to use VAR_NAME to evaluate function argument: VAR_NAME(...VAR_NAME...)? |
//! | M3COL | warning | no | Using three colons (a:b:c:d) in an expression is probably unintended. |
//! | BDLGI | warning | no | Variable might be set by a nonlogical operator. |
//! | BDLOG1 | warning | no | A scalar logical value is expected in the conditional expression. Use 'any' or 'all' to reduce the array to a logical scalar. |
//! | BDLOG2 | warning | no | A scalar logical value is expected in the conditional expression. Use 'any' or 'all' to reduce the array to a logical scalar, or compare the scalar value to 0. |
//! | BDSCI | warning | no | Variable might be set by a nonscalar operator. |
//! | MCHDP | warning | no | A property default value that is a handle will cause all instances to share the same object data. To avoid sharing, create the property value in the constructor. For intentional sharing, consider using a Constant property. |
//! | MCHDT | warning | no | Declaring the value of a property as a handle might cause all instances to share the same default handle. To avoid sharing, create the handle for this property in the constructor. To express that sharing is intentional, use the Constant property attribute. |
//! | SHVAU | warning | no | Confusing usage of name VAR_NAME on lines VAR_NUMBER and VAR_NUMBER. Initialize VAR_NAME before line VAR_NUMBER to make it a shared variable or rename VAR_NAME on line VAR_NUMBER to disambiguate. |
//! | GTARG | warning | no | Function might be called with too many arguments. |
//! | LTARG | warning | no | Function might be called with too few arguments. |
//! | CTPCT | warning | no | The format might not agree with the argument count. |
//! | FXSETA | warning | no | Loop index variable is changed inside of a `for` loop |
//! | SIMPT | warning | no | This import statement runs before any other code in function VAR_NAME. Consider placing it at the top of the function body. |
//! | TLEV | warning | no | VAR_NAME could be very inefficient unless it is a top-level statement in its function. |
//! | UNONC | warning | no | Assign the onCleanup output argument to a variable. Do not use the tilde operator (~) in place of a variable. |
//! | MIPC1 | warning | no | Calling the computer function with 'arch' returns 'win64', 'glnxa64', or 'maca64'. |
//! | SUBSINDEX | warning | no | Do not overload 'subsindex' for fundamental data types. |
//! | VTFIN | warning | no | VAR_NAME should be the first input argument to the VAR_NAME function. |
//! | CTOINW | warning | no | Use of constructed object as input to constructor is not necessary. |
//! | FXUP | warning | no | Outer loop variable VAR_NAME is set inside a nested function. |
//! | ADMTHDINV | warning | no | Use VAR_NAME(app, ...) to call this function. |
//! | ADPROP | warning | no | VAR_NAME is also the name of a property, which may be confusing. Use app.PropertyName syntax to reference the property, or change one of the names to improve readability. |
//! | MCNPN | warning | no | VAR_NAME is referenced but is not a property, method, or event name defined in this class. |
//! | MCNPR | warning | no | VAR_NAME is not a property, but is the target of an assignment. |
//! | MCSNOV | warning | no | Set function in value class must return the modified object. |
//! | MCSOH | warning | no | Set function in handle class does not need to return the modified object. |
//! | MCVM | warning | no | Value class method that modifies the object must return the modified object. |
//! | MCCSPS | warning | no | Constant property VAR_NAME is not modified. 'VAR_NAME.VAR_NAME' creates a struct named VAR_NAME with a field named VAR_NAME. |
//! | MCSUP | warning | no | The set method for the property VAR_NAME should not access another property (VAR_NAME). |
//!
//! ## Fix
//!
//! Rewrites the flagged construct into the cleaner equivalent:
//!
//! - `call(...) == true` → `call(...)` (COMPNOP) and
//!   `call(...) ~= true` / `call(...) == false` → `~call(...)` (COMPNOT).
//! - `parfor i = 1:step:end` → `parfor i = 1:end` (PFRNI).
//! - `%comment` → `% comment` (COMNC).
//! - `(scalar)` → `scalar` (NBRAK1).
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! if isa(x, 'double') == true      % COMPNOP
//!     disp('double');
//! end
//! parfor i = 1:2:10                % PFRNI
//!     y(i) = i;
//! end
//! %comment with no space           % COMNC
//! x = (5);                         % NBRAK1
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! if isa(x, 'double')              % COMPNOP fix applied
//!     disp('double');
//! end
//! parfor i = 1:10                  % PFRNI fix applied
//!     y(i) = i;
//! end
//! % comment with space             % COMNC fix applied
//! x = 5;                           % NBRAK1 fix applied
//! ```
//!
//! ### Fixed
//!
//! ```diff
//! - if isa(x, 'double') == true
//! + if isa(x, 'double')
//! - parfor i = 1:2:10
//! + parfor i = 1:10
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.GOOD_PRACTICES_ENGINE]
//! severity = "warning"
//! max_variable_name_length = 63
//! disabled_checks = []
//! ```

use malt_core::{Category, Config, Diagnostic, FileContext, Fix, NodeContext, Rule, Severity};
use serde::Deserialize;
use tree_sitter::Node;

use crate::analysis::metadata::{ClassMeta, FileMeta};
use crate::analysis::symbols::SymbolTable;

mod check_app_designer;
mod check_arity;
mod check_attf_attof;
mod check_chain;
mod check_comnc;
mod check_comnot;
mod check_compnop;
mod check_cprop;
mod check_ctch;
mod check_ctpct;
mod check_discouraged_functions;
mod check_display_override;
mod check_dspmda;
mod check_dualc;
mod check_elarlog;
mod check_eval;
mod check_fncolnd;
mod check_fndef;
mod check_fval;
mod check_fxset;
mod check_gvmis;
mod check_handle_defaults;
mod check_iters;
mod check_lngnm;
mod check_load;
mod check_logical_aggregation;
mod check_logical_usage;
mod check_m3col;
mod check_mccpe;
mod check_mccpi;
mod check_mcpo;
mod check_mcsac;
mod check_mdepin;
mod check_mexcep;
mod check_mgmd;
mod check_mherm;
mod check_mipc1;
mod check_misc_general;
mod check_mnuml;
mod check_mobsrv;
mod check_mthans;
mod check_nbrak1;
mod check_noans;
mod check_noin;
mod check_oop_practice;
mod check_parfor_reduction;
mod check_pfevb;
mod check_pfgp;
mod check_pfgv;
mod check_pfrni;
mod check_prop_validation;
mod check_rmwrn;
mod check_sepex;
mod check_shared_vars;
mod check_shociraa;
mod check_simpt;
mod check_spevb;
mod check_spgv;
mod check_stci;
mod check_stcmp;
mod check_stisa;
mod check_strnu;
mod check_strsz;
mod check_tlev;
mod check_trync;
mod check_unonc;
mod check_unrpwr;
mod check_valst;
mod check_warning_error_tag;

// ---------------------------------------------------------------------------
// Default configuration values
// ---------------------------------------------------------------------------

pub(crate) const fn default_max_variable_name_length() -> usize {
    63
}

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for the Good Practices engine.
///
/// Deserialized from `[lint.rules.GOOD_PRACTICES_ENGINE]` in `.malt.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct GoodPracticesConfig {
    /// Maximum variable name length before LNGNM fires.
    #[serde(default = "default_max_variable_name_length")]
    pub max_variable_name_length: usize,

    /// Check IDs to skip (e.g., `["TRYNC", "CTCH"]`).
    #[serde(default)]
    pub disabled_checks: Vec<String>,
}

impl Default for GoodPracticesConfig {
    fn default() -> Self {
        Self {
            max_variable_name_length: default_max_variable_name_length(),
            disabled_checks: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: node text extraction
// ---------------------------------------------------------------------------

/// Extract source text for a tree-sitter node.
pub(crate) fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// Parent node types indicating statement-level context.
pub(crate) const STATEMENT_PARENTS: &[&str] = &["source_file", "block"];

/// Warning message ID tags removed from MATLAB; RMWRN fires when a
/// `warning(...)` call uses one of these tags. Currently empty (placeholder):
/// no tags are populated yet, so RMWRN is inert until a tag is added here.
pub(crate) const REMOVED_WARNING_TAGS: &[&str] = &[];

// ---------------------------------------------------------------------------
// Node-level target types
// ---------------------------------------------------------------------------

/// Node types the engine subscribes to for per-node checks.
pub(crate) const TARGET_NODES: &[&str] = &[
    "function_call",
    "command",
    "try_statement",
    "catch_clause",
    "assignment",
    "binary_operator",
    "boolean_operator",
    "comparison_operator",
    "range",
    "if_statement",
    "while_statement",
    "for_statement",
    "spmd_statement",
    "comment",
    "matrix",
];

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Good Practices Engine — a hybrid rule handling 32+ good-practice checks.
///
/// Node-level checks fire on simple pattern matches during the DFS traversal.
/// File-level checks use [`FileMeta`] and [`SymbolTable`] for context-aware
/// analysis (e.g., last-statement detection, variable shadowing).
pub struct GoodPracticesEngine {
    config: GoodPracticesConfig,
}

impl GoodPracticesEngine {
    /// Factory constructor. Reads rule-specific params from config.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let rule_config: GoodPracticesConfig = config.rule_params("GOOD_PRACTICES_ENGINE");
        Box::new(Self {
            config: rule_config,
        })
    }

    /// Check whether a specific sub-check ID is enabled.
    pub(crate) fn is_check_enabled(&self, check_id: &str) -> bool {
        !self.config.disabled_checks.iter().any(|id| id == check_id)
    }

    /// WLAST: `warning('')` does not reset the warning state.
    fn check_wlast(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        if !self.is_check_enabled("WLAST") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let mut calls = Vec::new();
        collect_nodes_of_kind(tree.root_node(), "function_call", &mut calls);
        for call in calls {
            if get_function_call_name(call, source) != Some("warning") {
                continue;
            }
            if !call_has_empty_string_arg(call, source) {
                continue;
            }
            let pos = call.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "WLAST",
                message: "WARNING('') does not reset the warning state. Use LASTWARN('') instead."
                    .to_string(),
                severity: Severity::Warning,
                byte_range: call.start_byte()..call.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
        }

        diagnostics
    }

    /// RMFLD: `rmfield` result must be assigned back to the structure.
    fn check_rmfld(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        self.check_struct_assign_back(node, source, "rmfield", "RMFLD")
    }

    /// STFLD: `setfield` result must be assigned back to the structure.
    fn check_stfld(&self, node: Node, source: &str) -> Vec<Diagnostic> {
        self.check_struct_assign_back(node, source, "setfield", "STFLD")
    }

    /// Shared RMFLD/STFLD logic: fire when the call is a standalone statement
    /// (parent is `source_file`/`block`) so its result is silently discarded.
    fn check_struct_assign_back(
        &self,
        node: Node,
        source: &str,
        func_name: &str,
        check_id: &'static str,
    ) -> Vec<Diagnostic> {
        if !self.is_check_enabled(check_id) || node.kind() != "function_call" {
            return Vec::new();
        }
        if get_function_call_name(node, source) != Some(func_name) {
            return Vec::new();
        }
        if !is_statement_level(node) {
            return Vec::new();
        }

        let pos = node.start_position();
        vec![Diagnostic {
            rule_id: check_id,
            message: format!(
                "{} output must be assigned back to the structure.",
                func_name.to_uppercase()
            ),
            severity: Severity::Warning,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        }]
    }

    /// PFIIN / PFOUS / PFTUSW / PFUIXW: PARFOR loop data-flow checks.
    ///
    /// Uses the symbol table with byte-range comparisons because the symbol
    /// table has no per-parfor scope.
    fn check_parfor_file_level(&self, tree: &tree_sitter::Tree, source: &str) -> Vec<Diagnostic> {
        let any_enabled = self.is_check_enabled("PFIIN")
            || self.is_check_enabled("PFOUS")
            || self.is_check_enabled("PFTUSW")
            || self.is_check_enabled("PFUIXW");
        if !any_enabled {
            return Vec::new();
        }

        let sym = SymbolTable::build(tree, source);
        let mut diagnostics = Vec::new();
        let parfors = collect_parfor_nodes(tree.root_node(), source);

        for parfor in &parfors {
            let idx_var = extract_parfor_index_var(*parfor, source);
            let body = find_child_of_kind(*parfor, "block");

            // PFIIN: input variable read in the body but never initialized
            // before the loop and not assigned inside the loop.
            if self.is_check_enabled("PFIIN") {
                if let Some(body_node) = body {
                    let mut skip = std::collections::HashSet::new();
                    if let Some(ref idx) = idx_var {
                        skip.insert(idx.clone());
                    }
                    let reads = collect_parfor_body_reads(body_node, source, &skip);
                    for (name, read_node) in reads {
                        if is_defined_before_parfor(&sym, &name, parfor.start_byte())
                            || is_assigned_in_body(body_node, &name, source)
                        {
                            continue;
                        }
                        let pos = read_node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "PFIIN",
                            message: format!(
                                "The input variable {name} should be initialized before the PARFOR loop."
                            ),
                            severity: Severity::Warning,
                            byte_range: read_node.start_byte()..read_node.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }

            // PFOUS / PFTUSW: simple-identifier assignments in the body and
            // whether the variable is used after the loop.
            let need_assignments =
                self.is_check_enabled("PFOUS") || self.is_check_enabled("PFTUSW");
            if need_assignments {
                if let Some(body_node) = body {
                    let assigned = collect_simple_lhs_assignments(body_node, source);
                    for (name, lhs_node) in assigned {
                        if idx_var.as_deref() == Some(name.as_str()) {
                            continue;
                        }
                        let uses = uses_after_parfor(&sym, &name, parfor.end_byte(), tree, false);
                        if self.is_check_enabled("PFOUS") && uses.is_empty() {
                            let pos = lhs_node.start_position();
                            diagnostics.push(Diagnostic {
                                rule_id: "PFOUS",
                                message: format!(
                                    "The output variable {name} might not be used after the PARFOR loop."
                                ),
                                severity: Severity::Warning,
                                byte_range: lhs_node.start_byte()..lhs_node.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                        if self.is_check_enabled("PFTUSW") {
                            if let Some(line) = uses.first() {
                                let pos = lhs_node.start_position();
                                diagnostics.push(Diagnostic {
                                    rule_id: "PFTUSW",
                                    message: format!(
                                        "The temporary variable {name} might be used after the PARFOR loop on line {line}. The value set on this line is not available after the loop."
                                    ),
                                    severity: Severity::Warning,
                                    byte_range: lhs_node.start_byte()..lhs_node.end_byte(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                            }
                        }
                    }
                }
            }

            // PFUIXW: index variable used after the loop.
            if self.is_check_enabled("PFUIXW") {
                if let Some(ref idx) = idx_var {
                    let uses = uses_after_parfor(&sym, idx, parfor.end_byte(), tree, true);
                    if let Some(line) = uses.first() {
                        let idx_node = find_parfor_index_identifier(*parfor);
                        let target = idx_node.unwrap_or(*parfor);
                        let pos = target.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "PFUIXW",
                            message: format!(
                                "The index variable {idx} might be used after the PARFOR loop on line {line}. The value set on this line is not available after the loop."
                            ),
                            severity: Severity::Warning,
                            byte_range: target.start_byte()..target.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        diagnostics
    }

    /// Shared COMPNOP/COMPNOT logic: extract the `(call, literal, operator)`
    /// triple from a `comparison_operator` node when exactly one operand is a
    /// `function_call` and the other is the `true`/`false` literal.
    pub(crate) fn comparison_with_literal<'a>(
        &self,
        node: Node<'a>,
        source: &'a str,
    ) -> Option<(Node<'a>, Node<'a>, String)> {
        let lhs = node.child(0)?;
        let rhs = node.child(2)?;
        let op = find_operator_text(node, source);

        let (call_node, literal_node) = match (lhs.kind(), rhs.kind()) {
            ("function_call", "identifier") => (lhs, rhs),
            ("identifier", "function_call") => (rhs, lhs),
            _ => return None,
        };
        let literal = node_text(literal_node, source).trim();
        if literal == "true" || literal == "false" {
            Some((call_node, literal_node, op))
        } else {
            None
        }
    }
}

impl Rule for GoodPracticesEngine {
    fn id(&self) -> &'static str {
        "GOOD_PRACTICES_ENGINE"
    }

    fn description(&self) -> &'static str {
        "Common best-practice checks for MATLAB code"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn category(&self) -> Category {
        Category::GoodPractices
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        TARGET_NODES
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check(&self, ctx: &NodeContext) -> Vec<Diagnostic> {
        let node = ctx.node;
        let source = ctx.source;
        let mut diagnostics = Vec::new();

        // Error handling checks.
        diagnostics.extend(self.check_trync(node, source));
        diagnostics.extend(self.check_ctch(node, source));
        diagnostics.extend(self.check_mexcep(node, source));
        diagnostics.extend(self.check_warning_error_tag(node, source));

        // String / comparison checks.
        diagnostics.extend(self.check_stcmp(node, source));
        diagnostics.extend(self.check_stci(node, source));
        diagnostics.extend(self.check_stisa(node, source));

        // Eval / dynamic code checks.
        diagnostics.extend(self.check_eval(node, source));

        // General practice checks.
        diagnostics.extend(self.check_load(node, source));
        diagnostics.extend(self.check_lngnm(node, source));
        diagnostics.extend(self.check_nbrak1(node, source));
        diagnostics.extend(self.check_comnc(node, source));
        diagnostics.extend(self.check_elarlog(node, source));
        diagnostics.extend(self.check_logical_aggregation(node, source));
        diagnostics.extend(self.check_shociraa(node, source));
        diagnostics.extend(self.check_unrpwr(node, source));
        diagnostics.extend(self.check_discouraged_functions(node, source));
        diagnostics.extend(self.check_chain(node, source));

        // Function-call convention checks.
        diagnostics.extend(self.check_ctpct(node, source));
        diagnostics.extend(self.check_fxset(node, source));
        diagnostics.extend(self.check_simpt(node, source));
        diagnostics.extend(self.check_tlev(node, source));
        diagnostics.extend(self.check_unonc(node, source));
        diagnostics.extend(self.check_mipc1(node, source));

        // Structure / string / misc checks.
        diagnostics.extend(self.check_rmfld(node, source));
        diagnostics.extend(self.check_stfld(node, source));
        diagnostics.extend(self.check_rmwrn(node, source));
        diagnostics.extend(self.check_strsz(node, source));
        diagnostics.extend(self.check_dualc(node, source));

        // Logical / comparison / range checks.
        diagnostics.extend(self.check_compnop(node, source));
        diagnostics.extend(self.check_comnot(node, source));
        diagnostics.extend(self.check_m3col(node, source));

        // OOP / class / property checks.
        diagnostics.extend(self.check_mherm(node, source));
        diagnostics.extend(self.check_mnuml(node, source));

        // Parfor / SPMD checks.
        if node.kind() == "for_statement" || node.kind() == "spmd_statement" {
            diagnostics.extend(self.check_pfrni(node, source));
            diagnostics.extend(self.check_pfevb(node, source));
            diagnostics.extend(self.check_pfgp(node, source));
            diagnostics.extend(self.check_pfgv(node, source));
            diagnostics.extend(self.check_spevb(node, source));
            diagnostics.extend(self.check_spgv(node, source));
            diagnostics.extend(self.check_dspmda(node, source));
        }

        diagnostics
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        diagnostics.extend(self.check_wlast(ctx.tree, ctx.source));
        diagnostics.extend(self.check_display_override(ctx.tree, ctx.source));
        diagnostics.extend(self.check_noin(ctx.tree, ctx.source));
        diagnostics.extend(self.check_prop_validation(ctx.tree, ctx.source));
        diagnostics.extend(self.check_noans(ctx.tree, ctx.source));
        diagnostics.extend(self.check_cprop(ctx.tree, ctx.source));
        diagnostics.extend(self.check_iters(ctx.tree, ctx.source));
        diagnostics.extend(self.check_gvmis(ctx.tree, ctx.source));
        diagnostics.extend(self.check_sepex(ctx.tree, ctx.source));
        diagnostics.extend(self.check_strnu(ctx.tree, ctx.source));
        diagnostics.extend(self.check_fndef(ctx.tree, ctx.source));
        diagnostics.extend(self.check_valst(ctx.tree, ctx.source));
        diagnostics.extend(self.check_fval(ctx.tree, ctx.source));
        diagnostics.extend(self.check_fncolnd(ctx.tree, ctx.source));

        // OOP / class / property checks.
        diagnostics.extend(self.check_attf_attof(ctx.tree, ctx.source));
        diagnostics.extend(self.check_mcpo(ctx.tree, ctx.source));
        diagnostics.extend(self.check_mcsac(ctx.tree, ctx.source));
        diagnostics.extend(self.check_mobsrv(ctx.tree, ctx.source));
        diagnostics.extend(self.check_mdepin(ctx.tree, ctx.source));
        diagnostics.extend(self.check_mccpi(ctx.tree, ctx.source));
        diagnostics.extend(self.check_mgmd(ctx.tree, ctx.source));
        diagnostics.extend(self.check_mthans(ctx.tree, ctx.source));
        diagnostics.extend(self.check_mccpe(ctx.tree, ctx.source));

        // Parfor / SPMD file-level checks.
        diagnostics.extend(self.check_parfor_file_level(ctx.tree, ctx.source));

        // Parfor reduction-variable checks.
        diagnostics.extend(self.check_parfor_reduction(ctx.tree, ctx.source));

        // App Designer and OOP practice checks.
        diagnostics.extend(self.check_app_designer(ctx.tree, ctx.source));
        diagnostics.extend(self.check_oop_practice(ctx.tree, ctx.source));

        // Miscellaneous general practice checks.
        diagnostics.extend(self.check_misc_general(ctx.tree, ctx.source));

        // Logical-usage, handle-default, shared-variable, and arity checks.
        diagnostics.extend(self.check_logical_usage(ctx.tree, ctx.source));
        diagnostics.extend(self.check_handle_defaults(ctx.tree, ctx.source));
        diagnostics.extend(self.check_shared_vars(ctx.tree, ctx.source));
        diagnostics.extend(self.check_arity(ctx.tree, ctx.source));

        diagnostics
    }
}

inventory::submit!(crate::RuleRegistration::new(
    "GOOD_PRACTICES_ENGINE",
    GoodPracticesEngine::from_config
));

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Check whether a node has a child of the given kind.
pub(crate) fn has_child_of_kind(node: Node, kind: &str) -> bool {
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            if child.kind() == kind {
                return true;
            }
        }
    }
    false
}

/// Find the first child of a node with the given kind.
pub(crate) fn find_child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            if child.kind() == kind {
                return Some(child);
            }
        }
    }
    None
}

/// Get the first named child of a node.
pub(crate) fn first_named_child(node: Node) -> Option<Node> {
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            if child.is_named() {
                return Some(child);
            }
        }
    }
    None
}

/// Check if a node is at statement level (parent is `source_file` or `block`).
pub(crate) fn is_statement_level(node: Node) -> bool {
    node.parent()
        .map(|p| STATEMENT_PARENTS.contains(&p.kind()))
        .unwrap_or(false)
}

/// Extract the function name from a `function_call` node.
pub(crate) fn get_function_call_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    if node.kind() != "function_call" {
        return None;
    }
    let name_node = node.child_by_field_name("name")?;
    Some(node_text(name_node, source))
}

/// Check whether a function call's first argument is an empty string literal
/// (`''` or `""`).
pub(crate) fn call_has_empty_string_arg(node: Node, source: &str) -> bool {
    let Some(args) = find_child_of_kind(node, "arguments") else {
        return false;
    };
    let Some(first) = first_named_child(args) else {
        return false;
    };
    if first.kind() != "string" {
        return false;
    }
    let text = node_text(first, source).trim();
    let inner = text.trim_matches(|c| c == '\'' || c == '"');
    inner.is_empty()
}

/// Extract the command name from a `command` node.
pub(crate) fn get_command_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    if node.kind() != "command" {
        return None;
    }
    let first = node.child(0)?;
    if first.kind() == "command_name" {
        Some(node_text(first, source))
    } else {
        None
    }
}

/// Get the full arguments text from a function_call node.
pub(crate) fn get_arguments_text<'a>(node: Node<'a>, source: &'a str) -> String {
    find_child_of_kind(node, "arguments")
        .map(|a| node_text(a, source).to_string())
        .unwrap_or_default()
}

/// Find the operator text in a binary/boolean/comparison operator node.
pub(crate) fn find_operator_text<'a>(node: Node<'a>, source: &'a str) -> String {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if !child.is_named() {
            let text = &source[child.start_byte()..child.end_byte()];
            let trimmed = text.trim();
            if !trimmed.is_empty() && trimmed != "(" && trimmed != ")" && trimmed != "," {
                return trimmed.to_string();
            }
        }
    }

    if let Some(op_node) = node.child_by_field_name("operator") {
        return node_text(op_node, source).trim().to_string();
    }

    let text = node_text(node, source);
    for op in &[
        "==", "~=", ">=", "<=", "&&", "||", ">", "<", "&", "|", "+", "-", "*", "/", "^",
    ] {
        if text.contains(op) {
            return (*op).to_string();
        }
    }

    String::new()
}

/// Check if a node has any string-literal children (for STCMP).
pub(crate) fn node_has_string_child(node: Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if kind == "string" || kind == "char_vector" {
            return true;
        }
        // Also check for string literal wrapped in parens, etc.
        if child.is_named() && node_has_string_child(child) {
            return true;
        }
    }
    false
}

/// Classify the type of eval usage based on argument text patterns.
pub(crate) fn classify_eval_usage(
    args_text: &str,
    config: &GoodPracticesConfig,
) -> (&'static str, &'static str) {
    // EVLDOT: dynamic field access patterns like `eval(['s.' fieldname])`
    if (args_text.contains("s.") || args_text.contains(".("))
        && config.disabled_checks.iter().all(|c| c != "EVLDOT")
    {
        return (
            "EVLDOT",
            "'eval' is inefficient and makes code less clear. Use dynamic field names to access structure fields or object properties instead.",
        );
    }

    // EVLSYS: system command patterns
    if (args_text.contains("system") || args_text.contains("dos") || args_text.contains("unix"))
        && config.disabled_checks.iter().all(|c| c != "EVLSYS")
    {
        return (
            "EVLSYS",
            "'eval' is inefficient and makes code less clear. To make calls to the operating system use the system function instead.",
        );
    }

    // EVLEQ: dynamic variable creation like `eval([varname ' = ...'])`
    if args_text.contains(" = ") && config.disabled_checks.iter().all(|c| c != "EVLEQ") {
        return (
            "EVLEQ",
            "'eval' is inefficient and makes code less clear. Assign to the variable directly.",
        );
    }

    // EVLSEQVAR: sequential variable creation like `eval(['x' num2str(i)])`
    if (args_text.contains("num2str") || args_text.contains("int2str"))
        && config.disabled_checks.iter().all(|c| c != "EVLSEQVAR")
    {
        return (
            "EVLSEQVAR",
            "Using 'eval' to dynamically assign variables is not recommended.",
        );
    }

    // EVLCS: general eval usage (fallback)
    (
        "EVLCS",
        "'eval' is inefficient and makes code less clear. Call the statement directly.",
    )
}

/// Check if arguments contain a `lower()` or `upper()` call (for STCI).
pub(crate) fn args_has_case_conversion(args_node: Node, source: &str) -> bool {
    let mut cursor = args_node.walk();
    for child in args_node.children(&mut cursor) {
        if child.kind() == "function_call" {
            if let Some(name) = get_function_call_name(child, source) {
                if name == "lower" || name == "upper" {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if arguments contain a call to a specific function (for STISA).
pub(crate) fn args_has_function_call(args_node: Node, source: &str, target_name: &str) -> bool {
    let mut cursor = args_node.walk();
    for child in args_node.children(&mut cursor) {
        if child.kind() == "function_call" {
            if let Some(name) = get_function_call_name(child, source) {
                if name == target_name {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if a node is inside a matrix or cell context (for SHOCIRAA).
pub(crate) fn is_inside_array_context(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(p) = current {
        let kind = p.kind();
        if kind == "matrix" || kind == "cell" {
            return true;
        }
        if kind == "function_definition" || kind == "source_file" {
            break;
        }
        current = p.parent();
    }
    false
}

/// Find element-wise boolean operators (`&`, `|`) in if/while conditions.
pub(crate) fn find_element_wise_boolean_in_condition(
    node: Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // The condition of if/while is typically the first expression child.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "boolean_operator" {
            let text = node_text(child, source);
            // Single `&` or `|` (not `&&` or `||`)
            let has_element_wise = (text.contains('&') && !text.contains("&&"))
                || (text.contains('|') && !text.contains("||"));
            if has_element_wise {
                let pos = child.start_position();
                let op = if text.contains('&') { "&" } else { "|" };
                diagnostics.push(Diagnostic {
                    rule_id: "ELARLOG",
                    message: format!(
                        "The {op} operator in the expression {op}(A {op} B) is unexpected. Should this be {op}(A) {op} B?"
                    ),
                    severity: Severity::Warning,
                    byte_range: child.start_byte()..child.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }
        // Don't recurse into nested if/while/for/blocks — just check immediate condition.
        if child.kind() == "block"
            || child.kind() == "elseif_clause"
            || child.kind() == "else_clause"
        {
            continue;
        }
        find_element_wise_boolean_in_condition(child, source, diagnostics);
    }
}

/// Find `end` used as an index argument (FNCOLND).
pub(crate) fn find_end_as_index(node: Node, diagnostics: &mut Vec<Diagnostic>) {
    if node.kind() == "end" {
        // Check if parent is an indexing context (function_call arguments).
        if let Some(parent) = node.parent() {
            if parent.kind() == "arguments" {
                // Check if grandparent is a function_call (which could be indexing).
                if let Some(gp) = parent.parent() {
                    if gp.kind() == "function_call" {
                        // Count the number of argument separators (commas) to determine
                        // if this is a multi-dimensional index. `end` without explicit
                        // dimension is fine for 1-D indexing but ambiguous for multi-D.
                        let comma_count = count_children_of_kind(parent, ",");
                        if comma_count > 0 {
                            let pos = node.start_position();
                            diagnostics.push(Diagnostic {
                                rule_id: "FNCOLND",
                                message: "Consider explicitly defining the array, and then using the END operator to index into it.".to_string(),
                                severity: Severity::Warning,
                                byte_range: node.start_byte()..node.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    }
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_end_as_index(child, diagnostics);
    }
}

/// Count children of a specific kind.
pub(crate) fn count_children_of_kind(node: Node, kind: &str) -> usize {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|c| c.kind() == kind)
        .count()
}

/// Get the root node of a tree by walking up from a node.
pub(crate) fn root_node_of<'a>(mut node: Node<'a>) -> Node<'a> {
    while let Some(p) = node.parent() {
        node = p;
    }
    node
}

/// Check whether a `for_statement` node is a parfor loop.
pub(crate) fn is_parfor_node(node: Node, source: &str) -> bool {
    node.kind() == "for_statement" && node_text(node, source).trim_start().starts_with("parfor")
}

/// Count the named children of a node.
pub(crate) fn count_named_children(node: Node) -> usize {
    let mut cursor = node.walk();
    node.children(&mut cursor).filter(|c| c.is_named()).count()
}

/// Get the `range` node of a for/parfor statement's iterator.
pub(crate) fn find_iterator_range(node: Node) -> Option<Node> {
    let iter = find_child_of_kind(node, "iterator")?;
    find_child_of_kind(iter, "range")
}

/// Extract the parfor index variable name.
pub(crate) fn extract_parfor_index_var(node: Node, source: &str) -> Option<String> {
    find_parfor_index_identifier(node).map(|n| node_text(n, source).to_string())
}

/// Find the parfor index variable identifier node.
pub(crate) fn find_parfor_index_identifier(node: Node) -> Option<Node> {
    let iter = find_child_of_kind(node, "iterator")?;
    let count = iter.child_count();
    for i in 0..count {
        if let Some(c) = iter.child(i) {
            if c.kind() == "identifier" {
                return Some(c);
            }
        }
    }
    None
}

/// Walk a node collecting all `function_call` and `command` descendants.
pub(crate) fn walk_body_for_function_calls<'a>(node: Node<'a>, out: &mut Vec<Node<'a>>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "function_call" || child.kind() == "command" {
            out.push(child);
        }
        if child.is_named() {
            walk_body_for_function_calls(child, out);
        }
    }
}

/// Collect all descendants of a node with the given kind.
pub(crate) fn collect_nodes_of_kind<'a>(node: Node<'a>, kind: &str, out: &mut Vec<Node<'a>>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == kind {
            out.push(child);
        }
        if child.is_named() {
            collect_nodes_of_kind(child, kind, out);
        }
    }
}

/// Collect all parfor `for_statement` nodes in a tree.
pub(crate) fn collect_parfor_nodes<'a>(root: Node<'a>, source: &str) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    if is_parfor_node(root, source) {
        out.push(root);
    }
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.is_named() {
            out.extend(collect_parfor_nodes(child, source));
        }
    }
    out
}

/// Collect all identifiers declared `global` or `persistent` anywhere in a tree.
pub(crate) fn collect_global_persistent_vars(
    root: Node,
    source: &str,
) -> std::collections::HashSet<String> {
    let mut vars = std::collections::HashSet::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "global_operator" || child.kind() == "persistent_operator" {
            let mut c2 = child.walk();
            for id in child.children(&mut c2) {
                if id.kind() == "identifier" {
                    vars.insert(node_text(id, source).to_string());
                }
            }
        }
        if child.is_named() {
            vars.extend(collect_global_persistent_vars(child, source));
        }
    }
    vars
}

/// Check whether a function call / command is `evalin`/`assignin` with `'base'`.
pub(crate) fn is_evalin_assignin_base(node: Node, source: &str) -> bool {
    let func_name = match node.kind() {
        "function_call" => get_function_call_name(node, source),
        "command" => get_command_name(node, source),
        _ => return false,
    };
    let func_name = match func_name {
        Some(n) => n,
        None => return false,
    };
    if func_name != "evalin" && func_name != "assignin" {
        return false;
    }

    let first_arg = if node.kind() == "function_call" {
        find_child_of_kind(node, "arguments").and_then(|a| first_named_child(a))
    } else {
        find_child_of_kind(node, "command_argument")
    };
    let first_arg = match first_arg {
        Some(a) => a,
        None => return false,
    };
    let text = node_text(first_arg, source).trim();
    let text = text.trim_matches('\'').trim_matches('"');
    text == "base"
}

/// Check whether a function call / command constructs a distributed array.
pub(crate) fn is_distributed_call(node: Node, source: &str) -> bool {
    let func_name = match node.kind() {
        "function_call" => get_function_call_name(node, source),
        "command" => get_command_name(node, source),
        _ => return false,
    };
    matches!(
        func_name,
        Some("distributed") | Some("gpuArray") | Some("codistributed")
    )
}

/// Extract the base variable name assigned by an assignment LHS.
pub(crate) fn lhs_base_name(lhs: Node, source: &str) -> Option<String> {
    match lhs.kind() {
        "identifier" => Some(node_text(lhs, source).to_string()),
        "function_call" => get_function_call_name(lhs, source).map(|s| s.to_string()),
        "field_expression" => {
            find_child_of_kind(lhs, "identifier").map(|n| node_text(n, source).to_string())
        }
        _ => None,
    }
}

/// Emit use-of-global diagnostics for a body node (PFGV / SPGV).
///
/// When `skip_writes` is true, assignment LHS identifiers are excluded
/// (they are handled by the separate PFGP check).
pub(crate) fn collect_global_use_diagnostics<'a>(
    body: Node<'a>,
    source: &str,
    globals: &std::collections::HashSet<String>,
    rule_id: &'static str,
    skip_writes: bool,
) -> Vec<Diagnostic> {
    let mut seen = std::collections::HashSet::new();
    let mut diagnostics = Vec::new();
    let mut identifiers = Vec::new();
    collect_nodes_of_kind(body, "identifier", &mut identifiers);
    for id in identifiers {
        let name = node_text(id, source).to_string();
        if !globals.contains(&name) {
            continue;
        }
        if let Some(parent) = id.parent() {
            match parent.kind() {
                "global_operator" | "persistent_operator" => continue,
                "assignment"
                    if skip_writes
                        && parent
                            .child_by_field_name("left")
                            .map(|l| l.id() == id.id())
                            .unwrap_or(false) =>
                {
                    continue;
                }
                "field_expression"
                    if parent
                        .child_by_field_name("field")
                        .map(|f| f.id() == id.id())
                        .unwrap_or(false) =>
                {
                    continue;
                }
                _ => {}
            }
        }
        if !seen.insert(name.clone()) {
            continue;
        }
        let pos = id.start_position();
        let message = if rule_id == "PFGV" {
            format!("Avoid using GLOBAL variable {name} in a PARFOR loop.")
        } else {
            format!(
                "Using the GLOBAL or PERSISTENT variable {name} in an SPMD block might fail because it is accessed on a worker machine."
            )
        };
        diagnostics.push(Diagnostic {
            rule_id,
            message,
            severity: Severity::Warning,
            byte_range: id.start_byte()..id.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        });
    }
    diagnostics
}

/// Collect plain-identifier reads inside a parfor body (for PFIIN).
///
/// Function-call names, assignment LHS writes, field names, and declarations
/// are excluded; only genuine variable reads are returned, deduplicated by name.
pub(crate) fn collect_parfor_body_reads<'a>(
    node: Node<'a>,
    source: &str,
    skip: &std::collections::HashSet<String>,
) -> Vec<(String, Node<'a>)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut identifiers = Vec::new();
    collect_nodes_of_kind(node, "identifier", &mut identifiers);
    for id in identifiers {
        let name = node_text(id, source).to_string();
        if skip.contains(&name) {
            continue;
        }
        if let Some(parent) = id.parent() {
            match parent.kind() {
                "function_call"
                    if parent
                        .child_by_field_name("name")
                        .map(|n| n.id() == id.id())
                        .unwrap_or(false) =>
                {
                    continue;
                }
                "assignment"
                    if parent
                        .child_by_field_name("left")
                        .map(|l| l.id() == id.id())
                        .unwrap_or(false) =>
                {
                    continue;
                }
                "global_operator" | "persistent_operator" => continue,
                "field_expression"
                    if parent
                        .child_by_field_name("field")
                        .map(|f| f.id() == id.id())
                        .unwrap_or(false) =>
                {
                    continue;
                }
                "iterator" => continue,
                _ => {}
            }
        }
        if seen.insert(name.clone()) {
            out.push((name, id));
        }
    }
    out
}

/// Collect simple-identifier LHS assignments inside a node (for PFOUS/PFTUSW).
pub(crate) fn collect_simple_lhs_assignments<'a>(
    node: Node<'a>,
    source: &str,
) -> Vec<(String, Node<'a>)> {
    let mut out = Vec::new();
    let mut assignments = Vec::new();
    collect_nodes_of_kind(node, "assignment", &mut assignments);
    for assign in assignments {
        if let Some(lhs) = assign.child_by_field_name("left") {
            if lhs.kind() == "identifier" {
                out.push((node_text(lhs, source).to_string(), lhs));
            }
        }
    }
    out
}

/// Check whether a variable is assigned anywhere inside a node.
pub(crate) fn is_assigned_in_body(node: Node, name: &str, source: &str) -> bool {
    collect_simple_lhs_assignments(node, source)
        .iter()
        .any(|(n, _)| n == name)
}

/// Check whether a variable is defined before a parfor loop starts.
///
/// Function inputs and global/persistent declarations always count as defined;
/// other definitions count if their byte range ends before the loop.
pub(crate) fn is_defined_before_parfor(sym: &SymbolTable, name: &str, parfor_start: usize) -> bool {
    sym.scopes.iter().any(|s| {
        s.defs.iter().any(|d| {
            d.name == name
                && (matches!(
                    d.kind,
                    crate::analysis::symbols::DefKind::InputArg
                        | crate::analysis::symbols::DefKind::Global
                        | crate::analysis::symbols::DefKind::Persistent
                ) || d.byte_range.end <= parfor_start)
        })
    })
}

/// Collect lines where a variable is used after a byte offset.
///
/// When `exclude_loops` is true, uses inside any enclosing `for_statement`
/// are ignored (the variable is re-bound by a subsequent loop).
pub(crate) fn uses_after_parfor(
    sym: &SymbolTable,
    name: &str,
    after: usize,
    tree: &tree_sitter::Tree,
    exclude_loops: bool,
) -> Vec<usize> {
    let mut lines = Vec::new();
    for scope in &sym.scopes {
        for u in &scope.uses {
            if u.name != name || u.byte_range.start < after {
                continue;
            }
            if exclude_loops {
                if let Some(n) = tree
                    .root_node()
                    .descendant_for_byte_range(u.byte_range.start, u.byte_range.end)
                {
                    if is_inside_loop(n) {
                        continue;
                    }
                }
            }
            lines.push(u.line);
        }
    }
    lines.sort_unstable();
    lines.dedup();
    lines
}

/// Check whether a node is enclosed by a `for_statement` (loop body).
pub(crate) fn is_inside_loop(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(p) = current {
        if p.kind() == "for_statement" {
            return true;
        }
        if p.kind() == "function_definition" || p.kind() == "source_file" {
            return false;
        }
        current = p.parent();
    }
    false
}

/// Find the nearest enclosing `function_definition` of a node (for SIMPT).
pub(crate) fn enclosing_function(node: Node) -> Option<Node> {
    let mut current = node.parent();
    while let Some(p) = current {
        if p.kind() == "function_definition" {
            return Some(p);
        }
        current = p.parent();
    }
    None
}

/// Count `printf`-style format specifiers in a format string (for CTPCT).
///
/// Counts `%` followed by a conversion character. Escaped `%%`, `%*` width
/// specifiers, and `%n$` positional specifiers are not counted.
pub(crate) fn count_format_specifiers(fmt: &str) -> usize {
    let chars: Vec<char> = fmt.chars().collect();
    let mut count = 0;
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '%' {
            i += 1;
            continue;
        }
        let j = i + 1;
        if j >= chars.len() {
            break;
        }
        if chars[j] == '%' {
            i = j + 1;
            continue;
        }
        if chars[j] == '*' {
            let mut k = j + 1;
            while k < chars.len() && !chars[k].is_ascii_alphabetic() {
                k += 1;
            }
            i = k + 1;
            continue;
        }
        let mut k = j;
        while k < chars.len() && chars[k].is_ascii_digit() {
            k += 1;
        }
        if k < chars.len() && chars[k] == '$' {
            let mut m = k + 1;
            while m < chars.len() && !chars[m].is_ascii_alphabetic() {
                m += 1;
            }
            i = m + 1;
            continue;
        }
        while k < chars.len() && "-+ #0".contains(chars[k]) {
            k += 1;
        }
        while k < chars.len() && chars[k].is_ascii_digit() {
            k += 1;
        }
        if k < chars.len() && chars[k] == '.' {
            k += 1;
            while k < chars.len() && chars[k].is_ascii_digit() {
                k += 1;
            }
        }
        if k < chars.len() && chars[k].is_ascii_alphabetic() {
            count += 1;
            i = k + 1;
        } else {
            i = j;
        }
    }
    count
}

/// Collect assignments to `var_name` inside a loop body, without descending
/// into nested function definitions or anonymous functions (for FXSET).
pub(crate) fn collect_loop_assignments(
    node: Node,
    var_name: &str,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "function_definition" || node.kind() == "lambda" {
        return;
    }
    if node.kind() == "assignment" {
        if let Some(left) = node.child_by_field_name("left") {
            if left.kind() == "identifier" && node_text(left, source) == var_name {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "FXSETA",
                    message: format!("Loop index {var_name} is changed inside of a FOR loop."),
                    severity: Severity::Warning,
                    byte_range: node.start_byte()..node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_loop_assignments(child, var_name, source, diagnostics);
    }
}

/// Find the `attribute` node with the given name inside a node that has an
/// `attributes` child (e.g., `class_definition` or a `properties` block).
pub(crate) fn find_attribute_node<'a>(
    node: Node<'a>,
    name: &str,
    source: &'a str,
) -> Option<Node<'a>> {
    let attrs = find_child_of_kind(node, "attributes")?;
    let count = attrs.child_count();
    for i in 0..count {
        let child = attrs.child(i)?;
        if child.kind() != "attribute" {
            continue;
        }
        if find_child_of_kind(child, "identifier")
            .map(|id| node_text(id, source) == name)
            .unwrap_or(false)
        {
            return Some(child);
        }
    }
    None
}

/// Compute a diagnostic position for the `index`-th `properties` block of a
/// class, falling back to the class definition range if the block node cannot
/// be located.
pub(crate) fn block_diagnostic_position(
    class_node: Option<Node>,
    index: usize,
    class: &ClassMeta,
    _source: &str,
) -> (usize, usize, usize, usize) {
    let block_node = class_node.and_then(|cn| find_nth_properties_block(cn, index));
    match block_node {
        Some(bn) => (
            bn.start_byte(),
            bn.end_byte(),
            bn.start_position().row + 1,
            bn.start_position().column + 1,
        ),
        None => (class.byte_range.start, class.byte_range.end, class.line, 1),
    }
}

/// Find the `index`-th child of kind `properties` under a class definition.
pub(crate) fn find_nth_properties_block<'a>(
    class_node: Node<'a>,
    index: usize,
) -> Option<Node<'a>> {
    let mut seen = 0;
    let count = class_node.child_count();
    for i in 0..count {
        let child = class_node.child(i)?;
        if child.kind() == "properties" {
            if seen == index {
                return Some(child);
            }
            seen += 1;
        }
    }
    None
}

/// Walk a class definition and emit MCCPE diagnostics for `function_call`
/// nodes whose name matches a property or event name.
pub(crate) fn collect_property_event_calls(
    node: Node,
    source: &str,
    names: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "function_call" {
        if let Some(name) = get_function_call_name(node, source) {
            if names.contains(name) {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "MCCPE",
                    message: format!(
                        "Attempting to call a property or event {name} as a function."
                    ),
                    severity: Severity::Warning,
                    byte_range: node.start_byte()..node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_property_event_calls(child, source, names, diagnostics);
    }
}

#[cfg(test)]
use tree_sitter::Parser;

/// Parse MATLAB source and return the tree.
#[cfg(test)]
pub(crate) fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_matlab::LANGUAGE.into())
        .expect("failed to load tree-sitter-matlab");
    parser.parse(source, None).expect("parse failed")
}

/// Create an engine with default config.
#[cfg(test)]
pub(crate) fn engine() -> GoodPracticesEngine {
    GoodPracticesEngine {
        config: GoodPracticesConfig::default(),
    }
}

/// Recursively find the first descendant of a node with the given kind.
#[cfg(test)]
pub(crate) fn find_descendant_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_descendant_of_kind(child, kind) {
            return Some(found);
        }
    }
    None
}

/// Find all descendant nodes of the given kind.
#[cfg(test)]
pub(crate) fn all_descendants_of_kind<'a>(node: Node<'a>, kind: &str) -> Vec<Node<'a>> {
    let mut result = Vec::new();
    if node.kind() == kind {
        result.push(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        result.extend(all_descendants_of_kind(child, kind));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The twelve OOP/class/property check IDs implemented by GOODPRAC G1.
    const OOP_CHECK_IDS: &[&str] = &[
        "ATTF", "ATTOF", "MCPO", "MCSAC", "MOBSRV", "MDEPIN", "MCCPI", "MGMD", "MCCPE", "MTHANS",
        "MHERM", "MNUML",
    ];

    /// The 11 parfor/spmd check IDs implemented by this group.
    const G2_IDS: &[&str] = &[
        "PFEVB", "PFGP", "PFGV", "PFIIN", "PFOUS", "PFRNI", "PFTUSW", "PFUIXW", "SPEVB", "SPGV",
        "DSPMDA",
    ];

    fn g2_collect_ids(
        eng: &GoodPracticesEngine,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<&'static str> {
        let mut ids = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            let ctx = NodeContext {
                node: n,
                source,
                file_path: std::path::Path::new("test.m"),
            };
            ids.extend(eng.check(&ctx).iter().map(|d| d.rule_id));
            let mut cursor = n.walk();
            for c in n.children(&mut cursor) {
                stack.push(c);
            }
        }
        let fc = FileContext {
            tree,
            source,
            file_path: std::path::Path::new("test.m"),
        };
        ids.extend(eng.check_file(&fc).iter().map(|d| d.rule_id));
        ids
    }

    fn g2_file_ids_with(
        eng: &GoodPracticesEngine,
        tree: &tree_sitter::Tree,
        source: &str,
        check_id: &str,
    ) -> Vec<&'static str> {
        eng.check_parfor_file_level(tree, source)
            .iter()
            .filter(|d| d.rule_id == check_id)
            .map(|d| d.rule_id)
            .collect()
    }

    #[test]
    fn test_rmfld_fires_on_unassigned_call() {
        let source = "rmfield(s, 'a');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_child_of_kind(root, "function_call").unwrap();
        let diags = eng.check_rmfld(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "RMFLD");
    }

    #[test]
    fn test_rmfld_silent_when_assigned() {
        let source = "s = rmfield(s, 'a');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let assignment = find_child_of_kind(root, "assignment").unwrap();
        let mut cursor = assignment.walk();
        let fc = assignment
            .children(&mut cursor)
            .find(|c| c.kind() == "function_call")
            .unwrap();
        let diags = eng.check_rmfld(fc, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_stfld_fires_on_unassigned_call() {
        let source = "setfield(s, 'a', 1);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let fc = find_child_of_kind(root, "function_call").unwrap();
        let diags = eng.check_stfld(fc, source);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule_id, "STFLD");
    }

    #[test]
    fn test_stfld_silent_when_assigned() {
        let source = "s = setfield(s, 'a', 1);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let eng = engine();

        let assignment = find_child_of_kind(root, "assignment").unwrap();
        let mut cursor = assignment.walk();
        let fc = assignment
            .children(&mut cursor)
            .find(|c| c.kind() == "function_call")
            .unwrap();
        let diags = eng.check_stfld(fc, source);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_g5_checks_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec![
                    "RMFLD".to_string(),
                    "STFLD".to_string(),
                    "STRSZ".to_string(),
                    "DUALC".to_string(),
                ],
            },
        };

        let source = "rmfield(s, 'a');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let fc = find_child_of_kind(root, "function_call").unwrap();
        assert!(eng.check_rmfld(fc, source).is_empty());
        assert!(eng.check_stfld(fc, source).is_empty());

        let source = "x = 'abc' == 'abcd';\n";
        let tree = parse(source);
        let root = tree.root_node();
        let comp = find_descendant_of_kind(root, "comparison_operator").unwrap();
        assert!(eng.check_strsz(comp, source).is_empty());

        let source = "disp hello, disp world\n";
        let tree = parse(source);
        let root = tree.root_node();
        let cmd = find_child_of_kind(root, "command").unwrap();
        assert!(eng.check_dualc(cmd, source).is_empty());
    }

    #[test]
    fn test_g3_target_nodes_include_range() {
        assert!(TARGET_NODES.contains(&"range"));
        assert!(TARGET_NODES.contains(&"comparison_operator"));
    }

    #[test]
    fn test_g3_checks_fire_through_dispatch() {
        let source =
            "if isa(x,'double') == true\nend\nif isa(x,'double') ~= true\nend\na = 1:2:3:4;\n";
        let tree = parse(source);
        let eng = engine();

        let mut ids = Vec::new();
        collect_diagnostics(&eng, tree.root_node(), source, &mut ids);
        assert!(ids.iter().any(|id| id == &"COMPNOP"), "got: {ids:?}");
        assert!(ids.iter().any(|id| id == &"COMPNOT"), "got: {ids:?}");
        assert!(ids.iter().any(|id| id == &"M3COL"), "got: {ids:?}");
    }

    #[test]
    fn test_g3_checks_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec![
                    "COMPNOP".to_string(),
                    "COMPNOT".to_string(),
                    "M3COL".to_string(),
                ],
            },
        };

        let source =
            "if isa(x,'double') == true\nend\nif isa(x,'double') ~= true\nend\na = 1:2:3:4;\n";
        let tree = parse(source);
        let mut ids = Vec::new();
        collect_diagnostics(&eng, tree.root_node(), source, &mut ids);
        assert!(ids.iter().all(|id| id != &"COMPNOP"), "got: {ids:?}");
        assert!(ids.iter().all(|id| id != &"COMPNOT"), "got: {ids:?}");
        assert!(ids.iter().all(|id| id != &"M3COL"), "got: {ids:?}");
    }

    #[test]
    fn test_g1_node_checks_fire_through_dispatch() {
        let source = "classdef Foo\n    methods\n        function z = compute(obj, x)\n            z = x * x';\n        end\n    end\nend\ny = zeros(numel(x));\n";
        let tree = parse(source);
        let eng = engine();

        let mut ids = Vec::new();
        collect_diagnostics(&eng, tree.root_node(), source, &mut ids);
        assert!(ids.iter().any(|id| id == &"MHERM"), "got: {ids:?}");
        assert!(ids.iter().any(|id| id == &"MNUML"), "got: {ids:?}");
    }

    #[test]
    fn test_g1_file_checks_fire_through_dispatch() {
        let source = "classdef ValClass\n    properties (SetObservable)\n        Data\n    end\n    properties (Constant, SetAccess = public)\n        X = 1\n    end\n    properties (Constant, GetObservable)\n        W\n    end\n    properties (Constant)\n        Z\n    end\n    properties (Dependent)\n        Y = 5\n    end\nend\n";
        let tree = parse(source);
        let eng = engine();
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: std::path::Path::new("test.m"),
        };
        let ids: Vec<&str> = eng.check_file(&ctx).iter().map(|d| d.rule_id).collect();
        for expected in ["MCPO", "MCSAC", "MOBSRV", "MCCPI", "MDEPIN", "MGMD"] {
            assert!(
                ids.iter().any(|id| id == &expected),
                "missing {expected}, got: {ids:?}"
            );
        }
    }

    #[test]
    fn test_g1_checks_silent_on_valid_source() {
        let source = "classdef (Abstract = true) Foo\n    properties X = 42\n    end\n    methods\n        function z = compute(obj, x)\n            z = (x * x');\n        end\n    end\nend\ny = zeros(3, 3);\n";
        let tree = parse(source);
        let eng = engine();

        let mut ids = Vec::new();
        collect_diagnostics(&eng, tree.root_node(), source, &mut ids);
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: std::path::Path::new("test.m"),
        };
        ids.extend(eng.check_file(&ctx).iter().map(|d| d.rule_id));

        let oop_hits: Vec<&str> = ids
            .iter()
            .copied()
            .filter(|id| OOP_CHECK_IDS.contains(id))
            .collect();
        assert!(
            oop_hits.is_empty(),
            "unexpected OOP diagnostics: {oop_hits:?}"
        );
    }

    #[test]
    fn test_g4_checks_fire_through_dispatch() {
        let source = "for i = 1:10\n    i = 5;\nend\n\
function f()\n    x = 1;\n    import foo.bar\nend\n\
eval(x + y);\nonCleanup(@f);\n[~] = onCleanup(@f);\nc = computer('arch');\nfprintf('%d %d', x);\n";
        let tree = parse(source);
        let eng = engine();

        let mut ids = Vec::new();
        collect_diagnostics(&eng, tree.root_node(), source, &mut ids);
        for want in ["FXSETA", "SIMPT", "TLEV", "UNONC", "MIPC1", "CTPCT"] {
            assert!(
                ids.iter().any(|id| id == &want),
                "expected {want} to fire, got: {ids:?}"
            );
        }
    }

    #[test]
    fn test_g4_checks_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec![
                    "CTPCT".to_string(),
                    "FXSETA".to_string(),
                    "SIMPT".to_string(),
                    "TLEV".to_string(),
                    "UNONC".to_string(),
                    "MIPC1".to_string(),
                ],
            },
        };

        let source = "fprintf('%d %d', x);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        assert!(eng.check_ctpct(fc, source).is_empty());

        let source = "for i = 1:10\n  i = 5;\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let for_node = find_child_of_kind(root, "for_statement").unwrap();
        assert!(eng.check_fxset(for_node, source).is_empty());

        let source = "function f()\n  x = 1;\n  import foo.bar\nend\n";
        let tree = parse(source);
        let root = tree.root_node();
        let cmd = find_descendant_of_kind(root, "command").unwrap();
        assert!(eng.check_simpt(cmd, source).is_empty());

        let source = "eval(x + y);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        assert!(eng.check_tlev(fc, source).is_empty());

        let source = "onCleanup(@f);\n";
        let tree = parse(source);
        let root = tree.root_node();
        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        assert!(eng.check_unonc(fc, source).is_empty());

        let source = "c = computer('arch');\n";
        let tree = parse(source);
        let root = tree.root_node();
        let fc = find_descendant_of_kind(root, "function_call").unwrap();
        assert!(eng.check_mipc1(fc, source).is_empty());
    }

    #[test]
    fn test_pfiin_fires_on_uninitialized_input() {
        let source = "parfor i = 1:10\n    q = z + i;\nend\n";
        let tree = parse(source);
        let eng = engine();

        let ids = g2_file_ids_with(&eng, &tree, source, "PFIIN");
        assert_eq!(ids.len(), 1);
        let diag = eng
            .check_parfor_file_level(&tree, source)
            .into_iter()
            .find(|d| d.rule_id == "PFIIN")
            .unwrap();
        assert!(diag.message.contains("z"));
    }

    #[test]
    fn test_pfiin_silent_when_initialized_or_input_arg() {
        let eng = engine();

        let source = "z = 0;\nparfor i = 1:10\n    q = z + i;\nend\n";
        let tree = parse(source);
        assert!(g2_file_ids_with(&eng, &tree, source, "PFIIN").is_empty());

        let source = "function f(z)\nparfor i = 1:10\n    q = z + i;\nend\nend\n";
        let tree = parse(source);
        assert!(g2_file_ids_with(&eng, &tree, source, "PFIIN").is_empty());
    }

    #[test]
    fn test_pfous_fires_on_output_not_used_after() {
        let source = "parfor i = 1:10\n    q = i;\nend\n";
        let tree = parse(source);
        let eng = engine();

        let ids = g2_file_ids_with(&eng, &tree, source, "PFOUS");
        assert_eq!(ids.len(), 1);
        let diag = eng
            .check_parfor_file_level(&tree, source)
            .into_iter()
            .find(|d| d.rule_id == "PFOUS")
            .unwrap();
        assert!(diag.message.contains("q"));
    }

    #[test]
    fn test_pfous_silent_when_used_after() {
        let source = "parfor i = 1:10\n    q = i;\nend\ndisp(q);\n";
        let tree = parse(source);
        let eng = engine();

        assert!(g2_file_ids_with(&eng, &tree, source, "PFOUS").is_empty());
    }

    #[test]
    fn test_pftusw_fires_on_temp_used_after() {
        let source = "parfor i = 1:10\n    tmp = compute(i);\n    x(i) = tmp;\nend\ndisp(tmp);\n";
        let tree = parse(source);
        let eng = engine();

        let ids = g2_file_ids_with(&eng, &tree, source, "PFTUSW");
        assert_eq!(ids.len(), 1);
        let diag = eng
            .check_parfor_file_level(&tree, source)
            .into_iter()
            .find(|d| d.rule_id == "PFTUSW")
            .unwrap();
        assert!(diag.message.contains("on line 5"), "got: {}", diag.message);
    }

    #[test]
    fn test_pftusw_silent_without_after_use() {
        let source = "parfor i = 1:10\n    tmp = compute(i);\n    x(i) = tmp;\nend\n";
        let tree = parse(source);
        let eng = engine();

        assert!(g2_file_ids_with(&eng, &tree, source, "PFTUSW").is_empty());
    }

    #[test]
    fn test_pfuiwx_fires_on_index_used_after() {
        let source = "parfor i = 1:10\n    x(i) = i;\nend\ndisp(i);\n";
        let tree = parse(source);
        let eng = engine();

        let ids = g2_file_ids_with(&eng, &tree, source, "PFUIXW");
        assert_eq!(ids.len(), 1);
        let diag = eng
            .check_parfor_file_level(&tree, source)
            .into_iter()
            .find(|d| d.rule_id == "PFUIXW")
            .unwrap();
        assert!(diag.message.contains("on line 4"), "got: {}", diag.message);
    }

    #[test]
    fn test_pfuiwx_silent_without_after_use() {
        let source = "parfor i = 1:10\n    x(i) = i;\nend\n";
        let tree = parse(source);
        let eng = engine();

        assert!(g2_file_ids_with(&eng, &tree, source, "PFUIXW").is_empty());
    }

    #[test]
    fn test_g2_checks_respect_disabled_checks() {
        let eng = GoodPracticesEngine {
            config: GoodPracticesConfig {
                max_variable_name_length: 63,
                disabled_checks: vec!["PFRNI".to_string(), "PFUIXW".to_string()],
            },
        };

        let source = "parfor i = 1:2:10\n    evalin('base','x');\n    x(i) = i;\nend\ndisp(i);\n";
        let tree = parse(source);
        let ids = g2_collect_ids(&eng, &tree, source);
        assert!(ids.iter().all(|id| id != &"PFRNI"), "got: {ids:?}");
        assert!(ids.iter().all(|id| id != &"PFUIXW"), "got: {ids:?}");
        assert!(ids.iter().any(|id| id == &"PFEVB"), "got: {ids:?}");
    }

    #[test]
    fn test_g2_must_fire_example() {
        let eng = engine();
        let source = "parfor i = 1:2:10\n    evalin('base','x');\n    x(i) = i;\nend\nparfor i = 1:10\n    tmp = compute(i);\n    x(i) = tmp;\nend\ndisp(tmp);\nparfor i = 1:10\n    x(i) = i;\nend\ndisp(i);\nspmd\n    d = distributed(zeros(100));\nend\n";
        let tree = parse(source);
        let ids = g2_collect_ids(&eng, &tree, source);
        for must in ["PFRNI", "PFEVB", "PFTUSW", "PFUIXW", "DSPMDA"] {
            assert!(ids.contains(&must), "{must} missing from: {ids:?}");
        }
    }

    #[test]
    fn test_g2_must_not_fire_example() {
        let eng = engine();
        let source = "parfor i = 1:10\n    evalin('caller','x');\n    x(i) = i;\nend\ny = zeros(1,10);\nparfor i = 1:10\n    x(i) = y(i);\nend\nd = distributed(zeros(100));\nspmd\n    work_with(d);\nend\n";
        let tree = parse(source);
        let ids = g2_collect_ids(&eng, &tree, source);
        let g2_fired: Vec<&str> = ids
            .iter()
            .copied()
            .filter(|id| G2_IDS.contains(id))
            .collect();
        assert!(g2_fired.is_empty(), "unexpected G2 fires: {g2_fired:?}");
    }

    fn collect_diagnostics(
        eng: &GoodPracticesEngine,
        node: Node,
        source: &str,
        ids: &mut Vec<&'static str>,
    ) {
        let ctx = NodeContext {
            node,
            source,
            file_path: std::path::Path::new("test.m"),
        };
        ids.extend(eng.check(&ctx).iter().map(|d| d.rule_id));
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            collect_diagnostics(eng, child, source, ids);
        }
    }
}
