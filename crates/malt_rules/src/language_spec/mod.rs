//! # LANGUAGE_SPEC_ENGINE: Language Specification Errors
//!
//! ```malt
//! id = "LANGUAGE_SPEC_ENGINE"
//! title = "Language Specification Errors"
//! category = "language-specification"
//! severity = "error"
//! fix = true
//! icon = "lucide/book-marked"
//! slug = "language-spec"
//! ```
//!
//! ## Rule
//!
//! Enforces MATLAB language-specification constraints from the Code Analyzer.
//! All 165 checks share a single file-level engine (`LanguageSpecEngine`) that
//! performs a full DFS traversal with context stacks for parfor/spmd/loop
//! detection, then runs class, arguments-block, global/persistent, and
//! script-level checks over extracted file metadata. Each diagnostic carries
//! the specific check ID (e.g. `PFPF`, `MCFIL`).
//!
//! ## Fix
//!
//! MCFIL renames the `classdef` and every constructor method to the file
//! name, in one edit — as MATLAB's own fix does (`classdef Cls` in
//! `t__Cls.m` becomes `classdef t__Cls`, and the constructor follows). The
//! rewrite is UNSAFE: a method that was never the constructor (its name did
//! not match the class the runtime uses, which is the file name) becomes
//! one, so constructing the object starts running it; it is applied only
//! with `--fix --unsafe-fixes`. Every other check of this engine reports
//! without a fix.
//!
//! The checks are grouped into families:
//!
//! - **PF\*** — parfor restrictions, banned functions, and sliced-variable /
//!   reduction classification.
//! - **SP\*** — spmd block restrictions (`SPNST`, `SPRET`, `SPGP`, and the
//!   transparency group `SPDEC`, `SPEVC`, `SPLD`, `SPNF`, `SPSV`, `SPWHOS`,
//!   `SPBFN`).
//! - **AT\*** — class, events, property, and method attribute validation.
//! - **CLS\* / NOPRV** — class-file structure checks.
//! - **VTP\*** — property validation function checks.
//! - **MC\*** — class/OOP rules: naming, set/get signatures, constructor and
//!   superclass calls, Constant/WeakHandle properties.
//! - **FV\*** — `arguments` block validation (plus `TINVALDIM`,
//!   `TTOOFEWDIMS`).
//! - **Miscellaneous** — `FCONV`, `FCONF`, `GPFST`, `GPNES`, `NPERS`,
//!   `ROWLN`, `FCNANS`, `CLANS`, `BRKFOR`, `CONTFOR`, `IDXCOLND`, `ERTXT`,
//!   `WTXT`, `MHERIT`, `NCHKOS`, `CTOINE`, `CTORO`, `USESWNS`.
//!
//! With the opt-in path index (`[index] enabled = true`, or `--index`), MCEB is
//! also reported for a class whose superclasses are all value classes found on
//! the search path: MATLAB refuses to load such a class when it declares events.
//! The Code Analyzer itself stays silent for any superclass.
//!
//! ## Check IDs
//!
//! | Check ID | Severity | Fix | Description |
//! | --- | --- | --- | --- |
//! | PFPF | error | no | parfor loops cannot be used inside other parfor loops. |
//! | PFSPMD | error | no | spmd statements cannot be used inside parfor loops. |
//! | PFBRK | error | no | break statements cannot be used inside a parfor loop. |
//! | PFRTN | error | no | return statements cannot be used inside a parfor loop. |
//! | PFGLOB | error | no | Global variable declarations are not supported in parfor loops. |
//! | PFPERS | error | no | Persistent variable declarations are not supported in parfor loops. |
//! | PFFORA | error | no | Assigning to for loop variables is not supported in parfor loops. |
//! | PFXST | error | no | Assigning to the parfor loop index variable is not supported in parfor loops. |
//! | PFNF | error | no | Nested functions cannot be called from within parfor loops. |
//! | FPFORP | error | no | 'fprintf' is writing to a file that is opened with read permission only. Open a file using 'fopen(...,'W',...)' instead. |
//! | FWFORP | error | no | 'fwrite' is writing to a file that is opened with read permission only. Open a file using 'fopen(...,'W',...)' instead. |
//! | PFANON | error | no | Using a sliced output variable in an anonymous function is not supported in parfor loops. |
//! | PFANSLP | error | no | 'ans' is not supported as a parfor loop variable. |
//! | PFANSNS | error | no | 'ans' is not supported as a for loop variable in parfor loops. |
//! | PFCEL | error | no | The function VAR_NAME does not support cell arrays (argument VAR_NUMBER). |
//! | PFCTXT | error | no | When indexing a sliced variable with a nested for loop variable, the sliced variable must be inside the for loop that defines the range of the for loop variable. |
//! | PFEVC | error | no | EVALIN('caller') and ASSIGNIN('caller') are invalid inside of a PARFOR loop. |
//! | PFFRNG | error | no | When indexing a sliced variable with a nested for loop variable, the range of the for loop variable must be a row vector of positive constant numbers or variables. |
//! | PFFSUB | error | no | Indexing a nested for loop variable is not supported in parfor loops. |
//! | PFINCR | error | no | Using different reduction functions with the same reduction variable is not supported in parfor loops. |
//! | PFINPT | error | no | 'inputname' is not supported in parfor loops. |
//! | PFLD | error | no | 'load' must assign to an output variable in parfor loops. |
//! | PFMLTI | error | no | When indexing a sliced variable with a nested for loop variable, the for loop variable must not be assigned other than by its for statement. |
//! | PFNACK | error | no | 'narginchk' and 'nargoutchk' cannot be used in parfor loops. |
//! | PFNAIO | error | no | 'nargin' and 'nargout' require a function argument in parfor loops. |
//! | PFNAR | error | no | Subtracting reduction variable VAR_NAME from expressions is not supported in parfor loops. |
//! | PFRFH | error | no | The parfor reduction function VAR_NAME must either be a function name or a broadcast variable. |
//! | PFRNG | error | no | The range of a PARFOR statement must be consecutive integers. |
//! | PFSLO | error | no | Variable VAR_NAME is indexed using the parfor loop variable, but it is not a valid sliced output variable. |
//! | PFSLRD | error | no | Parfor loop variable VAR_NAME is accessed with an invalid combination of sliced indexing expressions and non-indexed reads. It is not valid to access the whole value of a sliced output variable. |
//! | PFSLW | error | no | Parfor loop variable VAR_NAME has multiple sliced accesses, but they do not all have the same list of subscripts. Each access to a sliced variable must use precisely the same list of subscripts. |
//! | PFSV | error | no | SAVE cannot be called in a PARFOR loop without the '-fromstruct' option. |
//! | PFUNK | error | no | Unable to classify variable VAR_NAME in the body of the parfor loop. |
//! | PFUTMP | error | no | Temporary variable VAR_NAME must be set inside the parfor loop before it is used. |
//! | PFUTVR | error | no | Variable VAR_NAME may have been intended as a reduction variable, but is an uninitialized temporary. |
//! | PFVARS | error | no | Parfor loop contains too many variables. |
//! | PFVSUB | error | no | Indexing parfor loop variables is not supported in parfor loops. |
//! | ATUNK | error | no | Unknown attribute name. |
//! | ATLAB | error | no | Attribute 'Input' and 'Output' must not be assigned a value or negated. |
//! | ATNAS | error | no | Set attribute to a single meta-class object or a cell array of meta-class objects. |
//! | ATNPI | error | no | Set attribute to 'public', 'private', 'protected', 'immutable', or a cell array of meta-classes instead. |
//! | ATNPP | error | no | Set attribute to 'public', 'private', 'protected', or a cell array of meta-classes instead. |
//! | ATPPI | error | no | The attribute value is unexpected. Use 'public', 'private', 'protected', 'immutable', or a cell array of meta-classes instead. |
//! | ATPPP | error | no | The attribute value is unexpected. Use 'public', 'private', 'protected', or a cell array of meta-classes instead. |
//! | ATAS | error | no | The attribute value is unexpected. Use a single meta-class object or a cell array of meta-class objects. |
//! | ATVIZE | error | no | The 'Visible' attribute is invalid for classes and events. Use the '~Hidden' attribute instead or omit the attribute since 'Hidden' is false by default. |
//! | CLSAT | error | no | Specify class attributes before the name of the class. |
//! | CLSUNK | error | no | This class, or one of its superclasses, could not be found on MATLAB's path. |
//! | NOPRV | error | no | A class definition cannot be inside a private directory. |
//! | VTPCON | error | no | For properties, validation functions must only use the property being validated or literals. |
//! | VTPEAL | error | no | Specify at least one input argument for validator. |
//! | VTPIN | error | no | Validation function must use the property as an input. |
//! | SPNST | error | no | PARFOR or SPMD cannot be used inside an SPMD block. |
//! | SPRET | error | no | VAR_RESERVED_WORD statement cannot be used inside an SPMD block. |
//! | SPBRK | error | no | The loop containing the BREAK or CONTINUE must be completely contained in the SPMD block. |
//! | DUPNAMEARG | warning | no | This named argument was already provided. If this function does not allow or ignores repeated named arguments, remove one of the duplicated named arguments. |
//! | SPGP | error | no | Setting the GLOBAL or PERSISTENT variable VAR_NAME in an SPMD block might fail because the set happens on a worker machine. |
//! | MCFIL | error | yes | Class name VAR_NAME and file name do not agree: VAR_FILE. Update the class name and constructor, if defined, or change the file name to match the class name. |
//! | MCDIR | error | no | Class name VAR_NAME and @directory name do not agree: VAR_FILE. |
//! | MCRED | error | no | Property, event, or enumeration names must be different from the name of the class VAR_NAME. |
//! | MCCBD | error | no | Constructor must be fully defined in the class definition file. |
//! | MCS2I | error | no | Set Methods must have exactly two inputs. |
//! | MCS1O | error | no | Set Methods must have at most one output. |
//! | MCG1I | error | no | Get methods must have exactly one input. |
//! | MCG1O | error | no | Get methods must have exactly one output. |
//! | MCEB | error | no | Events can be defined only in a handle class. |
//! | MCANI | error | no | Abstract property VAR_NAME cannot be initialized. |
//! | MCASC | error | no | Abstract property VAR_NAME cannot be used in a Sealed class. |
//! | MCSGA | error | no | Set or get method must be defined in a METHODS block with no attributes. |
//! | MCSGP | error | no | The method VAR_NAME does not refer to a valid property name. |
//! | MABSEAC | error | no | Instance properties and methods are illegal in classes that are both Sealed and Abstract. |
//! | MABSEAM | error | no | A method cannot be both Abstract and Sealed. |
//! | MCAPP | error | no | Private property cannot be Abstract. |
//! | MCCBS | error | no | A superclass constructor is being called, but VAR_NAME is not a declared superclass name. |
//! | MCCBU | error | no | This superclass constructor is called after a use of the constructed object. |
//! | MCCMC | error | no | Constructor for superclass can only be called once. |
//! | MCCSOP | error | no | Unable to modify Constant property VAR_NAME. |
//! | MCGSA | error | no | Method VAR_NAME tries to set or get an abstract property. |
//! | MCMIO | error | no | Method has too many inputs or outputs. |
//! | MCMSP | error | no | Private method cannot be Abstract. |
//! | MCMTP | error | no | TestParameterDefinition methods must be Static, so that they can be called at test suite creation time to set test parameter values. |
//! | MCPIN | error | no | Unable to initialize class property to an instance of the class itself. |
//! | MCPSG | error | no | Set or get method must be fully defined in the class definition file. |
//! | MCSCC | error | no | To call the superclass constructor, the name of the subclass constructor VAR_NAME must match the name of the subclass VAR_NAME. |
//! | MCSCF | error | no | A superclass constructor must be assigned to the first constructor output argument. |
//! | MCSCM | error | no | To call a superclass method, the method name VAR_NAME must match the name of the subclass method VAR_NAME. |
//! | MCSCN | error | no | Method VAR_NAME tries to set a constant property. |
//! | MCSCO | error | no | A superclass constructor must be called using the first constructor output argument. |
//! | MCSCT | error | no | Superclass constructor call must not be conditionalized or be part of another expression. |
//! | MCSMO | error | no | Returning multiple outputs from a superclass object initialization is not supported. |
//! | MCSWA | error | no | A sealed class cannot specify allowed subclasses. |
//! | MTAGS3 | error | no | Cannot use the Access attribute when using the SetAccess or GetAccess attribute. |
//! | MTMAT | error | no | Attribute can only be set once. |
//! | MWKCL | error | no | A WeakHandle property must restrict its type using a class validation. |
//! | MWKCT | error | no | Specifying both WeakHandle and Constant attributes on the same property is not supported. |
//! | MWKREF | error | no | Specifying both WeakHandle and Dependent attributes is invalid. A dependent property does not store a value. |
//! | FVNST | error | no | Arguments blocks in nested function declarations are not supported. |
//! | FVATF | error | no | Attribute values in arguments blocks must be logical constants. |
//! | FVBTN | error | no | Use of this function is not supported in arguments blocks. |
//! | FVDAN | error | no | Using the same name as both a name-value argument structure and as a positional argument is not supported. |
//! | FVDAP | error | no | Positional argument can only be declared once. |
//! | FVDNF | error | no | Name-value argument can only be declared once. |
//! | FVDREP | error | no | Multiple Repeating arguments blocks are not supported. |
//! | FVIDV | error | no | Specifying validation or default value for ignored arguments is not supported. |
//! | FVIOA | error | no | Specifying both 'Input' and 'Output' attributes on the same arguments block is not supported. |
//! | FVMCL | error | no | Specifying multiple name-value structures using .? syntax and a class name is not supported. |
//! | FVNDE | error | no | When specifying name-value arguments using a class name, it is illegal to specify default values for the arguments. |
//! | FVNIV | error | no | This variable is not an input to the function and cannot be used in an arguments block. |
//! | FVNREP | error | no | Name-value arguments are not supported in a Repeating arguments block. |
//! | FVNSC | error | no | Use of nested functions is not supported in arguments blocks. |
//! | FVNVL | error | no | When specifying name-value arguments using a class name, it is illegal to specify validation for the arguments. |
//! | FVOBI | error | no | Declare all input argument blocks before all output arguments blocks. |
//! | FVOCON | error | no | For output arguments, validation functions must only use the argument being validated or literals. |
//! | FVOND | error | no | Use of name-value arguments in default values is not supported. |
//! | FVONV | error | no | Use of name-value arguments without dotted name in the validation is not supported. |
//! | FVOOD | error | no | Specifying a default value for an output argument is not supported. |
//! | FVOOI | error | no | Use of ignored arguments in output arguments block is not supported. |
//! | FVOON | error | no | Using name-value argument as output argument is not supported. |
//! | FVORDI | error | no | Ignored input arguments are not allowed after a Repeating arguments block or name-value arguments. |
//! | FVORDN | error | no | Positional arguments must be defined before name-value arguments. |
//! | FVORDO | error | no | Repeating output arguments must be defined after required output arguments. |
//! | FVORDP | error | no | Positional arguments must be defined in the following order: required, optional, and repeating. |
//! | FVORM | error | no | Declaring multiple repeating output arguments is not supported. |
//! | FVOVREP | error | no | Output argument varargout can only be used inside a Repeating output arguments block. |
//! | FVREPD | error | no | Default values are not supported in a Repeating arguments block. |
//! | FVREPO | error | no | Repeating input arguments block containing varargin must not have other arguments. |
//! | FVSOR | error | no | Input arguments block declarations and the function line must contain the same input arguments in the same order, including ignored arguments. |
//! | FVSORO | error | no | Output arguments block declarations and the function line must contain the same output arguments in the same order. |
//! | FVUBD | error | no | Argument is referenced before it is declared in the arguments block. |
//! | FVVCON | error | no | For input arguments, validation functions must only use previously declared positional arguments, the argument being validated, or literals. |
//! | FVVIN | error | no | Validation function must use the argument as an input. |
//! | FVVREP | error | no | varargin can only be used inside repeating input arguments block. |
//! | TINVALDIM | error | no | Each dimension must be a nonnegative integer number or a colon. |
//! | TTOOFEWDIMS | error | no | Specify at least two dimensions for size. |
//! | FCONV | error | no | Unable to define variable VAR_NAME because it has the same name as the script. |
//! | FCONF | error | no | Unable to define local function VAR_NAME because it has the same name as the file. |
//! | GPFST | error | no | A GLOBAL or PERSISTENT declaration must precede first use. |
//! | PSET | warning | no | Persistent variable is used, but might be unset. |
//! | GPNES | error | no | A GLOBAL or PERSISTENT declaration must be in the outermost function where it is used. |
//! | NPERS | error | no | A PERSISTENT declaration is not valid in scripts. |
//! | ROWLN | error | no | All matrix rows must be the same length. |
//! | FCNANS | error | no | Using ANS as a function name is not supported. |
//! | CLANS | error | no | Using ANS as a class name is not supported. |
//! | BRKFOR | error | no | BREAK statement can only be used in a FOR or WHILE loop. |
//! | CONTFOR | error | no | CONTINUE statement can only be used in a FOR or WHILE loop. |
//! | IDXCOLND | error | no | The END operator must be used within an array index expression. |
//! | CTOINE | error | no | Use of constructed object as input to constructor is not supported. |
//! | CTORO | error | no | Class constructors must be declared with at least one output argument. |
//! | ERTXT | error | no | Specify an error message with the message identifier. |
//! | MHERIT | error | no | Deriving from the built-in MATLAB VAR_NAME class is not supported. |
//! | NCHKOS | error | no | NARGINCHK does not return any values. |
//! | SPBFN | error | no | Use of this function is invalid inside an SPMD block because it accesses or modifies the workspace in a non-transparent way. |
//! | SPDEC | error | no | The bounds on the number of workers an SPMD block can use must be a nonnegative integer. |
//! | SPDEC3 | error | no | An SPMD block can only specify a lower and upper bound for the number of workers to use. |
//! | SPEVC | error | no | EVALIN('caller') and ASSIGNIN('caller') are invalid inside of an SPMD block. |
//! | SPLD | error | no | To avoid a transparency violation, assign the output of LOAD to a variable in SPMD blocks. |
//! | SPNF | error | no | The nested function VAR_NAME cannot be called from within an SPMD block. |
//! | SPSV | error | no | SAVE cannot be called in an SPMD block without the '-fromstruct' option. |
//! | SPWHOS | error | no | Using "who" or "whos" without "-file" is invalid inside an SPMD block because it accesses the workspace in a non-transparent way. |
//! | USESWNS | error | no | Variable must be explicitly defined before first use. |
//! | WTXT | error | no | Specify a warning message with the message identifier. |
//!
//! ## Examples
//!
//! ### Incorrect
//!
//! ```matlab
//! parfor i = 1:10          % PFPF: nested parfor
//!     parfor j = 1:10
//!     end
//!     break                % PFBRK: break inside parfor
//! end
//!
//! function y = f(a)
//!     arguments (Input, Output)   % FVIOA
//!         a (1,1) double = 1      % FVOOD: default on output
//!     end
//! end
//! ```
//!
//! ### Correct
//!
//! ```matlab
//! parfor i = 1:10
//!     x(i) = i;
//! end
//!
//! function y = f(a)
//!     arguments (Input)
//!         a (1,1) double
//!     end
//!     arguments (Output)
//!         y (1,1) double
//!     end
//!     y = a;
//! end
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [lint.rules.LANGUAGE_SPEC_ENGINE]
//! severity = "error"
//! disabled_checks = ["PFPF", "FVIOA"]
//! ```
use std::collections::{HashMap, HashSet};
use std::ops::Range;

use malt_core::{Category, Config, Diagnostic, FileContext, Rule, Severity};
use serde::Deserialize;

use crate::analysis::metadata::{
    attribute_entries, attributes_of, ArgValidation, AttrForms, AttributeMeta, Attributes,
    ClassMeta, FileMeta, FileType, FunctionMeta,
};
use crate::analysis::symbols::{DefKind, SymbolTable};

// ---------------------------------------------------------------------------
// Rule-specific configuration
// ---------------------------------------------------------------------------

/// Configuration for the language specification engine.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LanguageSpecConfig {
    /// Check IDs to skip (e.g., `["MABSEAC", "MTAGS3"]`).
    #[serde(default)]
    pub disabled_checks: Vec<String>,
}

// ---------------------------------------------------------------------------
// Context tracking for DFS
// ---------------------------------------------------------------------------

/// Tracks what kind of loop/parallel context we are in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoopContext {
    /// Normal code (not inside any special parallel context).
    Normal,
    /// Inside a regular for/while loop.
    Loop,
    /// Inside a parfor loop.
    Parfor,
    /// Inside an spmd block.
    Spmd,
}

/// A frame on the context stack during DFS traversal.
#[derive(Debug, Clone)]
pub(crate) struct ContextFrame {
    /// The type of context this frame represents.
    context: LoopContext,
    /// The parfor loop variable name (only set for Parfor context).
    parfor_var: Option<String>,
    /// The for-loop iterator variable names active in this parfor (accumulated).
    for_vars_in_parfor: Vec<String>,
    /// Per-parfor analysis state for the parfor variable/slicing/reduction checks.
    /// Only populated for `LoopContext::Parfor` frames.
    parfor_analysis: ParforAnalysis,
}

/// Tracks a nested `for` loop inside a parfor body.
#[derive(Debug, Clone)]
struct NestedForInfo {
    /// The nested loop variable name.
    var: String,
    /// Byte range of the nested `for_statement` node.
    for_start: usize,
    /// End byte of the nested `for_statement` node.
    for_end: usize,
    /// Byte range of the nested loop's range expression.
    range_start: usize,
    /// End byte of the nested loop's range expression.
    range_end: usize,
    /// Whether the range is a row of positive constant numbers.
    range_is_const_pos: bool,
}

/// Per-parfor analysis state collected during the DFS traversal.
///
/// Aggregates variable classification (sliced outputs, temporaries, reductions),
/// indexed accesses, and file-handle tracking so that body-level parfor checks
/// can be finalized when the parfor frame is popped.
#[derive(Debug, Clone, Default)]
pub(crate) struct ParforAnalysis {
    /// The parfor loop variable name.
    parfor_var: String,
    /// Byte range of the parfor `for_statement` node.
    parfor_start: usize,
    /// End byte of the parfor `for_statement` node.
    parfor_end: usize,
    /// Nested `for` loops inside the parfor body.
    nested_fors: Vec<NestedForInfo>,
    /// Sliced output assignments `v(i,...) = ...`:
    /// var -> (arguments text, start, end, is_cell_indexing).
    sliced_lhs: HashMap<String, Vec<(String, usize, usize, bool)>>,
    /// Indexed reads `v(...)` used as a value:
    /// var -> (arguments text, start, end, is_cell_indexing).
    indexed_reads: HashMap<String, Vec<(String, usize, usize, bool)>>,
    /// Plain (temporary) assignments: var -> assignment start bytes.
    temp_assigns: HashMap<String, Vec<usize>>,
    /// Whole (non-indexed) reads of a variable: var -> read start bytes.
    whole_reads: HashMap<String, Vec<usize>>,
    /// Proper reductions `v = v op expr`: var -> (operator, assignment start).
    reductions: HashMap<String, Vec<(String, usize)>>,
    /// Functional reductions `v = f(v, ...)`: var -> (callee, assignment start).
    func_reductions: HashMap<String, Vec<(String, usize)>>,
    /// `expr - v` patterns in assignment RHSs: (var, assignment start).
    sub_right: Vec<(String, usize)>,
    /// Variables that appear on both sides of an assignment in the body.
    both_sides: HashSet<String>,
    /// Anonymous functions in the body: (start, referenced variable names).
    lambdas: Vec<(usize, Vec<String>, String)>,
    /// File handles opened read-only via `fid = fopen(...)`.
    read_only_handles: HashSet<String>,
    /// All variable names observed in the parfor body.
    var_names: HashSet<String>,
}

// ---------------------------------------------------------------------------
// Rule implementation
// ---------------------------------------------------------------------------

/// Language specification error engine.
///
/// Implements all language specification constraint checks as a single file-level
/// rule. Uses DFS traversal with a context stack to detect parfor/spmd/loop
/// violations, plus class metadata analysis for OOP rule checking.
pub struct LanguageSpecEngine {
    #[allow(dead_code)]
    config: LanguageSpecConfig,
}

impl LanguageSpecEngine {
    /// Factory constructor. Reads rule-specific params from config.
    pub fn from_config(config: &Config) -> Box<dyn Rule> {
        let mut rule_config: LanguageSpecConfig = config.rule_params("LANGUAGE_SPEC_ENGINE");
        rule_config
            .disabled_checks
            .extend(crate::off_under_mathworks(
                config,
                &["USESWNS", "PFUTVR", "GPNES", "FVORDO"],
            ));
        Box::new(Self {
            config: rule_config,
        })
    }
}

impl Rule for LanguageSpecEngine {
    fn id(&self) -> &'static str {
        "LANGUAGE_SPEC_ENGINE"
    }

    fn description(&self) -> &'static str {
        "Language specification constraint violation"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn category(&self) -> Category {
        Category::LanguageSpecification
    }

    fn target_node_types(&self) -> &'static [&'static str] {
        &[] // File-level only
    }

    fn has_file_check(&self) -> bool {
        true
    }

    fn check_file(&self, ctx: &FileContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Build metadata and symbol table
        let meta = FileMeta::build(ctx.tree, ctx.source);
        let symbol_table = SymbolTable::shared(|| ctx.tree.root_node(), ctx.source);

        // 1. DFS traversal for parfor/spmd/loop context violations
        let root = ctx.tree.root_node();
        let nested_functions = self.collect_nested_function_names(root, ctx.source);
        let mut context_stack = vec![ContextFrame {
            context: LoopContext::Normal,
            parfor_var: None,
            for_vars_in_parfor: Vec::new(),
            parfor_analysis: ParforAnalysis::default(),
        }];
        self.check_dfs(
            root,
            ctx.source,
            &mut context_stack,
            &nested_functions,
            &mut diagnostics,
        );

        // 1b. SPGP: assignment to a global/persistent variable inside spmd
        self.check_spmd_gp_assignment(root, ctx.source, &symbol_table, &mut diagnostics);

        // 2. Class/OOP checks
        if let Some(ref class) = meta.class {
            self.check_class_rules(class, &meta, ctx, &mut diagnostics);
        }

        // 3. Class-file level checks (CLSAT, CLSUNK, NOPRV)
        self.check_class_file_rules(&meta, ctx, &mut diagnostics);

        // 4. ATLAB: Input/Output arguments-block attributes with values/negation
        self.check_atlab(ctx.tree.root_node(), ctx.source, &mut diagnostics);

        // 5. Function validation checks
        self.check_function_validation(&meta, ctx, &mut diagnostics);

        // 6. Script-level checks
        self.check_script_rules(&meta, &symbol_table, ctx, &mut diagnostics);

        // 7. Global/persistent ordering checks
        self.check_global_persistent_rules(&symbol_table, ctx, &mut diagnostics);

        // 8. break/continue outside loop (top-level check)
        self.check_break_continue_outside_loop(root, ctx.source, &mut diagnostics);

        // 9. Matrix row length checks
        self.check_matrix_rows(root, ctx.source, &mut diagnostics);

        // 10. Function/class named 'ans'
        self.check_ans_naming(&meta, ctx, &mut diagnostics);

        // 11. END operator outside index expression
        self.check_end_operator(root, ctx.source, &mut diagnostics);

        // 11b. Repeated name=value argument in one call (DUPNAMEARG)
        self.check_dupnamearg(root, ctx.source, &mut diagnostics);

        // 12. Local function name same as file name
        self.check_local_function_name_conflict(&meta, ctx, &mut diagnostics);

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// DFS traversal for parfor/spmd/loop context checks
// ---------------------------------------------------------------------------

impl LanguageSpecEngine {
    /// DFS traversal to detect parfor/spmd context violations.
    fn check_dfs(
        &self,
        node: tree_sitter::Node,
        source: &str,
        context_stack: &mut Vec<ContextFrame>,
        nested_functions: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let kind = node.kind();
        let current_context = context_stack
            .last()
            .map(|f| f.context)
            .unwrap_or(LoopContext::Normal);

        match kind {
            "for_statement" => {
                let is_parfor = self.is_parfor_node(node);

                if is_parfor {
                    // This is a parfor loop

                    // Check: PFPF — nested parfor inside parfor
                    if current_context == LoopContext::Parfor {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "PFPF",
                            message: "parfor loops cannot be used inside other parfor loops."
                                .to_string(),
                            severity: Severity::Error,
                            byte_range: node.start_byte()..node.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }

                    // Check: SPNST — parfor inside spmd
                    if current_context == LoopContext::Spmd {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "SPNST",
                            message: "PARFOR or SPMD cannot be used inside an SPMD block."
                                .to_string(),
                            severity: Severity::Error,
                            byte_range: node.start_byte()..node.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }

                    // Extract the parfor loop variable
                    let parfor_var = self.extract_for_variable(node, source);

                    // C1 checks on the parfor header (loop variable name and range)
                    self.check_parfor_header(node, source, diagnostics);

                    // Push parfor context and recurse
                    context_stack.push(ContextFrame {
                        context: LoopContext::Parfor,
                        parfor_var,
                        for_vars_in_parfor: Vec::new(),
                        parfor_analysis: ParforAnalysis {
                            parfor_var: self.extract_for_variable(node, source).unwrap_or_default(),
                            parfor_start: node.start_byte(),
                            parfor_end: node.end_byte(),
                            ..Default::default()
                        },
                    });
                    self.visit_children(node, source, context_stack, nested_functions, diagnostics);
                    if let Some(frame) = context_stack.last() {
                        self.finalize_parfor_analysis(
                            &frame.parfor_analysis,
                            node,
                            source,
                            diagnostics,
                        );
                    }
                    context_stack.pop();
                } else {
                    // Normal for loop
                    if current_context == LoopContext::Parfor {
                        // Track the for-loop variable inside parfor
                        if let Some(var) = self.extract_for_variable(node, source) {
                            if let Some(frame) = context_stack.last_mut() {
                                frame.for_vars_in_parfor.push(var);
                            }
                        }
                        // C1 checks for nested for loops inside parfor
                        self.check_nested_for_in_parfor(node, source, context_stack, diagnostics);
                        // Stay in Parfor context
                        self.visit_children(
                            node,
                            source,
                            context_stack,
                            nested_functions,
                            diagnostics,
                        );
                        // Remove the for variable from tracking
                        if let Some(frame) = context_stack.last_mut() {
                            frame.for_vars_in_parfor.pop();
                        }
                    } else {
                        // Push normal loop context
                        context_stack.push(ContextFrame {
                            context: LoopContext::Loop,
                            parfor_var: None,
                            for_vars_in_parfor: Vec::new(),
                            parfor_analysis: ParforAnalysis::default(),
                        });
                        self.visit_children(
                            node,
                            source,
                            context_stack,
                            nested_functions,
                            diagnostics,
                        );
                        context_stack.pop();
                    }
                }
                return;
            }

            "while_statement" => {
                if current_context == LoopContext::Parfor {
                    // Stay in Parfor context for while inside parfor
                    self.visit_children(node, source, context_stack, nested_functions, diagnostics);
                } else {
                    // Push normal loop context
                    context_stack.push(ContextFrame {
                        context: LoopContext::Loop,
                        parfor_var: None,
                        for_vars_in_parfor: Vec::new(),
                        parfor_analysis: ParforAnalysis::default(),
                    });
                    self.visit_children(node, source, context_stack, nested_functions, diagnostics);
                    context_stack.pop();
                }
                return;
            }

            "spmd_statement" => {
                // Check: PFSPMD — spmd inside parfor
                if current_context == LoopContext::Parfor {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "PFSPMD",
                        message: "spmd statements cannot be used inside parfor loops.".to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }

                // Check: SPNST — spmd inside spmd
                if current_context == LoopContext::Spmd {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "SPNST",
                        message: "PARFOR or SPMD cannot be used inside an SPMD block.".to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }

                // Check: SPDEC / SPDEC3 — spmd worker bounds
                self.check_spmd_worker_bounds(node, source, diagnostics);

                // Push spmd context and recurse
                context_stack.push(ContextFrame {
                    context: LoopContext::Spmd,
                    parfor_var: None,
                    for_vars_in_parfor: Vec::new(),
                    parfor_analysis: ParforAnalysis::default(),
                });
                self.visit_children(node, source, context_stack, nested_functions, diagnostics);
                context_stack.pop();
                return;
            }

            "break_statement" => {
                // Check: PFBRK — the first break of the parfor loop itself
                if current_context == LoopContext::Parfor && self.is_first_parfor_break(node) {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "PFBRK",
                        message: "break statements cannot be used inside a parfor loop."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
                // Check: SPBRK — break inside spmd whose loop is outside the block
                if current_context == LoopContext::Spmd {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "SPBRK",
                        message: "The loop containing the BREAK or CONTINUE must be completely contained in the SPMD block.".to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }

            "continue_statement" if current_context == LoopContext::Spmd => {
                // Check: SPBRK — continue inside spmd whose loop is outside the block
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "SPBRK",
                    message: "The loop containing the BREAK or CONTINUE must be completely contained in the SPMD block.".to_string(),
                    severity: Severity::Error,
                    byte_range: node.start_byte()..node.end_byte(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }

            "return_statement" => {
                // Check: PFRTN — return inside parfor
                if current_context == LoopContext::Parfor {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "PFRTN",
                        message: "return statements cannot be used inside a parfor loop."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
                // Check: SPRET — return inside spmd
                if current_context == LoopContext::Spmd {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "SPRET",
                        message: "return statement cannot be used inside an SPMD block."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }

            "global_operator" => {
                // Check: PFGLOB — global inside parfor
                if current_context == LoopContext::Parfor {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "PFGLOB",
                        message: "Global variable declarations are not supported in parfor loops."
                            .to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }

            "persistent_operator" => {
                // Check: PFPERS — persistent inside parfor
                if current_context == LoopContext::Parfor {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "PFPERS",
                        message:
                            "Persistent variable declarations are not supported in parfor loops."
                                .to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }

            "assignment" if current_context == LoopContext::Parfor => {
                // PFXST for indexed (`i(1) = 2`) and multi-output (`[~, i] = f()`) targets
                // of the loop variable, reported at the variable (probed).
                let loop_var = context_stack
                    .iter()
                    .rev()
                    .find(|f| f.context == LoopContext::Parfor)
                    .and_then(|f| f.parfor_var.as_deref());
                if let (Some(pv), Some(lhs)) = (loop_var, node.child_by_field_name("left")) {
                    if lhs.kind() != "identifier" && lhs.kind() != "field_expression" {
                        let targets: Vec<tree_sitter::Node> = if lhs.kind() == "function_call" {
                            lhs.named_child(0).into_iter().collect()
                        } else {
                            let mut cursor = lhs.walk();
                            lhs.named_children(&mut cursor).collect()
                        };
                        for target in targets {
                            if target.kind() == "identifier" && node_text(target, source) == pv {
                                let pos = target.start_position();
                                diagnostics.push(Diagnostic {
                                    rule_id: "PFXST",
                                    message: "Assigning to the parfor loop index variable is not supported in parfor loops.".to_string(),
                                    severity: Severity::Error,
                                    byte_range: target.start_byte()..target.end_byte(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                            }
                        }
                    }
                }

                // Check: PFXST — assignment to parfor loop variable
                // Check: PFFORA — assignment to for-loop variable inside parfor
                if let Some(lhs_name) = self.extract_assignment_lhs_name(node, source) {
                    // PFXST: assignment to the parfor loop variable itself
                    let parfor_var = context_stack
                        .iter()
                        .rev()
                        .find(|f| f.context == LoopContext::Parfor)
                        .and_then(|f| f.parfor_var.as_deref());
                    if let Some(pv) = parfor_var {
                        if lhs_name == pv {
                            let pos = node.start_position();
                            diagnostics.push(Diagnostic {
                                rule_id: "PFXST",
                                message: "Assigning to the parfor loop index variable is not supported in parfor loops.".to_string(),
                                severity: Severity::Error,
                                byte_range: node.start_byte()..node.end_byte(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    }

                    // PFFORA: assignment to a for-loop variable inside the parfor
                    let for_vars: Vec<String> = context_stack
                        .iter()
                        .rev()
                        .find(|f| f.context == LoopContext::Parfor)
                        .map(|f| f.for_vars_in_parfor.clone())
                        .unwrap_or_default();
                    if for_vars.contains(&lhs_name) {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            rule_id: "PFFORA",
                            message:
                                "Assigning to for loop variables is not supported in parfor loops."
                                    .to_string(),
                            severity: Severity::Error,
                            byte_range: node.start_byte()..node.end_byte(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }

                // C1 checks: variable classification, reductions, nested loop variables
                self.check_parfor_assignment(node, source, context_stack, diagnostics);
            }

            "function_definition" => {
                // Nested function definitions cannot appear inside a parfor/spmd
                // body; do not recurse into them from a parallel context.
                if current_context == LoopContext::Parfor || current_context == LoopContext::Spmd {
                    return;
                }
                // For Normal/Loop context, recurse normally into function body
                // (but reset context to Normal since function creates its own scope)
                context_stack.push(ContextFrame {
                    context: LoopContext::Normal,
                    parfor_var: None,
                    for_vars_in_parfor: Vec::new(),
                    parfor_analysis: ParforAnalysis::default(),
                });
                self.visit_children(node, source, context_stack, nested_functions, diagnostics);
                context_stack.pop();
                return;
            }

            "function_call" => {
                // SPMD transparency checks (only inside spmd blocks)
                if current_context == LoopContext::Spmd {
                    self.check_spmd_function_call(node, source, nested_functions, diagnostics);
                }
                // C1 checks (only inside parfor blocks)
                if current_context == LoopContext::Parfor {
                    self.check_parfor_function_call(
                        node,
                        source,
                        context_stack,
                        nested_functions,
                        diagnostics,
                    );
                }
                // Message-specification checks (any context)
                self.check_error_warning_message(node, source, diagnostics);
                // NCHKOS: narginchk/nargoutchk used as a value
                self.check_nchkos_output_use(node, source, diagnostics);
                // Recurse into arguments (nested calls may contain checks)
                self.visit_children(node, source, context_stack, nested_functions, diagnostics);
                return;
            }

            "command" => {
                // Command-form calls inside spmd blocks
                if current_context == LoopContext::Spmd {
                    self.check_spmd_command(node, source, diagnostics);
                }
                // C1 checks (only inside parfor blocks)
                if current_context == LoopContext::Parfor {
                    self.check_parfor_command(node, source, diagnostics);
                }
                // ERTXT / WTXT: command form `error MyTool:id`
                self.check_error_warning_command(node, source, diagnostics);
                self.visit_children(node, source, context_stack, nested_functions, diagnostics);
                return;
            }

            "identifier" if current_context == LoopContext::Parfor => {
                // C1 checks: record whole (non-indexed) variable reads
                self.check_parfor_identifier(node, source, context_stack);
                // PFNAIO: bare `nargin`/`nargout` used as a value
                self.check_parfor_bare_nargin(node, source, diagnostics);
                // NCHKOS: bare `narginchk` used as a value
                self.check_nchkos_bare(node, source, diagnostics);
            }

            "identifier" if current_context == LoopContext::Spmd => {
                // SPWHOS: bare `who`/`whos` used as a value
                self.check_spmd_bare_who(node, source, diagnostics);
                // NCHKOS: bare `narginchk` used as a value
                self.check_nchkos_bare(node, source, diagnostics);
            }

            "identifier" => {
                // NCHKOS: bare `narginchk` used as a value
                self.check_nchkos_bare(node, source, diagnostics);
            }

            "lambda" if current_context == LoopContext::Parfor => {
                // C1 checks: record anonymous-function references (PFANON)
                self.check_parfor_lambda(node, source, context_stack);
            }

            _ => {}
        }

        // Recurse into children
        self.visit_children(node, source, context_stack, nested_functions, diagnostics);
    }

    /// Visit all children of a node during DFS traversal.
    fn visit_children(
        &self,
        node: tree_sitter::Node,
        source: &str,
        context_stack: &mut Vec<ContextFrame>,
        nested_functions: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_dfs(child, source, context_stack, nested_functions, diagnostics);
        }
    }

    /// Check if a `for_statement` node is actually a `parfor` loop.
    ///
    /// In tree-sitter-matlab, `parfor i = 1:10 ... end` is parsed as a
    /// `for_statement` with a child node of kind `"parfor"` (instead of `"for"`).
    fn is_parfor_node(&self, node: tree_sitter::Node) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "parfor" {
                return true;
            }
            // Stop early once we hit the iterator (keyword comes before it)
            if child.kind() == "iterator" {
                break;
            }
        }
        false
    }

    /// The parfor loop that owns `brk`: its nearest enclosing loop, provided no
    /// `while` or `switch` lies between them.
    fn owning_parfor<'a>(&self, brk: tree_sitter::Node<'a>) -> Option<tree_sitter::Node<'a>> {
        let mut parent = brk.parent();
        while let Some(p) = parent {
            match p.kind() {
                "while_statement" | "switch_statement" => return None,
                "for_statement" => return self.is_parfor_node(p).then_some(p),
                _ => {}
            }
            parent = p.parent();
        }
        None
    }

    /// Whether `brk` is the first break owned by its parfor loop. Probed: MATLAB
    /// reports PFBRK once per loop and is silent for breaks inside a nested `for`,
    /// `while` or `switch`.
    fn is_first_parfor_break(&self, brk: tree_sitter::Node) -> bool {
        let Some(parfor) = self.owning_parfor(brk) else {
            return false;
        };
        let mut first: Option<usize> = None;
        let mut stack = vec![parfor];
        while let Some(n) = stack.pop() {
            if n.kind() == "break_statement"
                && self.owning_parfor(n).is_some_and(|p| p.id() == parfor.id())
            {
                first = Some(first.map_or(n.start_byte(), |f| f.min(n.start_byte())));
            }
            let mut cursor = n.walk();
            stack.extend(n.children(&mut cursor));
        }
        first == Some(brk.start_byte())
    }

    /// Extract the loop variable from a `for_statement` (works for both for and parfor).
    pub(crate) fn extract_for_variable(
        &self,
        node: tree_sitter::Node,
        source: &str,
    ) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "iterator" {
                return self.extract_iterator_variable(child, source);
            }
        }
        None
    }

    /// Extract the variable name from an iterator node.
    fn extract_iterator_variable(
        &self,
        iter_node: tree_sitter::Node,
        source: &str,
    ) -> Option<String> {
        let mut cursor = iter_node.walk();
        for child in iter_node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Some(node_text(child, source).to_string());
            }
        }
        None
    }

    /// Extract the LHS variable name from a simple assignment.
    fn extract_assignment_lhs_name(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        let lhs = node.child_by_field_name("left")?;
        if lhs.kind() == "identifier" {
            Some(node_text(lhs, source).to_string())
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// SPGP: setting a global/persistent variable inside an spmd block
// ---------------------------------------------------------------------------

impl LanguageSpecEngine {
    /// SPGP: flag an assignment to a global or persistent variable inside an
    /// `spmd` block, substituting the variable name for VAR_NAME.
    ///
    /// A `global`/`persistent` declaration alone does not fire; only an
    /// assignment to such a variable inside the spmd body does.
    ///
    /// # Limitations
    ///
    /// Global/persistent resolution consults the enclosing scope chain in the
    /// symbol table, so an assignment to a name that is declared
    /// `global`/`persistent` in an OUTER scope is flagged, but a name that is
    /// only declared in a nested scope is not attributed to the spmd body.
    pub(crate) fn check_spmd_gp_assignment(
        &self,
        root: tree_sitter::Node,
        source: &str,
        symbol_table: &SymbolTable,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.spmd_gp_assignment_dfs(root, source, symbol_table, false, diagnostics);
    }

    /// DFS helper for [`Self::check_spmd_gp_assignment`].
    fn spmd_gp_assignment_dfs(
        &self,
        node: tree_sitter::Node,
        source: &str,
        symbol_table: &SymbolTable,
        in_spmd: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "spmd_statement" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.spmd_gp_assignment_dfs(child, source, symbol_table, true, diagnostics);
                }
                return;
            }
            "assignment" if in_spmd => {
                // Probed: field and indexed stores (`G.a = 1`, `G(2) = 1`) and each
                // identifier of a target list (`[G, h] = ...`) report at the variable.
                for (name, target) in self.assignment_lhs_targets(node, source) {
                    if self.is_global_or_persistent_in_scope(symbol_table, node.start_byte(), &name)
                    {
                        self.push_diag(
                            target,
                            "SPGP",
                            &format!(
                                "Setting the GLOBAL or PERSISTENT variable '{name}' in an SPMD block might fail because the set happens on a worker machine."
                            ),
                            diagnostics,
                        );
                    }
                }
            }
            _ => {}
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.spmd_gp_assignment_dfs(child, source, symbol_table, in_spmd, diagnostics);
        }
    }

    /// Whether `name` is declared `global` or `persistent` in the scope (or an
    /// ancestor scope) containing the given byte offset.
    fn is_global_or_persistent_in_scope(
        &self,
        symbol_table: &SymbolTable,
        byte: usize,
        name: &str,
    ) -> bool {
        let Some(idx) = symbol_table
            .scopes
            .iter()
            .enumerate()
            .filter(|(_, s)| s.byte_range.contains(&byte))
            .min_by_key(|(_, s)| s.byte_range.end - s.byte_range.start)
            .map(|(i, _)| i)
        else {
            return false;
        };
        let mut cur = Some(idx);
        while let Some(i) = cur {
            let scope = &symbol_table.scopes[i];
            if scope
                .defs
                .iter()
                .any(|d| d.name == name && matches!(d.kind, DefKind::Global | DefKind::Persistent))
            {
                return true;
            }
            cur = scope.parent;
        }
        false
    }

    /// Variables an assignment stores into, each with the node to report at: the
    /// base of `v`, `v(...)`, `v.f` or `v.f(...)`, or each identifier of a target
    /// list such as `[a, b] = ...`.
    fn assignment_lhs_targets<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
    ) -> Vec<(String, tree_sitter::Node<'a>)> {
        let Some(lhs) = node.child_by_field_name("left") else {
            return Vec::new();
        };
        let mut base = lhs;
        while matches!(base.kind(), "function_call" | "field_expression") {
            match base.named_child(0) {
                Some(child) => base = child,
                None => break,
            }
        }
        if base.kind() == "identifier" {
            return vec![(node_text(base, source).to_string(), base)];
        }
        let mut out = Vec::new();
        let mut cursor = lhs.walk();
        for child in lhs.named_children(&mut cursor) {
            if child.kind() == "identifier" {
                out.push((node_text(child, source).to_string(), child));
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Parfor variable / slicing / reduction checks (C1 group)
// ---------------------------------------------------------------------------

/// Functions that do not accept cell array inputs (PFCEL).
const PFCEL_NO_CELL_FUNCTIONS: &[&str] = &[
    "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "exp", "log", "log10",
    "log2", "sqrt", "abs", "sign", "floor", "ceil", "round", "fix", "mod", "rem",
];

/// Reduction functions supported by parfor functional reductions (PFRFH).
const PFRFH_REDUCTION_FUNCTIONS: &[&str] = &[
    "plus",
    "minus",
    "times",
    "mtimes",
    "max",
    "min",
    "or",
    "and",
    "xor",
    "horzcat",
    "vertcat",
    "hypot",
    "union",
    "intersect",
    "setxor",
    "setdiff",
    "unique",
    "strcat",
];

/// Maximum number of variables a parfor loop may contain (PFVARS).
const PARFOR_VAR_LIMIT: usize = 1024;

impl LanguageSpecEngine {
    /// Handle a nested `for` loop inside a parfor body (PFANSNS) and record it
    /// for the PFCTXT/PFFRNG body-level checks.
    fn check_nested_for_in_parfor(
        &self,
        node: tree_sitter::Node,
        source: &str,
        context_stack: &mut [ContextFrame],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(var) = self.extract_for_variable(node, source) else {
            return;
        };

        // PFANSNS: 'ans' as a for loop variable inside a parfor
        if var == "ans" {
            self.push_diag(
                node,
                "PFANSNS",
                "'ans' is not supported as a for loop variable in parfor loops.",
                diagnostics,
            );
        }

        // A nested `for` over the parfor index assigns it (probed: PFXST at the `for`
        // line, and no PFCTXT or PFFRNG for its indexing).
        let parfor_var = context_stack
            .iter()
            .rev()
            .find(|f| f.context == LoopContext::Parfor)
            .and_then(|f| f.parfor_var.as_deref());
        if parfor_var == Some(var.as_str()) {
            let pos = node.start_position();
            diagnostics.push(Diagnostic {
                rule_id: "PFXST",
                message:
                    "Assigning to the parfor loop index variable is not supported in parfor loops."
                        .to_string(),
                severity: Severity::Error,
                byte_range: node.start_byte()..node.end_byte(),
                line: pos.row + 1,
                column: pos.column + 1,
                fix: None,
            });
            return;
        }

        let (range_start, range_end, is_const_pos) = self.nested_for_range_info(node, source);
        if let Some(analysis) = current_parfor_analysis_mut(context_stack) {
            analysis.nested_fors.push(NestedForInfo {
                var: var.clone(),
                for_start: node.start_byte(),
                for_end: node.end_byte(),
                range_start,
                range_end,
                range_is_const_pos: is_const_pos,
            });
            analysis.var_names.insert(var);
        }
    }

    /// Record an assignment inside a parfor body: variable classification
    /// (temporary / sliced / reduction), PFMLTI, PFNAR candidates, and
    /// read-only file-handle tracking for FPFORP/FWFORP.
    fn check_parfor_assignment(
        &self,
        node: tree_sitter::Node,
        source: &str,
        context_stack: &mut [ContextFrame],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(lhs_node) = node.child_by_field_name("left") else {
            return;
        };
        if lhs_node.kind() != "identifier" {
            // Indexed LHS (`x(i) = ...`) is recorded by the function_call arm.
            return;
        }
        let lhs_name = node_text(lhs_node, source).to_string();
        let rhs = node.child_by_field_name("right");

        let Some(analysis) = current_parfor_analysis_mut(context_stack) else {
            return;
        };
        let nested_for_vars: Vec<String> = analysis
            .nested_fors
            .iter()
            .map(|nf| nf.var.clone())
            .collect();

        // PFMLTI: the nested for loop variable must not be assigned in the body
        if nested_for_vars.contains(&lhs_name) {
            self.push_diag(
                node,
                "PFMLTI",
                "When indexing a sliced variable with a nested for loop variable, the for loop variable must not be assigned other than by its for statement.",
                diagnostics,
            );
        }

        analysis.var_names.insert(lhs_name.clone());

        // Classify the assignment: proper reduction, functional reduction, or temp.
        let mut reduction_op: Option<String> = None;
        let mut func_reduction_callee: Option<String> = None;
        if let Some(rhs_node) = rhs {
            match rhs_node.kind() {
                "binary_operator" => {
                    let left = rhs_node.child(0);
                    if let Some(l) = left {
                        if l.kind() == "identifier" && node_text(l, source) == lhs_name {
                            reduction_op = Some(operator_text(rhs_node, source));
                        }
                    }
                }
                "matrix" => {
                    // `s = [s i]`: a reduction by concatenation (probed: PFINCR next to `s = s + i`).
                    let first = rhs_node.named_child(0).map(|r| {
                        if r.kind() == "row" {
                            r.named_child(0).unwrap_or(r)
                        } else {
                            r
                        }
                    });
                    if first.is_some_and(|a| {
                        a.kind() == "identifier" && node_text(a, source) == lhs_name
                    }) {
                        reduction_op = Some("[]".to_string());
                    }
                }
                "function_call" => {
                    let first = argument_nodes(rhs_node).first().copied();
                    if let Some(a) = first {
                        if a.kind() == "identifier" && node_text(a, source) == lhs_name {
                            if let Some(callee) = callee_name(rhs_node, source) {
                                func_reduction_callee = Some(callee);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        match (reduction_op, func_reduction_callee) {
            (Some(op), _) => {
                analysis
                    .reductions
                    .entry(lhs_name.clone())
                    .or_default()
                    .push((op, node.start_byte()));
            }
            (None, Some(callee)) => {
                analysis
                    .func_reductions
                    .entry(lhs_name.clone())
                    .or_default()
                    .push((callee, node.start_byte()));
            }
            (None, None) => {
                analysis
                    .temp_assigns
                    .entry(lhs_name.clone())
                    .or_default()
                    .push(node.start_byte());
            }
        }

        if let Some(rhs_node) = rhs {
            // Track variables that appear on both sides of an assignment.
            if contains_identifier(rhs_node, source, &lhs_name) {
                analysis.both_sides.insert(lhs_name.clone());
            }

            // PFNAR candidate: `expr - reduction_var`
            if rhs_node.kind() == "binary_operator" {
                let op = operator_text(rhs_node, source);
                if op == "-" {
                    let right = rhs_node.child(2);
                    if let Some(r) = right {
                        if r.kind() == "identifier" && node_text(r, source) == lhs_name {
                            analysis
                                .sub_right
                                .push((lhs_name.clone(), node.start_byte()));
                        }
                    }
                }
            }

            // FPFORP/FWFORP: track file handles opened read-only.
            if rhs_node.kind() == "function_call"
                && callee_name(rhs_node, source).as_deref() == Some("fopen")
                && fopen_is_read_only(rhs_node, source)
            {
                analysis.read_only_handles.insert(lhs_name.clone());
            }
        }
    }

    /// Handle a function call inside a parfor body: banned functions (PFEVC,
    /// PFINPT, PFNACK, PFNAIO, PFLD, PFSV, FPFORP, FWFORP, PFCEL), nested function
    /// calls (PFNF), loop-variable indexing (PFVSUB, PFFSUB), and indexed-access
    /// recording for the sliced-variable checks.
    fn check_parfor_function_call(
        &self,
        node: tree_sitter::Node,
        source: &str,
        context_stack: &mut [ContextFrame],
        nested_functions: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(name) = callee_name(node, source) else {
            return;
        };

        // PFNF: calling a nested function from within a parfor loop body.
        if nested_functions.contains(&name) {
            self.push_diag(
                node,
                "PFNF",
                "Nested functions cannot be called from within parfor loops.",
                diagnostics,
            );
        }

        // Snapshot the parfor state needed for the immediate checks.
        let snapshot = context_stack
            .iter()
            .rev()
            .find(|f| f.context == LoopContext::Parfor)
            .map(|f| {
                (
                    f.parfor_analysis.parfor_var.clone(),
                    f.parfor_analysis
                        .nested_fors
                        .iter()
                        .map(|nf| nf.var.clone())
                        .collect::<Vec<String>>(),
                    f.parfor_analysis.read_only_handles.clone(),
                )
            });
        let Some((parfor_var, nested_for_vars, read_only_handles)) = snapshot else {
            return;
        };

        let is_assign_lhs = node
            .parent()
            .map(|p| {
                p.kind() == "assignment"
                    && p.child_by_field_name("left")
                        .map(|l| l.id() == node.id())
                        .unwrap_or(false)
            })
            .unwrap_or(false);

        match name.as_str() {
            "evalin" | "assignin" => {
                if first_string_argument(node, source)
                    .map(|s| s.eq_ignore_ascii_case("caller"))
                    .unwrap_or(false)
                {
                    self.push_diag(
                        node,
                        "PFEVC",
                        "EVALIN('caller') and ASSIGNIN('caller') are invalid inside of a PARFOR loop.",
                        diagnostics,
                    );
                }
            }
            "inputname" => {
                self.push_diag(
                    node,
                    "PFINPT",
                    "'inputname' is not supported in parfor loops.",
                    diagnostics,
                );
            }
            "narginchk" | "nargoutchk" => {
                self.push_diag(
                    node,
                    "PFNACK",
                    "'narginchk' and 'nargoutchk' cannot be used in parfor loops.",
                    diagnostics,
                );
            }
            "nargin" | "nargout" => {
                if argument_nodes(node).is_empty() {
                    self.push_diag(
                        node,
                        "PFNAIO",
                        "'nargin' and 'nargout' require a function argument in parfor loops.",
                        diagnostics,
                    );
                }
            }
            "load" => {
                if !is_assignment_rhs(node) && !node_is_inside_lambda(node) {
                    self.push_diag(
                        node,
                        "PFLD",
                        "'load' must assign to an output variable in parfor loops.",
                        diagnostics,
                    );
                }
            }
            "save" => {
                if !has_string_argument(node, source, "-fromstruct") {
                    self.push_diag(
                        node,
                        "PFSV",
                        "SAVE cannot be called in a PARFOR loop without the '-fromstruct' option.",
                        diagnostics,
                    );
                }
            }
            "fprintf" => {
                if self.fwrite_handle_is_read_only(node, source, &read_only_handles) {
                    self.push_diag(
                        node,
                        "FPFORP",
                        "'fprintf' is writing to a file that is opened with read permission only. \
                         Open a file using 'fopen(...,'W',...)' instead.",
                        diagnostics,
                    );
                }
            }
            "fwrite" => {
                if self.fwrite_handle_is_read_only(node, source, &read_only_handles) {
                    self.push_diag(
                        node,
                        "FWFORP",
                        "'fwrite' is writing to a file that is opened with read permission only. Open a file using 'fopen(...,'W',...)' instead.",
                        diagnostics,
                    );
                }
            }
            _ => {
                // PFCEL: the function does not support cell arrays
                if PFCEL_NO_CELL_FUNCTIONS.contains(&name.as_str()) {
                    for (i, arg) in argument_nodes(node).iter().enumerate() {
                        if is_cell_array_arg(*arg, source) {
                            self.push_diag(
                                node,
                                "PFCEL",
                                &format!(
                                    "The function {name} does not support cell arrays \
                                     (argument {}).",
                                    i + 1
                                ),
                                diagnostics,
                            );
                            break;
                        }
                    }
                }
            }
        }

        // Loop-variable indexing and indexed-access recording.
        if name == parfor_var {
            self.push_diag(
                node,
                "PFVSUB",
                "Indexing parfor loop variables is not supported in parfor loops.",
                diagnostics,
            );
        } else if nested_for_vars.contains(&name) {
            self.push_diag(
                node,
                "PFFSUB",
                "Indexing a nested for loop variable is not supported in parfor loops.",
                diagnostics,
            );
        } else if !node_is_inside_lambda(node) {
            if let Some(analysis) = current_parfor_analysis_mut(context_stack) {
                let args = arguments_text(node, source);
                // `v(...)` uses `(` while `v{...}` uses `{` for cell indexing.
                let is_cell = node
                    .child(1)
                    .map(|c| node_text(c, source) == "{")
                    .unwrap_or(false);
                analysis.var_names.insert(name.clone());
                if is_assign_lhs {
                    analysis.sliced_lhs.entry(name).or_default().push((
                        args,
                        node.start_byte(),
                        node.end_byte(),
                        is_cell,
                    ));
                } else {
                    analysis.indexed_reads.entry(name).or_default().push((
                        args,
                        node.start_byte(),
                        node.end_byte(),
                        is_cell,
                    ));
                }
            }
        }
    }

    /// Record a whole (non-indexed) read of a variable inside a parfor body.
    fn check_parfor_identifier(
        &self,
        node: tree_sitter::Node,
        source: &str,
        context_stack: &mut [ContextFrame],
    ) {
        if let Some(parent) = node.parent() {
            match parent.kind() {
                // Callee of a function call — recorded by the function_call arm.
                "function_call" => return,
                "field_expression"
                    if parent
                        .child_by_field_name("field")
                        .map(|f| f.id() == node.id())
                        .unwrap_or(false) =>
                {
                    return;
                }
                "assignment"
                    if parent
                        .child_by_field_name("left")
                        .map(|l| l.id() == node.id())
                        .unwrap_or(false) =>
                {
                    return;
                }
                "iterator" => return,
                "global_operator" | "persistent_operator" => return,
                "arguments" | "lambda_arguments" => {
                    if let Some(gp) = parent.parent() {
                        if gp.kind() == "lambda" {
                            return;
                        }
                    }
                }
                _ => {}
            }
        }
        if node_is_inside_lambda(node) {
            return;
        }
        let name = node_text(node, source).to_string();
        if let Some(analysis) = current_parfor_analysis_mut(context_stack) {
            analysis
                .whole_reads
                .entry(name.clone())
                .or_default()
                .push(node.start_byte());
            analysis.var_names.insert(name);
        }
    }

    /// Record an anonymous function in a parfor body for PFANON.
    fn check_parfor_lambda(
        &self,
        node: tree_sitter::Node,
        source: &str,
        context_stack: &mut [ContextFrame],
    ) {
        let mut params = HashSet::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "arguments" || child.kind() == "lambda_arguments" {
                collect_identifiers(child, source, &mut params);
            }
        }
        let mut refs = HashSet::new();
        let mut c2 = node.walk();
        for child in node.children(&mut c2) {
            if child.kind() == "arguments" || child.kind() == "lambda_arguments" {
                continue;
            }
            collect_lambda_refs(child, source, &params, &mut refs);
        }
        if let Some(analysis) = current_parfor_analysis_mut(context_stack) {
            analysis.lambdas.push((
                node.start_byte(),
                refs.into_iter().collect(),
                node_text(node, source).to_string(),
            ));
        }
    }

    /// The first whole-word mention of `name` in the body of a `parfor` loop. MATLAB
    /// places the sliced-output and temporary-variable reports here, not at the
    /// first invalid access (probed: `disp(r)` before `r(i) = 1`, `save('a.mat', 'r')`,
    /// `x = v(k)` before `v(i) = x`, `v(i) = 1` before `x = v(i - 1)`).
    fn first_body_mention(
        &self,
        parfor: tree_sitter::Node,
        source: &str,
        name: &str,
    ) -> Option<usize> {
        let mut cursor = parfor.walk();
        let body = parfor.children(&mut cursor).find(|c| c.kind() == "block")?;
        let text = &source[body.start_byte()..body.end_byte()];
        let bytes = text.as_bytes();
        let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        let mut from = 0;
        while let Some(pos) = text[from..].find(name) {
            let start = from + pos;
            let end = start + name.len();
            if (start == 0 || !is_word(bytes[start - 1]))
                && (end == bytes.len() || !is_word(bytes[end]))
            {
                return Some(body.start_byte() + start);
            }
            from = end;
        }
        None
    }

    /// Run the body-level parfor checks once the whole parfor body has been
    /// visited: PFSLO, PFSLRD, PFSLW, PFINCR, PFNAR, PFRFH, PFUTMP, PFUTVR,
    /// PFUNK, PFVARS, PFANON, PFCTXT, PFFRNG.
    fn finalize_parfor_analysis(
        &self,
        analysis: &ParforAnalysis,
        parfor_node: tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let predefined = self.predefined_vars_before(parfor_node, source);
        let parfor_var = analysis.parfor_var.clone();
        let mut reported_utvr = HashSet::new();

        // PFINCR: different reduction functions with the same reduction variable
        // A variable reduced only by functions (`s = max(s, i); s = min(s, i)`) has no
        // operator entry, so the keys of both maps are visited (probed: PFINCR).
        let mut reduced: Vec<&String> = analysis
            .reductions
            .keys()
            .chain(analysis.func_reductions.keys())
            .collect();
        reduced.sort();
        reduced.dedup();
        for var in reduced {
            let empty = Vec::new();
            let ops = analysis.reductions.get(var).unwrap_or(&empty);
            let mut fns: Vec<String> = ops.iter().map(|(op, _)| op.clone()).collect();
            if let Some(frs) = analysis.func_reductions.get(var) {
                fns.extend(frs.iter().map(|(c, _)| c.clone()));
            }
            fns.sort();
            fns.dedup();
            if fns.len() > 1 {
                // MATLAB reports at the first operation that differs from the first one (probed).
                let mut all: Vec<(String, usize)> = ops.clone();
                if let Some(frs) = analysis.func_reductions.get(var) {
                    all.extend(frs.iter().map(|(c, s)| (c.clone(), *s)));
                }
                all.sort_by_key(|(_, s)| *s);
                let first_op = all.first().map(|(c, _)| c.clone()).unwrap_or_default();
                let start = all
                    .iter()
                    .find(|(c, _)| *c != first_op)
                    .map(|(_, s)| *s)
                    .unwrap_or(0);
                self.push_diag_bytes(
                    source,
                    start,
                    start + 1,
                    "PFINCR",
                    "Using different reduction functions with the same reduction variable is not supported in parfor loops.",
                    diagnostics,
                );
            }
        }

        // PFNAR: subtracting a reduction variable from expressions
        for (var, start) in &analysis.sub_right {
            if analysis.both_sides.contains(var) {
                self.push_diag_bytes(
                    source,
                    *start,
                    *start + 1,
                    "PFNAR",
                    &format!("Subtracting reduction variable {var} from expressions is not supported in parfor loops."),
                    diagnostics,
                );
            }
        }

        // PFRFH: the functional reduction must be a function name or broadcast variable
        for frs in analysis.func_reductions.values() {
            for (callee, start) in frs {
                let known = PFRFH_REDUCTION_FUNCTIONS.contains(&callee.as_str());
                let is_plain_ident = callee.chars().all(|c| c.is_alphanumeric() || c == '_');
                let is_broadcast = !analysis.temp_assigns.contains_key(callee);
                if !known && (!is_plain_ident || !is_broadcast) {
                    self.push_diag_bytes(
                        source,
                        *start,
                        *start + 1,
                        "PFRFH",
                        &format!("The parfor reduction function {callee} must either be a function name or a broadcast variable."),
                        diagnostics,
                    );
                }
            }
        }

        // PFUTMP: temporary variable used before it is set in the body
        let mut pfutmp_vars: Vec<String> = Vec::new();
        for (var, set_starts) in &analysis.temp_assigns {
            // Probed: a read before the first set is PFUTMP even when the variable had a value
            // before the loop, and an indexed read (`x = u(1)`) counts. A read inside an `if`,
            // loop, `switch` or `try` is not reported. A variable with a self-update
            // (`u = u + 1`) is PFUTVR instead (probed).
            if analysis.reductions.contains_key(var) {
                continue;
            }
            // A function input read in the body is an input (PFIIN), not a temporary.
            let is_predefined = predefined.contains(var);
            if is_predefined && !assigned_before_text(source, parfor_node.start_byte(), var) {
                continue;
            }
            let first_set = set_starts.iter().min().copied();
            let mut all_reads: Vec<usize> =
                analysis.whole_reads.get(var).cloned().unwrap_or_default();
            if let Some(rs) = analysis.indexed_reads.get(var) {
                all_reads.extend(rs.iter().map(|(_, s, _, _)| *s));
            }
            let read_starts: Vec<usize> = all_reads
                .into_iter()
                .filter(|&s| top_level_read(parfor_node, s, var, source))
                .collect();
            if let (Some(fs), Some(fr)) = (first_set, read_starts.into_iter().min()) {
                if fr < fs {
                    pfutmp_vars.push(var.clone());
                    self.push_diag_bytes(
                        source,
                        fr,
                        fr + 1,
                        "PFUTMP",
                        &format!("Temporary variable '{var}' must be set inside the parfor loop before it is used."),
                        diagnostics,
                    );
                }
            }
            // `k = k(i) + 1`: the indexed read on the right of a whole-variable set comes first.
            if let (Some(fs), false) = (first_set, is_predefined) {
                let same_statement_read = analysis.indexed_reads.get(var).and_then(|rs| {
                    rs.iter()
                        .map(|(_, s, _, _)| *s)
                        .filter(|&s| s > fs && !source[fs..s].contains(['\n', ';', ',']))
                        .min()
                });
                if let Some(fr) = same_statement_read.filter(|_| !pfutmp_vars.contains(var)) {
                    pfutmp_vars.push(var.clone());
                    self.push_diag_bytes(
                        source,
                        fr,
                        fr + 1,
                        "PFUTMP",
                        &format!("Temporary variable '{var}' must be set inside the parfor loop before it is used."),
                        diagnostics,
                    );
                }
            }
        }

        // PFUTVR: variable may have been intended as a reduction but is uninitialized
        for (var, ops) in analysis
            .reductions
            .iter()
            .chain(analysis.func_reductions.iter())
        {
            if predefined.contains(var) {
                continue;
            }
            let first_red = ops.iter().map(|(_, s)| *s).min().unwrap_or(0);
            let prior_set = analysis
                .temp_assigns
                .get(var)
                .map(|v| v.iter().any(|s| *s < first_red))
                .unwrap_or(false)
                || analysis
                    .sliced_lhs
                    .get(var)
                    .map(|v| v.iter().any(|(_, s, _, _)| *s < first_red))
                    .unwrap_or(false);
            if !prior_set && reported_utvr.insert(var.clone()) {
                self.push_diag_bytes(
                    source,
                    first_red,
                    first_red + 1,
                    "PFUTVR",
                    &format!("Variable {var} may have been intended as a reduction variable, but is an uninitialized temporary."),
                    diagnostics,
                );
            }
        }

        // A temporary read as `t(i)` is not reported as PFSLO: MATLAB is silent for
        // `a = data; b(i) = a(i)` and for `f = @(x) ...; a(i) = f(i)` (probed).

        // PFSLRD: sliced output variable read without slicing
        let saved = save_named_vars(parfor_node, source);
        for var in analysis.sliced_lhs.keys() {
            // Probed: a temporary (assigned as a whole first) is not a sliced output, and
            // the report is at the first sliced write (pfunk_02, pfunk_12). A reduction
            // variable is a temporary to MATLAB (PFUTVR), not a sliced output.
            if analysis.temp_assigns.contains_key(var) || analysis.reductions.contains_key(var) {
                continue;
            }
            if saved.contains(var) && !analysis.whole_reads.contains_key(var) {
                let first = self
                    .first_body_mention(parfor_node, source, var)
                    .unwrap_or_else(|| {
                        analysis
                            .sliced_lhs
                            .get(var)
                            .and_then(|w| w.iter().map(|(_, s, _, _)| *s).min())
                            .unwrap_or(0)
                    });
                self.push_diag_bytes(
                    source,
                    first,
                    first + 1,
                    "PFSLRD",
                    &format!("Parfor loop variable {var} is accessed with an invalid combination of sliced indexing expressions and non-indexed reads. It is not valid to access the whole value of a sliced output variable."),
                    diagnostics,
                );
                continue;
            }
            if let Some(wrs) = analysis.whole_reads.get(var) {
                if let Some(first) = wrs.iter().min().copied() {
                    let first = self
                        .first_body_mention(parfor_node, source, var)
                        .unwrap_or_else(|| {
                            analysis
                                .sliced_lhs
                                .get(var)
                                .and_then(|w| w.iter().map(|(_, s, _, _)| *s).min())
                                .unwrap_or(first)
                        });
                    self.push_diag_bytes(
                        source,
                        first,
                        first + 1,
                        "PFSLRD",
                        &format!("Parfor loop variable {var} is accessed with an invalid combination of sliced indexing expressions and non-indexed reads. It is not valid to access the whole value of a sliced output variable."),
                        diagnostics,
                    );
                }
            }
        }

        // PFSLO / PFSLW for sliced outputs. MATLAB reports PFSLW only when every access
        // has the loop variable as its first subscript, all with the same number of
        // subscripts, and the lists differ (`a(i,1)` and `a(i,2)`). Any other
        // inconsistency is PFSLO, at the first sliced write (probed).
        for (var, writes) in &analysis.sliced_lhs {
            let mut accesses: Vec<&str> =
                writes.iter().map(|(args, _, _, _)| args.as_str()).collect();
            if let Some(reads) = analysis.indexed_reads.get(var) {
                accesses.extend(reads.iter().map(|(args, _, _, _)| args.as_str()));
            }
            let lists: Vec<Vec<&str>> = accesses
                .iter()
                .map(|a| a.split(',').map(str::trim).collect())
                .collect();
            // Only accesses that use the loop variable as a whole subscript form a sliced
            // output (probed: `v(k)` with a temporary `k` is not one, pfslo_07).
            if !lists.iter().any(|l| l.contains(&parfor_var.as_str())) {
                continue;
            }
            // The loop variable is one whole subscript, at the same position in every
            // access (probed: `z(1, i)` alone is valid; `a(i, 1)` with `a(1, i)` is not).
            let positions: Vec<Option<usize>> = lists
                .iter()
                .map(|l| {
                    let hits: Vec<usize> = l
                        .iter()
                        .enumerate()
                        .filter(|(_, t)| **t == parfor_var.as_str())
                        .map(|(p, _)| p)
                        .collect();
                    (hits.len() == 1).then(|| hits[0])
                })
                .collect();
            // A temporary in a subscript list is not a sliced subscript (probed: `q = 1;
            // v(q, i) = 1` is PFSLO).
            let temp_subscript = lists.iter().flatten().any(|t| {
                t.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .any(|tok| {
                        tok != parfor_var.as_str() && analysis.temp_assigns.contains_key(tok)
                    })
            });
            let consistent = !temp_subscript
                && lists.iter().zip(&positions).all(|(l, p)| {
                    p.is_some()
                        && l.iter()
                            .enumerate()
                            .all(|(q, t)| Some(q) == *p || !args_tokens(t).contains(&parfor_var))
                })
                && positions.iter().all(|p| *p == positions[0]);
            let same_count = lists.windows(2).all(|w| w[0].len() == w[1].len());
            let first = writes.iter().map(|(_, s, _, _)| *s).min().unwrap_or(0);
            if !consistent || !same_count {
                let first = self
                    .first_body_mention(parfor_node, source, var)
                    .unwrap_or(first);
                self.push_diag_bytes(
                    source,
                    first,
                    first + 1,
                    "PFSLO",
                    &format!("Variable {var} is indexed using the parfor loop variable, but it is not a valid sliced output variable."),
                    diagnostics,
                );
            } else if lists.iter().any(|l| *l != lists[0]) {
                self.push_diag_bytes(
                    source,
                    first,
                    first + 1,
                    "PFSLW",
                    &format!("Parfor loop variable {var} has multiple sliced accesses, but they do not all have the same list of subscripts. Each access to a sliced variable must use precisely the same list of subscripts."),
                    diagnostics,
                );
            }
        }

        // PFUNK for writes that are neither sliced nor temporary subscripts: a variable
        // with no write subscripted by the loop variable or by a bare name, and a write
        // whose subscripts are literals, ranges, `end`, calls or fields (`x(3) = 1`,
        // `x(1:3) = 1`, `x(end) = 1`, `x(v(i)) = 1`, `x{1} = i`) (probed).
        let mut unclassified: Vec<(&String, usize)> = Vec::new();
        for (var, writes) in &analysis.sliced_lhs {
            if !predefined.contains(var) || analysis.temp_assigns.contains_key(var) {
                continue;
            }
            if analysis.reductions.contains_key(var) {
                continue;
            }
            let classified = writes.iter().any(|(args, _, _, _)| {
                args.split(',').any(|sub| {
                    let sub: String = sub.trim().chars().filter(|c| !c.is_whitespace()).collect();
                    is_sliced_text(&sub, &parfor_var)
                        || (sub != "end"
                            && sub.starts_with(|c: char| c.is_alphabetic() || c == '_')
                            && sub.chars().all(|c| c.is_alphanumeric() || c == '_'))
                })
            });
            if classified {
                continue;
            }
            if let Some(start) = writes.iter().map(|(_, s, _, _)| *s).min() {
                unclassified.push((var, start));
            }
        }
        for (var, start) in unclassified {
            self.push_diag_bytes(
                source,
                start,
                start + 1,
                "PFUNK",
                &format!("Unable to classify variable '{var}' in the body of the parfor loop."),
                diagnostics,
            );
        }

        // PFUNK for a field of a predefined variable written in the body (`s.f = 1`,
        // `s.f(i) = 1`), at the assignment (probed: `s.i = 1` and `v(i) = s.i`).
        let mut field_writes: Vec<(String, usize)> = Vec::new();
        let mut walk = vec![parfor_node];
        while let Some(m) = walk.pop() {
            if m.kind() == "lambda" {
                continue;
            }
            if m.kind() == "assignment" {
                if let Some(left) = m.child_by_field_name("left") {
                    let base = match left.kind() {
                        "field_expression" => left.named_child(0),
                        "function_call" => left
                            .named_child(0)
                            .filter(|b| b.kind() == "field_expression")
                            .and_then(|b| b.named_child(0)),
                        _ => None,
                    };
                    if let Some(b) = base.filter(|b| b.kind() == "identifier") {
                        field_writes.push((node_text(b, source).to_string(), m.start_byte()));
                    }
                }
            }
            let mut c = m.walk();
            walk.extend(m.children(&mut c));
        }
        field_writes.sort_by_key(|(_, start)| *start);
        let mut reported_fields: HashSet<String> = HashSet::new();
        for (var, start) in field_writes {
            if var == parfor_var
                || !(predefined.contains(&var)
                    || assigned_before_text(source, parfor_node.start_byte(), &var))
                || !reported_fields.insert(var.clone())
            {
                continue;
            }
            self.push_diag_bytes(
                source,
                start,
                start + 1,
                "PFUNK",
                &format!("Unable to classify variable '{var}' in the body of the parfor loop."),
                diagnostics,
            );
        }

        // PFUNK is not emitted for a variable that is both sliced and assigned as a
        // whole. Probed: MATLAB is silent for `a(i) = i; a = 3` and for `q = zeros(3,1);
        // q(i) = 1`, and reports PFUNK only for less common shapes such as a sliced
        // variable indexed by a temporary (`k = i; v(k) = 1`).

        // PFUNK: a sliced write subscripted by an inner loop variable (`for j=1:3, y(j) = i;
        // end`), whether or not `y` had a value before the loop (probed). A variable that
        // is also indexed by the loop variable is classified and stays silent.
        let mut inner_loop_vars: HashSet<String> = HashSet::new();
        let mut walk_loops = vec![parfor_node];
        while let Some(m) = walk_loops.pop() {
            if m.kind() == "for_statement"
                && m.id() != parfor_node.id()
                && !node_text(m, source).starts_with("parfor")
            {
                if let Some(var) = self.extract_for_variable(m, source) {
                    inner_loop_vars.insert(var);
                }
            }
            let mut c = m.walk();
            walk_loops.extend(m.children(&mut c));
        }
        if !inner_loop_vars.is_empty() {
            let tokens = |args: &str| -> Vec<String> {
                args.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .map(str::to_string)
                    .collect()
            };
            for (var, writes) in &analysis.sliced_lhs {
                let loop_indexed = |args: &str| tokens(args).contains(&parfor_var);
                if analysis.temp_assigns.contains_key(var)
                    || writes
                        .iter()
                        .any(|(args, _, _, _)| loop_indexed(args.as_str()))
                    || analysis.indexed_reads.get(var).is_some_and(|rs| {
                        rs.iter().any(|(args, _, _, _)| loop_indexed(args.as_str()))
                    })
                {
                    continue;
                }
                let first = writes
                    .iter()
                    .filter(|(args, _, _, _)| {
                        tokens(args.as_str())
                            .iter()
                            .any(|t| inner_loop_vars.contains(t))
                    })
                    .map(|(_, start, _, _)| *start)
                    .min();
                if let Some(start) = first {
                    self.push_diag_bytes(
                        source,
                        start,
                        start + 1,
                        "PFUNK",
                        &format!(
                            "Unable to classify variable '{var}' in the body of the parfor loop."
                        ),
                        diagnostics,
                    );
                }
            }
        }

        // PFUNK: a sliced write whose subscript is a temporary of the body, as in
        // `k = i; v(k) = 1`. Probed: MATLAB reports it whatever the temporary's value
        // (`k = 2; v(k) = i`) and whether `k` is reassigned later (`k = 2` after the write).
        for (var, writes) in &analysis.sliced_lhs {
            // A variable that is also indexed by the loop variable is PFSLO, not PFUNK
            // (probed: `w = v(i); v(w) = 1` and `k = i; v(k) = 1; v(i) = 2`).
            let loop_indexed = |args: &str| {
                args.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .any(|t| t == parfor_var.as_str())
            };
            if writes
                .iter()
                .any(|(args, _, _, _)| loop_indexed(args.as_str()))
                || analysis
                    .indexed_reads
                    .get(var)
                    .is_some_and(|rs| rs.iter().any(|(args, _, _, _)| loop_indexed(args.as_str())))
            {
                continue;
            }
            let mut first_write: Option<usize> = None;
            for (args, start, _end, _is_cell) in writes {
                let uses_temp = args
                    .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .any(|t| t != analysis.parfor_var && analysis.temp_assigns.contains_key(t));
                if uses_temp
                    && !args
                        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .any(|t| t == analysis.parfor_var)
                {
                    first_write = Some(first_write.map_or(*start, |f| f.min(*start)));
                }
            }
            if let Some(start) = first_write {
                self.push_diag_bytes(
                    source,
                    start,
                    start + 1,
                    "PFUNK",
                    &format!("Unable to classify variable '{var}' in the body of the parfor loop."),
                    diagnostics,
                );
            }
        }

        // Anonymous functions in the body (probed on R2026b): a sliced output read whole is
        // PFSLRD and one indexed by anything but the plain loop variable (`x(1)`, `x(i - 1)`,
        // `x(t)` with a lambda parameter) is PFSLO, both at the first mention of the variable
        // in the body; `x(i)` is PFANON at the variable inside the anonymous function.
        for (start, refs, text) in &analysis.lambdas {
            for var in refs.iter().filter(|r| analysis.sliced_lhs.contains_key(*r)) {
                let uses = lambda_uses(text, var, &parfor_var);
                let first = self
                    .first_body_mention(parfor_node, source, var)
                    .unwrap_or(*start);
                let (id, message, at) = if uses.whole {
                    ("PFSLRD", format!("Parfor loop variable {var} is accessed with an invalid combination of sliced indexing expressions and non-indexed reads. It is not valid to access the whole value of a sliced output variable."), first)
                } else if let Some(offset) = uses.loop_indexed {
                    ("PFANON", "Using a sliced output variable in an anonymous function is not supported in parfor loops.".to_string(), *start + offset)
                } else if uses.constant {
                    ("PFSLO", format!("Variable {var} is indexed using the parfor loop variable, but it is not a valid sliced output variable."), first)
                } else {
                    continue;
                };
                self.push_diag_bytes(source, at, at + 1, id, &message, diagnostics);
                break;
            }
        }

        // PFCTXT: a sliced variable indexed with a nested for loop variable must be
        // inside the for loop that defines its range.
        let mut accesses: Vec<(String, String, usize, usize)> = Vec::new();
        for (var, writes) in &analysis.sliced_lhs {
            for (args, start, end, _) in writes {
                accesses.push((var.clone(), args.clone(), *start, *end));
            }
        }
        for (var, reads) in &analysis.indexed_reads {
            if analysis.sliced_lhs.contains_key(var) {
                for (args, start, end, _) in reads {
                    accesses.push((var.clone(), args.clone(), *start, *end));
                }
            }
        }
        for (_, args, start, end) in accesses {
            let tokens = args_tokens(&args);
            for nf in &analysis.nested_fors {
                if tokens.contains(&nf.var) && !(start >= nf.for_start && start <= nf.for_end) {
                    self.push_diag_bytes(
                        source,
                        start,
                        end,
                        "PFCTXT",
                        "When indexing a sliced variable with a nested for loop variable, the sliced variable must be inside the for loop that defines the range of the for loop variable.",
                        diagnostics,
                    );
                }
            }
        }

        // PFFRNG: nested for loop range must be a row of positive constants when it
        // indexes a sliced variable.
        for nf in &analysis.nested_fors {
            let indexes_sliced = analysis.sliced_lhs.iter().any(|(_, writes)| {
                writes
                    .iter()
                    .any(|(args, _, _, _)| args_tokens(args).contains(&nf.var))
            }) || analysis.indexed_reads.iter().any(|(v, reads)| {
                analysis.sliced_lhs.contains_key(v)
                    && reads
                        .iter()
                        .any(|(args, _, _, _)| args_tokens(args).contains(&nf.var))
            });
            if indexes_sliced && !nf.range_is_const_pos {
                self.push_diag_bytes(
                    source,
                    nf.range_start,
                    nf.range_end,
                    "PFFRNG",
                    "When indexing a sliced variable with a nested for loop variable, the \
                     range of the for loop variable must be a row vector of positive constant numbers or variables.",
                    diagnostics,
                );
            }
        }

        // PFVARS: too many variables in the parfor loop
        let count = analysis.var_names.len() + usize::from(!analysis.parfor_var.is_empty());
        if count > PARFOR_VAR_LIMIT {
            self.push_diag_bytes(
                source,
                analysis.parfor_start,
                analysis.parfor_end,
                "PFVARS",
                "Parfor loop contains too many variables.",
                diagnostics,
            );
        }
    }

    /// Determine whether `fprintf`/`fwrite` writes to a file handle opened
    /// read-only (via a prior `fid = fopen(...)` or an inline `fopen(...)`).
    pub(crate) fn fwrite_handle_is_read_only(
        &self,
        node: tree_sitter::Node,
        source: &str,
        read_only_handles: &HashSet<String>,
    ) -> bool {
        let Some(first_arg) = argument_nodes(node).first().copied() else {
            return false;
        };
        match first_arg.kind() {
            "identifier" => read_only_handles.contains(node_text(first_arg, source)),
            "function_call" => {
                callee_name(first_arg, source).as_deref() == Some("fopen")
                    && fopen_is_read_only(first_arg, source)
            }
            _ => false,
        }
    }

    /// Collect the variables that are predefined before the parfor loop starts
    /// (function inputs/outputs, global/persistent declarations, and assignments
    /// in the enclosing scope before the parfor).
    pub(crate) fn predefined_vars_before(
        &self,
        node: tree_sitter::Node,
        source: &str,
    ) -> HashSet<String> {
        let mut predefined = HashSet::new();
        let stop = node.start_byte();
        let mut func: Option<tree_sitter::Node> = None;
        let mut top: tree_sitter::Node = node;
        let mut cur = node.parent();
        while let Some(p) = cur {
            if p.kind() == "function_definition" {
                func = Some(p);
                break;
            }
            top = p;
            cur = p.parent();
        }
        if let Some(f) = func {
            let mut cursor = f.walk();
            for child in f.children(&mut cursor) {
                match child.kind() {
                    "function_arguments" | "function_output" => {
                        collect_identifiers(child, source, &mut predefined);
                    }
                    "block" => {
                        collect_assigns_before(child, stop, source, &mut predefined);
                    }
                    _ => {}
                }
            }
        } else {
            collect_assigns_before(top, stop, source, &mut predefined);
        }
        predefined
    }

    /// Return the byte range of a nested for loop's range expression and whether
    /// it consists of positive constant numbers.
    pub(crate) fn nested_for_range_info(
        &self,
        node: tree_sitter::Node,
        source: &str,
    ) -> (usize, usize, bool) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "iterator" {
                let mut inner = child.walk();
                for c in child.children(&mut inner) {
                    if c.kind() == "range" {
                        return (
                            c.start_byte(),
                            c.end_byte(),
                            range_is_positive_constants(c, source),
                        );
                    }
                }
            }
        }
        (node.start_byte(), node.start_byte(), false)
    }

    /// Push a diagnostic at a byte range (used by the body-level finalize checks).
    pub(crate) fn push_diag_bytes(
        &self,
        source: &str,
        start: usize,
        end: usize,
        rule_id: &'static str,
        message: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled(rule_id) {
            return;
        }
        let start = start.min(source.len());
        let end = end.min(source.len());
        let line = source[..start].bytes().filter(|&b| b == b'\n').count() + 1;
        let column = source[..start]
            .bytes()
            .rev()
            .take_while(|&b| b != b'\n')
            .count()
            + 1;
        diagnostics.push(Diagnostic {
            rule_id,
            message: message.to_string(),
            severity: Severity::Error,
            byte_range: start..end,
            line,
            column,
            fix: None,
        });
    }
}

impl LanguageSpecEngine {
    /// Check all class/OOP related rules.
    fn check_class_rules(
        &self,
        class: &ClassMeta,
        meta: &FileMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.check_mcfil(class, ctx, diagnostics);
        self.check_mcdir(class, ctx, diagnostics);
        self.check_mcred(class, diagnostics);
        self.check_mccbd(class, meta, ctx, diagnostics);
        self.check_setter_getter_signatures(class, meta, diagnostics);
        let value_bases = class.has_only_value_bases(ctx.project, ctx.file_path);
        self.check_mceb(class, value_bases, diagnostics);
        self.check_mcani(class, diagnostics);
        self.check_mcasc(class, diagnostics);
        self.check_mcsga(class, meta, diagnostics);
        self.check_mcsgp(class, meta, diagnostics);
        self.check_ctoro(class, diagnostics);
        self.check_mherit(class, diagnostics);
        self.check_ctoine(class, ctx, diagnostics);
        self.check_mabseac(class, diagnostics);
        self.check_mcswa(class, diagnostics);
        self.check_mtmat(class, diagnostics);
        self.check_mtags3(class, diagnostics);
        self.check_mabseam(class, diagnostics);
        self.check_mcmsp(class, diagnostics);
        self.check_mcapp(class, diagnostics);
        self.check_mcgsa(class, diagnostics);
        self.check_mcpsg(class, diagnostics);
        self.check_mccsop(class, ctx, diagnostics);
        self.check_mwkcl(class, diagnostics);
        self.check_mwkct(class, diagnostics);
        self.check_mwkref(class, diagnostics);
        self.check_superclass_rules(class, ctx, diagnostics);
        self.check_mcmio(class, diagnostics);
        self.check_mcmtp(class, diagnostics);
        self.check_mcpin(class, diagnostics);
        self.check_class_attributes(class, diagnostics);
    }
}

impl LanguageSpecEngine {
    /// Check whether a specific sub-check ID is enabled.
    pub(crate) fn is_check_enabled(&self, check_id: &str) -> bool {
        !self.config.disabled_checks.iter().any(|id| id == check_id)
    }

    /// Collect all `function_definition` nodes in a tree with their names.
    pub(crate) fn collect_c2_function_definitions<'a>(
        &self,
        root: tree_sitter::Node<'a>,
        source: &str,
    ) -> Vec<(tree_sitter::Node<'a>, String)> {
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            if n.kind() == "function_definition" {
                let name = n
                    .child_by_field_name("name")
                    .map(|f| node_text(f, source).to_string())
                    .unwrap_or_default();
                out.push((n, name));
            }
            let mut cursor = n.walk();
            for child in n.children(&mut cursor) {
                stack.push(child);
            }
        }
        out
    }
}

impl LanguageSpecEngine {
    /// Check function validation rules.
    fn check_function_validation(
        &self,
        meta: &FileMeta,
        ctx: &FileContext,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.check_fvnst(ctx, diagnostics);
        self.check_vtpod(meta, diagnostics);

        // VTP* property-validation function checks (class methods blocks).
        self.check_property_validation_functions(ctx, diagnostics);

        // Walk the tree to collect top-level function definitions (matched to
        // metadata by byte range) and the names of nested functions.
        let mut function_nodes = Vec::new();
        let mut nested_function_names = Vec::new();
        Self::collect_function_definitions(
            ctx.tree.root_node(),
            0,
            ctx.source,
            &mut function_nodes,
            &mut nested_function_names,
        );

        for function_node in function_nodes {
            let func_meta = meta
                .functions
                .iter()
                .chain(meta.local_functions.iter())
                .find(|f| {
                    f.byte_range.start == function_node.start_byte()
                        && f.byte_range.end == function_node.end_byte()
                });
            let Some(func_meta) = func_meta else {
                continue;
            };

            let blocks = extract_arguments_blocks_grouped(function_node, ctx.source);
            if blocks.is_empty() {
                continue;
            }

            self.check_fv_structure(&blocks, ctx.source, diagnostics);
            self.check_fv_ordering(func_meta, &blocks, ctx.source, diagnostics);
            self.check_fv_validation(&blocks, &nested_function_names, ctx.source, diagnostics);
        }
    }

    /// Collect top-level `function_definition` nodes (depth 0) and the names
    /// of nested functions (depth >= 1).
    fn collect_function_definitions<'a>(
        node: tree_sitter::Node<'a>,
        depth: usize,
        source: &str,
        out_nodes: &mut Vec<tree_sitter::Node<'a>>,
        out_nested_names: &mut Vec<String>,
    ) {
        if node.kind() == "function_definition" {
            if depth > 0 {
                if let Some(name) = node.child_by_field_name("name") {
                    out_nested_names.push(node_text(name, source).to_string());
                }
            } else {
                out_nodes.push(node);
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                Self::collect_function_definitions(
                    child,
                    depth + 1,
                    source,
                    out_nodes,
                    out_nested_names,
                );
            }
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_function_definitions(child, depth, source, out_nodes, out_nested_names);
        }
    }
}

impl LanguageSpecEngine {
    /// Push a diagnostic at the start of an arguments block.
    pub(crate) fn push_block_diag(
        &self,
        block: &ArgumentsBlockMeta,
        rule_id: &'static str,
        message: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let pos = block.node.start_position();
        diagnostics.push(Diagnostic {
            rule_id,
            message: message.to_string(),
            severity: Severity::Error,
            byte_range: block.byte_range.clone(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        });
    }

    /// Like [`Self::push_block_diag`], but reported at the declaration line of
    /// argument `name` inside the block (MATLAB points at the argument, probed).
    /// Falls back to the block header when the argument is not found.
    pub(crate) fn push_arg_diag(
        &self,
        block: &ArgumentsBlockMeta,
        name: &str,
        rule_id: &'static str,
        message: &str,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let header = block.node.start_position();
        let text = &source[block.byte_range.clone()];
        let (line, column) = text
            .lines()
            .enumerate()
            .skip(1)
            .find_map(|(i, l)| {
                let trimmed = l.trim_start();
                let first = trimmed
                    .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .next();
                (first == Some(name)).then(|| (header.row + i + 1, l.len() - trimmed.len() + 1))
            })
            .unwrap_or((header.row + 1, header.column + 1));
        diagnostics.push(Diagnostic {
            rule_id,
            message: message.to_string(),
            severity: Severity::Error,
            byte_range: block.byte_range.clone(),
            line,
            column,
            fix: None,
        });
    }
}

impl LanguageSpecEngine {
    /// Collect the names of all nested functions in the file.
    ///
    /// Nested functions are `function_definition` nodes whose ancestor is another
    /// `function_definition`. Methods in `methods` blocks are not nested functions.
    fn collect_nested_function_names(
        &self,
        root: tree_sitter::Node,
        source: &str,
    ) -> HashSet<String> {
        let mut names = HashSet::new();
        Self::collect_nested_functions_dfs(root, false, source, &mut names);
        names
    }

    /// DFS helper for [`Self::collect_nested_function_names`].
    fn collect_nested_functions_dfs(
        node: tree_sitter::Node,
        inside_function: bool,
        source: &str,
        names: &mut HashSet<String>,
    ) {
        if node.kind() == "function_definition" {
            if inside_function {
                if let Some(name_node) = node.child_by_field_name("name") {
                    names.insert(node_text(name_node, source).to_string());
                }
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                Self::collect_nested_functions_dfs(child, true, source, names);
            }
            return;
        }
        if node.kind() == "class_definition" || node.kind() == "methods" {
            // Methods are not nested functions; do not mark them as such.
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                Self::collect_nested_functions_dfs(child, inside_function, source, names);
            }
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_nested_functions_dfs(child, inside_function, source, names);
        }
    }

    /// Push a diagnostic at the start of a node.
    pub(crate) fn push_diag(
        &self,
        node: tree_sitter::Node,
        rule_id: &'static str,
        message: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.is_check_enabled(rule_id) {
            return;
        }
        let pos = node.start_position();
        diagnostics.push(Diagnostic {
            rule_id,
            message: message.to_string(),
            severity: Severity::Error,
            byte_range: node.start_byte()..node.end_byte(),
            line: pos.row + 1,
            column: pos.column + 1,
            fix: None,
        });
    }
}

/// Get the innermost active parfor frame's analysis state, if any.
pub(crate) fn current_parfor_analysis_mut(
    context_stack: &mut [ContextFrame],
) -> Option<&mut ParforAnalysis> {
    context_stack
        .iter_mut()
        .rev()
        .find(|f| f.context == LoopContext::Parfor)
        .map(|f| &mut f.parfor_analysis)
}

/// Whether `node` is nested inside a `lambda` node.
pub(crate) fn node_is_inside_lambda(node: tree_sitter::Node) -> bool {
    let mut cur = node.parent();
    while let Some(p) = cur {
        if p.kind() == "lambda" {
            return true;
        }
        cur = p.parent();
    }
    false
}

/// Extract the operator text of a binary/boolean/comparison operator node.
pub(crate) fn operator_text(node: tree_sitter::Node, source: &str) -> String {
    if let Some(op) = node.child_by_field_name("operator") {
        return node_text(op, source).trim().to_string();
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if !child.is_named() {
            let t = node_text(child, source).trim();
            if !t.is_empty() && t != "(" && t != ")" && t != "," {
                return t.to_string();
            }
        }
    }
    String::new()
}

/// Extract the text of the `arguments` child of a function call.
pub(crate) fn arguments_text(node: tree_sitter::Node, source: &str) -> String {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "arguments" {
            return node_text(child, source).to_string();
        }
    }
    String::new()
}

/// Tokenize an arguments text into identifier-like tokens.
/// `i` or `i+k` / `i-k` for a constant `k` (whitespace-free text).
fn is_sliced_text(text: &str, parfor_var: &str) -> bool {
    if text == parfor_var {
        return true;
    }
    text.strip_prefix(parfor_var)
        .and_then(|rest| rest.strip_prefix(['+', '-']))
        .is_some_and(|k| !k.is_empty() && k.chars().all(|c| c.is_ascii_digit()))
}

/// Whether a line before `before` starts with `name` followed by `=`, `.`, `(` or `{`.
fn assigned_before_text(source: &str, before: usize, name: &str) -> bool {
    source[..before].lines().any(|line| {
        line.trim_start()
            .strip_prefix(name)
            .and_then(|rest| rest.trim_start().chars().next())
            .is_some_and(|c| matches!(c, '=' | '.' | '(' | '{'))
    })
}

/// Variables named by a `save` in a parfor body, as `save('f.mat', 'v')` or
/// `save f v` (probed: both read the whole value, PFSLRD). The first argument is the
/// file, so it is not a variable.
/// Whether the read of `var` that starts at byte `start` is a statement of the body of
/// `parfor`: no `if`, loop, `switch`, `try` or anonymous function lies between them
/// (probed: MATLAB reports PFUTMP for a top-level read only).
fn top_level_read(parfor: tree_sitter::Node, start: usize, var: &str, source: &str) -> bool {
    let Some(mut node) = parfor.descendant_for_byte_range(start, start + 1) else {
        return false;
    };
    while let Some(parent) = node.parent() {
        if parent.kind() == "block" && parent.parent().is_some_and(|p| p.id() == parfor.id()) {
            // `var = var + i` reads `var` before its own store: it is a reduction, not a read
            // before the first set (probed: no PFUTMP for `s = s + i; s = 2 * s`).
            let self_update = node.kind() == "assignment"
                && node
                    .child_by_field_name("left")
                    .is_some_and(|l| &source[l.byte_range()] == var);
            return !self_update;
        }
        if parent.id() == parfor.id()
            || matches!(
                parent.kind(),
                "if_statement"
                    | "for_statement"
                    | "while_statement"
                    | "switch_statement"
                    | "try_statement"
                    | "lambda"
            )
        {
            return false;
        }
        node = parent;
    }
    false
}

fn save_named_vars(parfor_node: tree_sitter::Node, source: &str) -> HashSet<String> {
    let mut names = HashSet::new();
    let mut walk = vec![parfor_node];
    while let Some(m) = walk.pop() {
        if m.kind() == "function_call"
            && m.named_child(0)
                .is_some_and(|b| node_text(b, source) == "save")
        {
            for arg in argument_nodes(m).into_iter().skip(1) {
                let text = node_text(arg, source).trim();
                let name = text.trim_matches('\'');
                if name.chars().all(|c| c.is_alphanumeric() || c == '_') && !name.is_empty() {
                    names.insert(name.to_string());
                }
            }
        } else if m.kind() == "command"
            && node_text(m, source).split_whitespace().next() == Some("save")
        {
            for word in node_text(m, source).split_whitespace().skip(2) {
                names.insert(word.to_string());
            }
        }
        let mut c = m.walk();
        walk.extend(m.children(&mut c));
    }
    names
}

/// How a variable is used in the text of an anonymous function: `whole` for a bare
/// read, `constant` for an index without the loop variable, `loop_indexed` for one
/// with it.
struct LambdaUses {
    whole: bool,
    constant: bool,
    /// The byte offset in the lambda text of the first `x(i)`, an index that is exactly
    /// the loop variable.
    loop_indexed: Option<usize>,
}

fn lambda_uses(text: &str, var: &str, parfor_var: &str) -> LambdaUses {
    let bytes = text.as_bytes();
    let is_word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut uses = LambdaUses {
        whole: false,
        constant: false,
        loop_indexed: None,
    };
    let mut search = 0;
    while let Some(found) = text[search..].find(var) {
        let at = search + found;
        let end = at + var.len();
        search = end;
        let before_ok = at == 0 || (!is_word(bytes[at - 1]) && bytes[at - 1] != b'.');
        let after_ok = end >= bytes.len() || !is_word(bytes[end]);
        if !before_ok || !after_ok {
            continue;
        }
        let rest = text[end..].trim_start();
        if !rest.starts_with('(') {
            uses.whole = true;
            continue;
        }
        let open = text.len() - rest.len();
        let mut depth = 0;
        let mut close = text.len();
        for (k, c) in text[open..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close = open + k;
                        break;
                    }
                }
                _ => {}
            }
        }
        let args = &text[open + 1..close.min(text.len())];
        if args.split(',').any(|a| a.trim() == parfor_var) {
            uses.loop_indexed.get_or_insert(at);
        } else {
            uses.constant = true;
        }
    }
    uses
}

pub(crate) fn args_tokens(args: &str) -> Vec<String> {
    args.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Whether `name` occurs as an identifier anywhere inside `node`.
pub(crate) fn contains_identifier(node: tree_sitter::Node, source: &str, name: &str) -> bool {
    if node.kind() == "identifier" {
        return node_text(node, source) == name;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if contains_identifier(child, source, name) {
            return true;
        }
    }
    false
}

/// Whether an `fopen` call opens the file read-only.
pub(crate) fn fopen_is_read_only(call: tree_sitter::Node, source: &str) -> bool {
    let args = argument_nodes(call);
    match args.get(1) {
        Some(mode_arg) => {
            let mode = node_text(*mode_arg, source).trim();
            let mode = mode.trim_matches('\'').trim_matches('"');
            fopen_mode_is_read_only(mode)
        }
        // fopen(filename) defaults to read permission.
        None => true,
    }
}

/// Whether an `fopen` mode string grants read permission only.
pub(crate) fn fopen_mode_is_read_only(mode: &str) -> bool {
    let m = mode.to_ascii_lowercase();
    !m.contains('w') && !m.contains('a') && !m.contains('+') && m.contains('r')
}

/// Whether a function argument is a cell array (literal `{...}` or `cell(...)`).
pub(crate) fn is_cell_array_arg(arg: tree_sitter::Node, source: &str) -> bool {
    match arg.kind() {
        "cell" => true,
        "function_call" => callee_name(arg, source).as_deref() == Some("cell"),
        _ => false,
    }
}

/// Whether a `range` node consists entirely of positive constant numbers.
pub(crate) fn range_is_positive_constants(range_node: tree_sitter::Node, source: &str) -> bool {
    let mut found = 0;
    let mut all_pos = true;
    let mut cursor = range_node.walk();
    for child in range_node.children(&mut cursor) {
        if child.is_named() {
            found += 1;
            if child.kind() == "number" {
                if let Ok(n) = node_text(child, source).parse::<i64>() {
                    if n <= 0 {
                        all_pos = false;
                    }
                } else {
                    all_pos = false;
                }
            } else {
                all_pos = false;
            }
        }
    }
    found > 0 && all_pos
}

/// Collect all identifier texts inside `node`.
pub(crate) fn collect_identifiers(
    node: tree_sitter::Node,
    source: &str,
    out: &mut HashSet<String>,
) {
    if node.kind() == "identifier" {
        out.insert(node_text(node, source).to_string());
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_identifiers(child, source, out);
    }
}

/// Collect identifiers referenced inside a lambda body, excluding its parameters.
pub(crate) fn collect_lambda_refs(
    node: tree_sitter::Node,
    source: &str,
    params: &HashSet<String>,
    out: &mut HashSet<String>,
) {
    if node.kind() == "identifier" {
        let name = node_text(node, source).to_string();
        if !params.contains(&name) {
            out.insert(name);
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_lambda_refs(child, source, params, out);
    }
}

/// Collect assignment LHS names and global/persistent names in nodes that start
/// before `stop` bytes (used to find variables predefined before a parfor loop).
pub(crate) fn collect_assigns_before(
    node: tree_sitter::Node,
    stop: usize,
    source: &str,
    out: &mut HashSet<String>,
) {
    if node.start_byte() >= stop {
        return;
    }
    match node.kind() {
        "assignment" => {
            if let Some(lhs) = node.child_by_field_name("left") {
                if lhs.kind() == "identifier" {
                    out.insert(node_text(lhs, source).to_string());
                }
            }
        }
        "global_operator" | "persistent_operator" => {
            collect_identifiers(node, source, out);
        }
        "function_definition" => return,
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_assigns_before(child, stop, source, out);
    }
}

/// Find the first child of a node with the given kind.
pub(crate) fn find_child_kind<'a>(
    node: tree_sitter::Node<'a>,
    kind: &str,
) -> Option<tree_sitter::Node<'a>> {
    let mut cursor = node.walk();
    let mut result = None;
    for child in node.children(&mut cursor) {
        if child.kind() == kind {
            result = Some(child);
            break;
        }
    }
    result
}

/// Collect the output argument names of a function (excluding `~`).
pub(crate) fn function_output_names(node: tree_sitter::Node, source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "function_output" {
            collect_identifiers_recursive(child, source, &mut out);
        }
    }
    out
}

/// Collect all identifier names within a node.
pub(crate) fn collect_identifiers_recursive(
    node: tree_sitter::Node,
    source: &str,
    out: &mut Vec<String>,
) {
    if node.kind() == "identifier" {
        out.push(node_text(node, source).to_string());
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_identifiers_recursive(child, source, out);
    }
}

/// Whether a node subtree references the given identifier name.
pub(crate) fn node_contains_identifier(node: tree_sitter::Node, name: &str, source: &str) -> bool {
    if node.kind() == "identifier" && node_text(node, source) == name {
        return true;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if node_contains_identifier(child, name, source) {
            return true;
        }
    }
    false
}

/// Get the property name of a `field_expression` assignment LHS (e.g., `obj.x` → `x`).
pub(crate) fn assignment_field_property(node: tree_sitter::Node, source: &str) -> Option<String> {
    let lhs = node.child_by_field_name("left")?;
    if lhs.kind() != "field_expression" {
        return None;
    }
    let last = lhs.child(lhs.child_count() - 1)?;
    if last.kind() == "identifier" {
        Some(node_text(last, source).to_string())
    } else {
        None
    }
}

/// Whether a property default value creates an instance of `class_name`.
pub(crate) fn contains_self_constructor_call(default_value: &str, class_name: &str) -> bool {
    let needle = format!("{class_name}(");
    let mut start = 0;
    while let Some(idx) = default_value[start..].find(&needle) {
        let abs = start + idx;
        let prev_is_ident = abs
            .checked_sub(1)
            .and_then(|i| default_value.as_bytes().get(i))
            .copied()
            .map(|b| b.is_ascii_alphanumeric() || b == b'_')
            .unwrap_or(false);
        if !prev_is_ident {
            return true;
        }
        start = abs + 1;
    }
    false
}

// ---------------------------------------------------------------------------
// Arguments block structure and validation checks (FV* / TIN* / TTOO* checks)
// ---------------------------------------------------------------------------

/// The role of an `arguments ... end` block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockRole {
    /// Explicit `(Input)` block.
    Input,
    /// Explicit `(Output)` block.
    Output,
    /// Block with the `Repeating` attribute.
    Repeating,
    /// No attributes — implicitly an input block.
    Default,
}

impl BlockRole {
    /// Whether this block declares input arguments.
    fn is_input(self) -> bool {
        matches!(self, BlockRole::Input | BlockRole::Default)
    }

    /// Whether this block declares output arguments.
    fn is_output(self) -> bool {
        matches!(self, BlockRole::Output)
    }

    /// Whether this is a repeating block.
    fn is_repeating(self) -> bool {
        matches!(self, BlockRole::Repeating)
    }
}

/// Grouped metadata for a single `arguments ... end` block.
#[derive(Debug, Clone)]
pub(crate) struct ArgumentsBlockMeta<'a> {
    /// The raw `arguments_statement` AST node.
    node: tree_sitter::Node<'a>,
    /// Attribute identifiers declared on the block.
    attributes: Vec<String>,
    /// The role derived from the block attributes.
    role: BlockRole,
    /// Per-argument validation entries in declaration order.
    args: Vec<ArgValidation>,
    /// Byte range of the block.
    byte_range: Range<usize>,
}

impl ArgumentsBlockMeta<'_> {
    /// Whether this block declares repeating output arguments.
    fn is_output_repeating(&self) -> bool {
        self.role.is_repeating()
            && (self.attributes.iter().any(|a| a == "Output")
                || self.args.iter().any(|a| a.name == "varargout"))
    }

    /// Whether this block declares repeating input arguments.
    fn is_input_repeating(&self) -> bool {
        self.role.is_repeating() && !self.is_output_repeating()
    }
}

/// Extract all `arguments ... end` blocks from a function definition,
/// preserving block grouping, attributes, and declaration order.
pub(crate) fn extract_arguments_blocks_grouped<'a>(
    func_node: tree_sitter::Node<'a>,
    source: &str,
) -> Vec<ArgumentsBlockMeta<'a>> {
    let mut blocks = Vec::new();
    let mut cursor = func_node.walk();
    for child in func_node.children(&mut cursor) {
        if child.kind() != "arguments_statement" {
            continue;
        }
        let attributes = block_attributes(child, source);
        let role = block_role(&attributes);
        let args = block_arg_validations(child, source);
        blocks.push(ArgumentsBlockMeta {
            node: child,
            attributes,
            role,
            args,
            byte_range: child.start_byte()..child.end_byte(),
        });
    }
    blocks
}

/// Collect the attribute identifiers declared on an arguments block.
pub(crate) fn block_attributes(block_node: tree_sitter::Node, source: &str) -> Vec<String> {
    attributes_of(block_node, source, AttrForms::Bare)
        .into_iter()
        .map(|attr| attr.name)
        .collect()
}

/// Derive the role of an arguments block from its attribute identifiers.
pub(crate) fn block_role(attributes: &[String]) -> BlockRole {
    if attributes.iter().any(|a| a == "Repeating") {
        BlockRole::Repeating
    } else if attributes.iter().any(|a| a == "Output") {
        BlockRole::Output
    } else if attributes.iter().any(|a| a == "Input") {
        BlockRole::Input
    } else {
        BlockRole::Default
    }
}

/// Extract per-argument validation entries from an arguments block.
pub(crate) fn block_arg_validations(
    block_node: tree_sitter::Node,
    source: &str,
) -> Vec<ArgValidation> {
    let mut validations = Vec::new();
    let mut cursor = block_node.walk();
    for child in block_node.children(&mut cursor) {
        if child.kind() == "property" {
            if let Some(validation) = extract_block_arg_validation(child, source) {
                validations.push(validation);
            }
        }
    }
    validations
}

/// Extract a single argument validation entry from an arguments block property.
pub(crate) fn extract_block_arg_validation(
    prop_node: tree_sitter::Node,
    source: &str,
) -> Option<ArgValidation> {
    let name = if prop_has_ignored(prop_node) {
        "~".to_string()
    } else if prop_has_name_value(prop_node) {
        prop_name_value_text(prop_node, source)
    } else {
        prop_plain_name(prop_node, source)?
    };

    let dimensions =
        find_child_kind(prop_node, "dimensions").map(|n| node_text(n, source).to_string());

    let type_constraint = extract_block_type_constraint(prop_node, source);

    let validators = find_child_kind(prop_node, "validation_functions")
        .map(|vf| {
            let mut v = Vec::new();
            let mut vc = vf.walk();
            for vchild in vf.children(&mut vc) {
                if vchild.kind() == "identifier" {
                    v.push(node_text(vchild, source).to_string());
                }
            }
            v
        })
        .unwrap_or_default();

    let default_value = find_child_kind(prop_node, "default_value").map(|dv| {
        let text = node_text(dv, source).trim();
        let text = text.strip_prefix('=').unwrap_or(text).trim().to_string();
        text
    });

    Some(ArgValidation {
        name,
        dimensions,
        type_constraint,
        validators,
        default_value,
    })
}

/// Extract the type constraint (class name) of an arguments block property.
pub(crate) fn extract_block_type_constraint(
    prop_node: tree_sitter::Node,
    source: &str,
) -> Option<String> {
    let mut cursor = prop_node.walk();
    let mut past_name = false;
    let mut past_dims = false;
    for child in prop_node.children(&mut cursor) {
        match child.kind() {
            "identifier" | "property_name" => {
                if !past_name {
                    past_name = true;
                    continue;
                }
                if !past_dims {
                    continue;
                }
                return Some(node_text(child, source).to_string());
            }
            "ignored_argument" => {
                past_name = true;
            }
            "dimensions" => {
                past_name = true;
                past_dims = true;
            }
            "validation_functions" | "default_value" | "=" => {
                return None;
            }
            _ => {}
        }
    }
    None
}

/// Whether the property is an ignored argument (`~`).
pub(crate) fn prop_has_ignored(prop: tree_sitter::Node) -> bool {
    find_child_kind(prop, "ignored_argument").is_some()
}

/// Whether the property is a name-value argument (`struct.field`).
///
/// A declaration with a class name (`camObj pkg.Class`) also holds a `property_name`
/// node, but the grammar gives its plain name the `name` field, and a name-value
/// declaration has no such field or sets it to the `property_name` itself.
pub(crate) fn prop_has_name_value(prop: tree_sitter::Node) -> bool {
    find_child_kind(prop, "property_name").is_some()
        && prop
            .child_by_field_name("name")
            .is_none_or(|name| name.kind() == "property_name")
}

/// Whether the property is a name-value argument, including the class form `struct.?Class`.
pub(crate) fn prop_has_name_value_src(prop: tree_sitter::Node, source: &str) -> bool {
    prop_has_name_value(prop) || prop_meta_class_struct(prop, source).is_some()
}

/// The struct name of a name-value property, including the class form `struct.?Class`.
pub(crate) fn prop_name_value_struct_src(prop: tree_sitter::Node, source: &str) -> Option<String> {
    // The `?` form first: a class-typed name (`opts.?pkg.Class`) has a dotted
    // `property_name` that is not the struct.
    prop_meta_class_struct(prop, source).or_else(|| prop_name_value_struct(prop, source))
}

/// The struct name of `struct.?Class`, which the grammar does not give a `property_name`.
fn prop_meta_class_struct(prop: tree_sitter::Node, source: &str) -> Option<String> {
    let text = source.get(prop.start_byte()..prop.end_byte())?;
    let (head, rest) = text.split_once('.')?;
    let head = head.trim();
    let is_name = !head.is_empty() && head.chars().all(|c| c.is_alphanumeric() || c == '_');
    (is_name && rest.trim_start().starts_with('?')).then(|| head.to_string())
}

/// The full dotted text of a name-value property.
pub(crate) fn prop_name_value_text(prop: tree_sitter::Node, source: &str) -> String {
    find_child_kind(prop, "property_name")
        .map(|n| node_text(n, source).to_string())
        .unwrap_or_default()
}

/// The struct name of a name-value property (before the first dot).
pub(crate) fn prop_name_value_struct(prop: tree_sitter::Node, source: &str) -> Option<String> {
    let pn = find_child_kind(prop, "property_name")?;
    find_child_kind(pn, "identifier").map(|n| node_text(n, source).to_string())
}

/// The plain identifier name of a positional property.
pub(crate) fn prop_plain_name(prop: tree_sitter::Node, source: &str) -> Option<String> {
    let name = prop
        .child_by_field_name("name")
        .or_else(|| find_child_kind(prop, "identifier"))?;
    Some(node_text(name, source).to_string())
}

/// Whether the property declares a default value.
pub(crate) fn prop_has_default(prop: tree_sitter::Node) -> bool {
    find_child_kind(prop, "default_value").is_some()
}

/// Whether the property declares validation functions.
pub(crate) fn prop_has_validation(prop: tree_sitter::Node) -> bool {
    find_child_kind(prop, "validation_functions").is_some()
}

/// Collect the `property` and `class_property` children of an arguments block.
///
/// A `struct.?Class = default` line does not parse as a `class_property` (the
/// grammar has no default value for it): the grammar wraps the class property in
/// an `ERROR` node next to the block's other declarations, so those are looked
/// into too.
pub(crate) fn block_properties(block_node: tree_sitter::Node) -> Vec<tree_sitter::Node> {
    let mut props = Vec::new();
    let mut cursor = block_node.walk();
    for child in block_node.children(&mut cursor) {
        if child.kind() == "property" || child.kind() == "class_property" {
            props.push(child);
        } else if child.kind() == "ERROR" {
            let mut inner = child.walk();
            props.extend(
                child
                    .children(&mut inner)
                    .filter(|c| c.kind() == "class_property"),
            );
        }
    }
    props
}

/// Whether an arguments block assigns an attribute a value other than `true` or `false`.
pub(crate) fn attributes_have_value(block_node: tree_sitter::Node, source: &str) -> bool {
    let mut cursor = block_node.walk();
    let attrs: Vec<tree_sitter::Node> = block_node
        .children(&mut cursor)
        .filter(|child| child.kind() == "attributes")
        .collect();
    attrs.into_iter().any(|attrs| {
        attribute_entries(attrs, source, AttrForms::Written)
            .iter()
            .any(|attr| {
                attr.value
                    .as_deref()
                    .is_some_and(|v| v != "true" && v != "false")
            })
    })
}

/// A reference made inside a validation function call.
#[derive(Debug, Clone)]
enum CallReference {
    /// A plain identifier reference, or the callee of a nested call.
    Identifier(String),
    /// A dotted field reference such as `opts.Name`.
    Field(String),
    /// Any other non-literal expression, such as a metaclass operator.
    Other,
}

/// A validation function call extracted from an arguments block property.
#[derive(Debug, Clone)]
pub(crate) struct ValidatorCall {
    /// References to arguments made inside the call.
    references: Vec<CallReference>,
    /// Whether the call has no argument list, `{mustBeText()}` (VTPEAL, not FVVIN).
    pub(crate) no_arguments: bool,
}

/// Extract all validation function calls from a property.
pub(crate) fn prop_validator_calls(prop: tree_sitter::Node, source: &str) -> Vec<ValidatorCall> {
    let mut calls = Vec::new();
    let Some(vf) = find_child_kind(prop, "validation_functions") else {
        return calls;
    };
    let mut cursor = vf.walk();
    for child in vf.children(&mut cursor) {
        if child.kind() == "function_call" {
            if let Some(call) = extract_validator_call(child, source) {
                calls.push(call);
            }
        }
    }
    calls
}

/// Extract a single validation function call node.
pub(crate) fn extract_validator_call(
    call_node: tree_sitter::Node,
    source: &str,
) -> Option<ValidatorCall> {
    let mut references = Vec::new();
    collect_validator_references(call_node, source, &mut references);
    Some(ValidatorCall {
        references,
        no_arguments: crate::readability::find_arguments(call_node).is_none(),
    })
}

/// Collect the references made by the arguments of a validation call, descending into nested calls.
fn collect_validator_references(
    call_node: tree_sitter::Node,
    source: &str,
    references: &mut Vec<CallReference>,
) {
    let Some(args_node) = find_child_kind(call_node, "arguments") else {
        return;
    };
    let mut cursor = args_node.walk();
    let children: Vec<tree_sitter::Node> = args_node.children(&mut cursor).collect();
    for arg in children {
        match arg.kind() {
            "identifier" => {
                references.push(CallReference::Identifier(
                    node_text(arg, source).to_string(),
                ));
            }
            "field_expression" => {
                references.push(CallReference::Field(node_text(arg, source).to_string()));
            }
            "function_call" => {
                match find_child_kind(arg, "identifier") {
                    Some(callee) => references.push(CallReference::Identifier(
                        node_text(callee, source).to_string(),
                    )),
                    None => references.push(CallReference::Other),
                }
                collect_validator_references(arg, source, references);
            }
            "metaclass_operator" => references.push(CallReference::Other),
            _ => {}
        }
    }
}

/// Collect all `function_call` nodes within a subtree.
pub(crate) fn collect_function_calls<'a>(
    node: tree_sitter::Node<'a>,
    out: &mut Vec<tree_sitter::Node<'a>>,
) {
    if node.kind() == "function_call" {
        out.push(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_function_calls(child, out);
    }
}

/// Whether a default value expression references a name-value structure.
pub(crate) fn default_refs_name_value(
    node: tree_sitter::Node,
    source: &str,
    nv_structs: &[String],
) -> bool {
    if node.kind() == "field_expression" {
        if let Some(base) = find_child_kind(node, "identifier") {
            if nv_structs.iter().any(|s| s == node_text(base, source)) {
                return true;
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if default_refs_name_value(child, source, nv_structs) {
            return true;
        }
    }
    false
}

/// Functions that are not supported inside arguments blocks.
const BANNED_ARGUMENTS_FUNCTIONS: &[&str] = &[
    "eval",
    "evalin",
    "assignin",
    "inputname",
    "nargin",
    "nargout",
    "narginchk",
    "nargoutchk",
];

/// Built-in MATLAB classes that cannot be subclassed directly.
// MATLAB reports char, cell, struct and function_handle; it was silent on the numeric and
// logical classes in the probes.
const NON_SUBCLASSABLE_BUILTINS: &[&str] = &["char", "cell", "struct", "function_handle"];

/// Get the callee name of a `function_call` node: `f` for `f(x)` and `s.f(x)`.
pub(crate) fn callee_name(node: tree_sitter::Node, source: &str) -> Option<String> {
    crate::analysis::callee::call_identifier(node, source).map(str::to_string)
}

/// Get the named argument nodes of a `function_call` (excluding commas).
pub(crate) fn argument_nodes(node: tree_sitter::Node) -> Vec<tree_sitter::Node> {
    let mut cursor = node.walk();
    let mut result = Vec::new();
    for child in node.children(&mut cursor) {
        if child.kind() == "arguments" {
            let mut inner = child.walk();
            for arg in child.children(&mut inner) {
                if arg.is_named() && arg.kind() != "," {
                    result.push(arg);
                }
            }
            break;
        }
    }
    result
}

/// Extract the content of a string argument node (strips the surrounding quotes).
pub(crate) fn string_content(node: tree_sitter::Node, source: &str) -> Option<String> {
    if node.kind() != "string" {
        return None;
    }
    let text = node_text(node, source);
    let trimmed = text.trim();
    if trimmed.len() < 2 {
        return None;
    }
    let content = trimmed[1..trimmed.len() - 1].to_string();
    Some(content)
}

/// Get the first string argument of a `function_call` as its content.
pub(crate) fn first_string_argument(node: tree_sitter::Node, source: &str) -> Option<String> {
    argument_nodes(node)
        .first()
        .and_then(|arg| string_content(*arg, source))
}

/// Check whether any string argument of a `function_call` has the given content.
pub(crate) fn has_string_argument(node: tree_sitter::Node, source: &str, needle: &str) -> bool {
    argument_nodes(node)
        .iter()
        .filter_map(|arg| string_content(*arg, source))
        .any(|content| content.starts_with(needle))
}

/// Check whether a `function_call` is the right-hand side of an assignment.
pub(crate) fn is_assignment_rhs(node: tree_sitter::Node) -> bool {
    node.parent()
        .map(|parent| {
            parent.kind() == "assignment"
                && parent
                    .child_by_field_name("right")
                    .map(|right| right.id() == node.id())
                    .unwrap_or(false)
        })
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the source text covered by a node.
pub(crate) fn node_text<'a>(node: tree_sitter::Node, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Auto-registration
// ---------------------------------------------------------------------------

inventory::submit!(crate::RuleRegistration::new(
    "LANGUAGE_SPEC_ENGINE",
    LanguageSpecEngine::from_config
));

// ---------------------------------------------------------------------------
// Sub-check modules
// ---------------------------------------------------------------------------

mod check_ans_naming;
mod check_break_continue_outside_loop;
mod check_class_attributes;
mod check_class_file_rules;
mod check_ctoine;
mod check_ctoro;
mod check_dupnamearg;
mod check_end_operator;
pub(crate) use check_end_operator::in_control_header;
mod check_error_warning_message;
mod check_fconv;
mod check_fv_ordering;
mod check_fv_structure;
mod check_fv_validation;
mod check_fvatf;
mod check_fvbtn;
mod check_fvdan;
mod check_fvdap;
mod check_fvdnf;
mod check_fvdrep;
mod check_fvidv;
mod check_fvioa;
mod check_fvmcl;
mod check_fvnde_fvnvl;
mod check_fvniv;
mod check_fvnrep_fvrepd;
mod check_fvnsc;
mod check_fvnst;
mod check_fvobi;
mod check_fvond;
mod check_fvood_fvooi_fvoon;
mod check_fvordi;
mod check_fvordn;
mod check_fvordo;
mod check_fvordp;
mod check_fvorm;
mod check_fvovrep;
mod check_fvrepo;
mod check_fvsor;
mod check_fvsoro;
mod check_fvtinvaldim_ttoofewdims;
mod check_fvubd;
mod check_fvvin_fvocon;
mod check_fvvin_fvvcon;
mod check_fvvrep;
mod check_global_persistent_rules;
mod check_gpfst;
mod check_gpnes;
mod check_local_function_name_conflict;
mod check_mabseac;
mod check_mabseam;
mod check_matrix_rows;
mod check_mcani;
mod check_mcapp;
mod check_mcasc;
mod check_mccbd;
mod check_mccsop_check_mcscn;
mod check_mcdir;
mod check_mceb;
mod check_mcfil;
mod check_mcgsa;
mod check_mcmio;
mod check_mcmsp;
mod check_mcmtp;
mod check_mcpin;
mod check_mcpsg;
mod check_mcred;
mod check_mcsga;
mod check_mcsgp;
mod check_mcswa;
mod check_mherit;
mod check_mtags3;
mod check_mtmat;
mod check_mwkcl;
mod check_mwkct;
mod check_mwkref;
mod check_nchkos_output_use;
mod check_npers;
mod check_parfor_command;
mod check_parfor_header;
mod check_property_validation_functions;
mod check_script_rules;
mod check_setter_getter_signatures;
mod check_spmd_command;
mod check_spmd_function_call;
mod check_spmd_worker_bounds;
mod check_superclass_rules;
mod check_useswns;
mod check_vtpod;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::*;
    use malt_core::FileContext;
    use std::path::Path;
    use tree_sitter::Parser;

    /// Parse MATLAB source and return the tree.
    pub(crate) fn parse_matlab(source: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        let language = tree_sitter_matlab::LANGUAGE;
        parser
            .set_language(&language.into())
            .expect("failed to set MATLAB language");
        malt_core::note_parse();
        parser.parse(source, None).expect("failed to parse")
    }

    /// Run the language spec engine on source code with a given file path.
    pub(crate) fn check_source(source: &str, file_path: &str) -> Vec<Diagnostic> {
        check_source_with_config(&Config::default(), source, file_path)
    }

    /// Like `check_source`, with the `all` preset so checks that the `mathworks`
    /// preset leaves off (PFUTVR, VTP*) are on.
    pub(crate) fn check_source_all_presets(source: &str, file_path: &str) -> Vec<Diagnostic> {
        let config = Config::from_toml("[lint]\npreset = \"all\"\n").unwrap();
        check_source_with_config(&config, source, file_path)
    }

    fn check_source_with_config(config: &Config, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let rule = LanguageSpecEngine::from_config(config);
        let tree = parse_matlab(source);
        let path = Path::new(file_path);
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: path,
            project: None,
        };
        rule.check_file(&ctx)
    }

    /// Like `check_source`, with a search path (the path index turned on).
    pub(crate) fn check_source_with_project(
        source: &str,
        file_path: &str,
        project: &dyn malt_core::ProjectIndex,
    ) -> Vec<Diagnostic> {
        let rule = LanguageSpecEngine::from_config(&Config::default());
        let tree = parse_matlab(source);
        let path = Path::new(file_path);
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: path,
            project: Some(project),
        };
        rule.check_file(&ctx)
    }

    /// Filter diagnostics by rule_id.
    pub(crate) fn filter_by_id<'a>(diagnostics: &'a [Diagnostic], id: &str) -> Vec<&'a Diagnostic> {
        diagnostics.iter().filter(|d| d.rule_id == id).collect()
    }

    // ===== Function validation checks (FV* / TIN* / TTOO* group) =====

    const FV_CHECK_IDS: &[&str] = &[
        "FVATF",
        "FVBTN",
        "FVDAN",
        "FVDAP",
        "FVDNF",
        "FVDREP",
        "FVIDV",
        "FVIOA",
        "FVMCL",
        "FVNDE",
        "FVNIV",
        "FVNREP",
        "FVNSC",
        "FVNVL",
        "FVOBI",
        "FVOCON",
        "FVOND",
        "FVONV",
        "FVOOD",
        "FVOOI",
        "FVOON",
        "FVORDI",
        "FVORDN",
        "FVORDO",
        "FVORDP",
        "FVORM",
        "FVOVREP",
        "FVREPD",
        "FVREPO",
        "FVSOR",
        "FVSORO",
        "FVUBD",
        "FVVCON",
        "FVVIN",
        "FVVREP",
        "TINVALDIM",
        "TTOOFEWDIMS",
    ];

    #[test]
    fn test_fv_no_fire_valid_arguments_block() {
        let source = "\
function y = f(x)
    arguments
        x (1,1) double {mustBeReal}
    end
    y = x;
end
";
        let diags = check_source(source, "f.m");
        for id in FV_CHECK_IDS {
            let hits = filter_by_id(&diags, id);
            assert!(
                hits.is_empty(),
                "{id} should NOT fire on a valid arguments block, got: {}",
                hits.len()
            );
        }
    }

    #[test]
    fn test_fv_blocks_extract_roles() {
        let source = "\
function y = f(x)
    arguments (Input)
        x (1,1) double
    end
    arguments (Output)
        y (1,1) double
    end
end
";
        let tree = parse_matlab(source);
        let root = tree.root_node();
        let func_node = root.child(0).unwrap();
        let blocks = extract_arguments_blocks_grouped(func_node, source);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].role, BlockRole::Input);
        assert_eq!(blocks[0].attributes, vec!["Input".to_string()]);
        assert_eq!(blocks[1].role, BlockRole::Output);
        assert_eq!(blocks[1].attributes, vec!["Output".to_string()]);
        assert_eq!(blocks[0].args.len(), 1);
        assert_eq!(blocks[0].args[0].name, "x");
        assert_eq!(blocks[1].args[0].name, "y");
    }

    #[test]
    fn test_fv_checks_respect_disabled_config() {
        let mut rule_config = LanguageSpecConfig::default();
        rule_config.disabled_checks.push("FVNIV".to_string());
        let engine = LanguageSpecEngine {
            config: rule_config,
        };
        let source = "\
function f(a)
    arguments
        z (1,1) double
    end
end
";
        let tree = parse_matlab(source);
        let path = Path::new("f.m");
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: path,
            project: None,
        };
        let diags = engine.check_file(&ctx);
        assert!(
            filter_by_id(&diags, "FVNIV").is_empty(),
            "FVNIV should be disabled via config disabled_checks"
        );
    }

    // ===== Parfor restriction tests =====

    #[test]
    fn test_pfpf_nested_parfor() {
        let source = "\
function foo()
    parfor i = 1:10
        parfor j = 1:5
            x = i + j;
        end
    end
end
";
        let diags = check_source(source, "foo.m");
        let pfpf = filter_by_id(&diags, "PFPF");
        assert!(!pfpf.is_empty(), "PFPF should fire for nested parfor");
    }

    #[test]
    fn test_pfspmd_spmd_in_parfor() {
        let source = "\
function foo()
    parfor i = 1:10
        spmd
            x = i;
        end
    end
end
";
        let diags = check_source(source, "foo.m");
        let pfspmd = filter_by_id(&diags, "PFSPMD");
        assert!(
            !pfspmd.is_empty(),
            "PFSPMD should fire for spmd inside parfor"
        );
    }

    #[test]
    fn test_pfbrk_break_in_parfor() {
        let source = "\
function foo()
    parfor i = 1:10
        if i > 5
            break;
        end
    end
end
";
        let diags = check_source(source, "foo.m");
        let pfbrk = filter_by_id(&diags, "PFBRK");
        assert!(
            !pfbrk.is_empty(),
            "PFBRK should fire for break inside parfor"
        );
    }

    #[test]
    fn test_pfrtn_return_in_parfor() {
        let source = "\
function foo()
    parfor i = 1:10
        return;
    end
end
";
        let diags = check_source(source, "foo.m");
        let pfrtn = filter_by_id(&diags, "PFRTN");
        assert!(
            !pfrtn.is_empty(),
            "PFRTN should fire for return inside parfor"
        );
    }

    #[test]
    fn test_pfglob_global_in_parfor() {
        let source = "\
function foo()
    parfor i = 1:10
        global x;
    end
end
";
        let diags = check_source(source, "foo.m");
        let pfglob = filter_by_id(&diags, "PFGLOB");
        assert!(
            !pfglob.is_empty(),
            "PFGLOB should fire for global inside parfor"
        );
    }

    #[test]
    fn test_pfpers_persistent_in_parfor() {
        let source = "\
function foo()
    parfor i = 1:10
        persistent x;
    end
end
";
        let diags = check_source(source, "foo.m");
        let pfpers = filter_by_id(&diags, "PFPERS");
        assert!(
            !pfpers.is_empty(),
            "PFPERS should fire for persistent inside parfor"
        );
    }

    #[test]
    fn test_pfnf_nested_function_call_in_parfor() {
        let source = "\
function foo()
    parfor i = 1:10
        y = helper(i);
    end
    function z = helper(x)
        z = x + 1;
    end
end
";
        let diags = check_source(source, "foo.m");
        let pfnf = filter_by_id(&diags, "PFNF");
        assert!(
            !pfnf.is_empty(),
            "PFNF should fire for nested function call inside parfor"
        );
    }

    #[test]
    fn test_pfnf_no_fire_local_function_call() {
        let source = "\
function main()
    parfor i = 1:10
        y = helper(i);
    end
end

function z = helper(x)
    z = x + 1;
end
";
        let diags = check_source(source, "main.m");
        let pfnf = filter_by_id(&diags, "PFNF");
        assert!(
            pfnf.is_empty(),
            "PFNF should NOT fire for a top-level local function call"
        );
    }

    // ===== SPMD restriction tests =====

    #[test]
    fn test_spnst_parfor_in_spmd() {
        let source = "\
function foo()
    spmd
        parfor i = 1:10
            x = i;
        end
    end
end
";
        let diags = check_source(source, "foo.m");
        let spnst = filter_by_id(&diags, "SPNST");
        assert!(
            !spnst.is_empty(),
            "SPNST should fire for parfor inside spmd"
        );
    }

    #[test]
    fn test_spnst_spmd_in_spmd() {
        let source = "\
function foo()
    spmd
        spmd
            x = 1;
        end
    end
end
";
        let diags = check_source(source, "foo.m");
        let spnst = filter_by_id(&diags, "SPNST");
        assert!(!spnst.is_empty(), "SPNST should fire for spmd inside spmd");
    }

    #[test]
    fn test_spret_return_in_spmd() {
        let source = "\
function foo()
    spmd
        return;
    end
end
";
        let diags = check_source(source, "foo.m");
        let spret = filter_by_id(&diags, "SPRET");
        assert!(
            !spret.is_empty(),
            "SPRET should fire for return inside spmd"
        );
    }

    #[test]
    fn test_spgp_global_assignment_in_spmd() {
        let source = "\
function foo()
    global x;
    spmd
        x = 1;
    end
end
";
        let diags = check_source(source, "foo.m");
        let spgp = filter_by_id(&diags, "SPGP");
        assert!(
            !spgp.is_empty(),
            "SPGP should fire for assignment to a global inside spmd"
        );
    }

    #[test]
    fn test_spgp_persistent_assignment_in_spmd() {
        let source = "\
function foo()
    persistent p;
    spmd
        p = p + 1;
    end
end
";
        let diags = check_source(source, "foo.m");
        let spgp = filter_by_id(&diags, "SPGP");
        assert!(
            !spgp.is_empty(),
            "SPGP should fire for assignment to a persistent inside spmd"
        );
        let message = spgp[0].message.clone();
        assert!(
            message.contains('p'),
            "SPGP message should substitute the variable name, got: {message}"
        );
    }

    #[test]
    fn test_spgp_no_fire_declaration_only() {
        let source = "\
function foo()
    global x;
    spmd
        y = 1;
    end
end
";
        let diags = check_source(source, "foo.m");
        let spgp = filter_by_id(&diags, "SPGP");
        assert!(
            spgp.is_empty(),
            "SPGP should NOT fire for a declaration alone (no assignment to the global)"
        );
    }

    // ===== Class/OOP rule tests =====

    // ===== Function validation tests =====

    // ===== Script-level tests =====

    // ===== Other language spec tests =====

    #[test]
    fn test_no_parfor_violations_in_normal_code() {
        let source = "\
function foo()
    for i = 1:10
        global x;
        persistent y;
        break;
        return;
    end
end
";
        let diags = check_source(source, "foo.m");
        let pf_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.rule_id.starts_with("PF"))
            .collect();
        assert!(
            pf_diags.is_empty(),
            "No PF* violations should fire in normal for-loop code, got {:?}",
            pf_diags.iter().map(|d| d.rule_id).collect::<Vec<_>>()
        );
    }

    // ===== C4: SPMD transparency checks =====

    #[test]
    fn test_spbrk_fires_for_break_whose_loop_is_outside_spmd() {
        // Probed against R2026b: break/continue inside spmd whose loop encloses the
        // spmd (or with no loop at all) is SPBRK, not SPRET.
        let source = "\
function foo()
for i = 1:3
    spmd
        break;
    end
end
end
";
        let diags = check_source(source, "foo.m");
        assert_eq!(filter_by_id(&diags, "SPBRK").len(), 1, "got: {diags:?}");
        assert!(filter_by_id(&diags, "SPRET").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_spbrk_ok_when_loop_is_inside_spmd() {
        let source = "\
function foo()
    spmd
        for i = 1:3
            break;
        end
    end
end
";
        let diags = check_source(source, "foo.m");
        assert!(filter_by_id(&diags, "SPBRK").is_empty(), "got: {diags:?}");
        assert!(filter_by_id(&diags, "SPRET").is_empty(), "got: {diags:?}");
    }

    // ===== C4: error/warning message checks =====

    // ===== C4: class construction checks =====

    // ===== C4: script variable definition check =====

    // ===== C2 class/method attribute checks =====

    const C2_CHECK_IDS: [&str; 26] = [
        "MABSEAC", "MABSEAM", "MCAPP", "MCCBS", "MCCBU", "MCCMC", "MCCSOP", "MCGSA", "MCMIO",
        "MCMSP", "MCMTP", "MCPIN", "MCPSG", "MCSCC", "MCSCF", "MCSCM", "MCSCN", "MCSCO", "MCSCT",
        "MCSMO", "MCSWA", "MTAGS3", "MTMAT", "MWKCL", "MWKCT", "MWKREF",
    ];

    #[test]
    fn test_c2_no_fire_valid_class() {
        let source = "\
classdef Foo
    properties
        x double = 5
    end
    methods
        function y = f(self)
            y = self.x;
        end
    end
end
";
        let diags = check_source(source, "Foo.m");
        for id in C2_CHECK_IDS {
            let hits = filter_by_id(&diags, id);
            assert!(
                hits.is_empty(),
                "{id} should NOT fire on a valid class, got: {}",
                hits.len()
            );
        }
    }

    #[test]
    fn test_c2_checks_respect_disabled_config() {
        let mut rule_config = LanguageSpecConfig::default();
        rule_config.disabled_checks.push("MABSEAC".to_string());
        let engine = LanguageSpecEngine {
            config: rule_config,
        };
        let source = "\
classdef (Sealed, Abstract) Foo
    properties
        x
    end
end
";
        let tree = parse_matlab(source);
        let path = Path::new("Foo.m");
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: path,
            project: None,
        };
        let diags = engine.check_file(&ctx);
        let hits = filter_by_id(&diags, "MABSEAC");
        assert!(
            hits.is_empty(),
            "MABSEAC should be disabled via config disabled_checks"
        );
    }

    // ===== C2 constant property checks =====

    // ===== C2 WeakHandle property checks =====

    // ===== C2 constructor/superclass checks =====

    // ===== C2 method signature checks =====

    // ===== C1: parfor/SPMD variable, slicing, and reduction checks =====

    // -- PFEVC ---------------------------------------------------------------

    #[test]
    fn test_pfevc_fires_evalin_caller_in_parfor() {
        let source = "\
function f()
    parfor i = 1:10
        evalin('caller', 'x');
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFEVC").is_empty(),
            "PFEVC should fire for evalin('caller') in parfor"
        );
    }

    #[test]
    fn test_pfevc_no_fire_not_caller_or_not_parfor() {
        let source = "\
function f()
    parfor i = 1:10
        evalin('base', 'x');
    end
    for i = 1:10
        evalin('caller', 'x');
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFEVC").is_empty(),
            "PFEVC should NOT fire for evalin('base') or outside parfor"
        );
    }

    // -- PFINPT --------------------------------------------------------------

    #[test]
    fn test_pfinpt_fires_inputname_in_parfor() {
        let source = "\
function f()
    parfor i = 1:10
        inputname(1);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFINPT").is_empty(),
            "PFINPT should fire for inputname in parfor"
        );
    }

    #[test]
    fn test_pfinpt_no_fire_outside_parfor() {
        let source = "\
function f()
    for i = 1:10
        inputname(1);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFINPT").is_empty(),
            "PFINPT should NOT fire outside parfor"
        );
    }

    // -- PFNACK --------------------------------------------------------------

    #[test]
    fn test_pfnack_fires_narginchk_in_parfor() {
        let source = "\
function f()
    parfor i = 1:10
        narginchk(1, 2);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFNACK").is_empty(),
            "PFNACK should fire for narginchk in parfor"
        );
    }

    #[test]
    fn test_pfnack_no_fire_outside_parfor() {
        let source = "\
function f()
    narginchk(1, 2);
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFNACK").is_empty(),
            "PFNACK should NOT fire outside parfor"
        );
    }

    // -- PFNAIO --------------------------------------------------------------

    #[test]
    fn test_pfnaio_fires_bare_nargin_in_parfor() {
        let source = "\
function f()
    parfor i = 1:10
        nargin;
        nargout();
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFNAIO").is_empty(),
            "PFNAIO should fire for nargin/nargout without arguments in parfor"
        );
    }

    #[test]
    fn test_pfnaio_no_fire_with_argument_or_outside_parfor() {
        let source = "\
function f()
    for i = 1:10
        nargin;
    end
    nargin();
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFNAIO").is_empty(),
            "PFNAIO should NOT fire outside parfor"
        );
    }

    // -- PFLD ----------------------------------------------------------------

    #[test]
    fn test_pfld_fires_load_in_parfor() {
        let source = "\
function f()
    parfor i = 1:10
        load('data.mat');
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFLD").is_empty(),
            "PFLD should fire for load without output in parfor"
        );
    }

    #[test]
    fn test_pfld_no_fire_load_with_output() {
        let source = "\
function f()
    parfor i = 1:10
        s = load('data.mat');
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFLD").is_empty(),
            "PFLD should NOT fire when load has an output"
        );
    }

    // -- PFSV ----------------------------------------------------------------

    #[test]
    fn test_pfsv_fires_save_in_parfor() {
        let source = "\
function f()
    parfor i = 1:10
        save('out.mat');
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFSV").is_empty(),
            "PFSV should fire for save without -fromstruct in parfor"
        );
    }

    #[test]
    fn test_pfsv_no_fire_with_fromstruct() {
        let source = "\
function f()
    parfor i = 1:10
        save('-fromstruct', 's', 'out.mat');
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFSV").is_empty(),
            "PFSV should NOT fire for save with -fromstruct"
        );
    }

    // -- FPFORP / FWFORP -----------------------------------------------------

    #[test]
    fn test_fpforp_fires_fprintf_to_read_only_handle() {
        let source = "\
function f()
    parfor i = 1:10
        fid = fopen('f.txt', 'r');
        fprintf(fid, 'x');
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FPFORP").is_empty(),
            "FPFORP should fire for fprintf to a read-only handle"
        );
    }

    #[test]
    fn test_fpforp_fires_inline_fopen_read_only() {
        let source = "\
function f()
    parfor i = 1:10
        fprintf(fopen('f.txt', 'r'), 'x');
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FPFORP").is_empty(),
            "FPFORP should fire for inline read-only fopen"
        );
    }

    #[test]
    fn test_fpforp_no_fire_write_handle_or_stdout() {
        let source = "\
function f()
    parfor i = 1:10
        fid = fopen('f.txt', 'w');
        fprintf(fid, 'x');
        fprintf('hello');
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FPFORP").is_empty(),
            "FPFORP should NOT fire for a writable handle or stdout"
        );
    }

    #[test]
    fn test_fwforp_fires_fwrite_to_read_only_handle() {
        let source = "\
function f()
    parfor i = 1:10
        fid = fopen('f.txt', 'r');
        fwrite(fid, 1);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "FWFORP").is_empty(),
            "FWFORP should fire for fwrite to a read-only handle"
        );
    }

    #[test]
    fn test_fwforp_no_fire_write_handle() {
        let source = "\
function f()
    parfor i = 1:10
        fwrite(fopen('g.txt', 'w'), 1);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "FWFORP").is_empty(),
            "FWFORP should NOT fire for a writable handle"
        );
    }

    // -- PFCEL ---------------------------------------------------------------

    #[test]
    fn test_pfcel_fires_cell_array_arg() {
        let source = "\
function f()
    parfor i = 1:10
        y = sin({1, 2});
        z = cos(cell(3, 1));
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFCEL").is_empty(),
            "PFCEL should fire for cell array arguments"
        );
    }

    #[test]
    fn test_pfcel_no_fire_numeric_arg() {
        let source = "\
function f()
    parfor i = 1:10
        y = sin(x(i));
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFCEL").is_empty(),
            "PFCEL should NOT fire for numeric arguments"
        );
    }

    // -- PFANON --------------------------------------------------------------

    #[test]
    fn test_pfslrd_and_pfslo_at_first_body_mention() {
        // Probed (e_sr01, e_so05): MATLAB reports at the first mention of the variable in
        // the body, not at the later invalid access.
        let read_first = "function r = f(n)\nr = zeros(1, n);\nparfor i = 1:n\n    disp(r);\n    r(i) = 1;\nend\nend\n";
        let diags = check_source(read_first, "f.m");
        let pfslrd = filter_by_id(&diags, "PFSLRD");
        assert_eq!(pfslrd.len(), 1, "got: {diags:?}");
        assert_eq!(pfslrd[0].line, 4);
        let shifted = "function v = f(n)\nv = zeros(1, n);\nparfor i = 1:n\n    v(i) = 1;\n    x = v(i - 1);\nend\nend\n";
        let diags = check_source(shifted, "f.m");
        let pfslo = filter_by_id(&diags, "PFSLO");
        assert_eq!(pfslo.len(), 1, "got: {diags:?}");
        assert_eq!(pfslo[0].line, 4);
        // A valid sliced write alone is neither PFSLO nor PFSLRD.
        let valid =
            "function v = f(n)\nv = zeros(1, n);\nparfor i = 1:n\n    v(i) = i;\nend\nend\n";
        let diags = check_source(valid, "f.m");
        assert!(filter_by_id(&diags, "PFSLO").is_empty(), "got: {diags:?}");
        assert!(filter_by_id(&diags, "PFSLRD").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_pfunk_not_for_temp_write_on_indexed_variable() {
        // Probed (e_uk13): with a read `v(i)` in the loop, the temporary subscript is PFSLO
        // at the first mention of `v`, not PFUNK.
        let source = "function v = f(n)\nv = zeros(1, n);\nparfor i = 1:n\n    w = v(i);\n    v(w) = 1;\nend\nend\n";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "PFUNK").is_empty(), "got: {diags:?}");
        let pfslo = filter_by_id(&diags, "PFSLO");
        assert_eq!(pfslo.len(), 1, "got: {diags:?}");
        assert_eq!(pfslo[0].line, 4);
    }

    /// Probed on R2026b (v1-v8, f, h): with `x(i) = i;` on line 3, a lambda on line 4 gives
    /// PFANON at the `x` inside it for `x(i)`, PFSLO at the first mention (3:9) for `x(1)`,
    /// `x(i - 1)` and `x(t)` (a lambda parameter), and PFSLRD at the first mention for a
    /// whole read.
    #[test]
    fn test_sliced_output_in_a_lambda_follows_matlab() {
        let cases: [(&str, &str, (usize, usize)); 7] = [
            ("g = @() x(i);", "PFANON", (4, 17)),
            ("g = @(t) x(i) + t;", "PFANON", (4, 18)),
            ("g = @() x(1);", "PFSLO", (3, 9)),
            ("g = @(t) x(i - 1);", "PFSLO", (3, 9)),
            ("g = @(t) x(t);", "PFSLO", (3, 9)),
            ("g = @() x;", "PFSLRD", (3, 9)),
            ("g = @() numel(x);", "PFSLRD", (3, 9)),
        ];
        for (lambda, id, at) in cases {
            let source = format!("function f(n)\n    parfor i = 1:n\n        x(i) = i;\n        {lambda}\n    end\nend\n");
            let diags = check_source(&source, "f.m");
            let found: Vec<(usize, usize)> = filter_by_id(&diags, id)
                .iter()
                .map(|d| (d.line, d.column))
                .collect();
            assert_eq!(found, vec![at], "{lambda}: {diags:?}");
        }
        // A lambda that does not touch the sliced output reports nothing.
        let source = "function f(n)\n    c = 3;\n    parfor i = 1:n\n        g = @(t) t + c;\n        z(i) = g(i);\n    end\nend\n";
        let diags = check_source(source, "f.m");
        for id in ["PFANON", "PFSLO", "PFSLRD"] {
            assert!(filter_by_id(&diags, id).is_empty(), "{id}");
        }
    }

    #[test]
    fn test_pfanon_fires_sliced_var_in_lambda() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
        g = @(t) x(i);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFANON").is_empty(),
            "PFANON should fire when a sliced output is used in a lambda"
        );
    }

    #[test]
    fn test_pfanon_no_fire_lambda_unrelated() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
        g = @(t) t * 2;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFANON").is_empty(),
            "PFANON should NOT fire for unrelated lambdas"
        );
    }

    // -- PFANSLP / PFANSNS ---------------------------------------------------

    #[test]
    fn test_pfansns_fires_ans_for_variable_in_parfor() {
        let source = "\
function f()
    parfor i = 1:10
        for ans = 1:5
            x(i, ans) = i;
        end
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFANSNS").is_empty(),
            "PFANSNS should fire for 'ans' as for variable in parfor"
        );
    }

    #[test]
    fn test_pfansns_no_fire_normal_for_variable() {
        let source = "\
function f()
    parfor i = 1:10
        for j = 1:5
            x(i, j) = i;
        end
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFANSNS").is_empty(),
            "PFANSNS should NOT fire for a normal for variable"
        );
    }

    // -- PFCTXT --------------------------------------------------------------

    #[test]
    fn test_pfctxt_fires_sliced_access_outside_nested_loop() {
        let source = "\
function f()
    parfor i = 1:10
        for j = 1:5
            x(i, j) = i * j;
        end
        y(i) = x(i, j);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFCTXT").is_empty(),
            "PFCTXT should fire when a sliced variable is indexed outside the defining for loop"
        );
    }

    #[test]
    fn test_pfctxt_no_fire_access_inside_nested_loop() {
        let source = "\
function f()
    parfor i = 1:10
        for j = 1:5
            x(i, j) = i * j;
        end
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFCTXT").is_empty(),
            "PFCTXT should NOT fire when accesses are inside the defining for loop"
        );
    }

    // -- PFFRNG --------------------------------------------------------------

    #[test]
    fn test_pffrng_fires_non_constant_nested_range() {
        let source = "\
function f()
    parfor i = 1:10
        for j = 1:i
            x(i, j) = i * j;
        end
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFFRNG").is_empty(),
            "PFFRNG should fire for a non-constant nested loop range"
        );
    }

    #[test]
    fn test_pffrng_no_fire_constant_positive_range() {
        let source = "\
function f()
    parfor i = 1:10
        for j = 1:5
            x(i, j) = i * j;
        end
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFFRNG").is_empty(),
            "PFFRNG should NOT fire for a constant positive range"
        );
    }

    // -- PFFSUB --------------------------------------------------------------

    #[test]
    fn test_pffsub_fires_indexing_nested_for_variable() {
        let source = "\
function f()
    parfor i = 1:10
        for j = 1:5
            y = j(k);
        end
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFFSUB").is_empty(),
            "PFFSUB should fire when a nested for variable is indexed"
        );
    }

    #[test]
    fn test_pffsub_no_fire_nested_for_as_argument() {
        let source = "\
function f()
    parfor i = 1:10
        for j = 1:5
            x(i, j) = i * j;
        end
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFFSUB").is_empty(),
            "PFFSUB should NOT fire when the nested for variable is an argument"
        );
    }

    // -- PFINCR --------------------------------------------------------------

    #[test]
    fn test_pfincr_fires_mixed_reduction_operators() {
        let source = "\
function f(s)
    parfor i = 1:10
        s = s + x(i);
        s = s * y(i);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFINCR").is_empty(),
            "PFINCR should fire for mixed reduction operators"
        );
    }

    #[test]
    fn test_pfincr_no_fire_single_reduction_operator() {
        let source = "\
function f(s)
    parfor i = 1:10
        s = s + x(i);
        s = s + y(i);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFINCR").is_empty(),
            "PFINCR should NOT fire for a single reduction operator"
        );
    }

    // -- PFMLTI --------------------------------------------------------------

    #[test]
    fn test_pfmlti_fires_assigning_nested_for_variable() {
        let source = "\
function f()
    parfor i = 1:10
        for j = 1:5
            j = i;
        end
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFMLTI").is_empty(),
            "PFMLTI should fire when the nested for variable is assigned"
        );
    }

    #[test]
    fn test_pfmlti_no_fire_no_assignment_to_loop_variable() {
        let source = "\
function f()
    parfor i = 1:10
        for j = 1:5
            x(i, j) = i * j;
        end
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFMLTI").is_empty(),
            "PFMLTI should NOT fire when the nested for variable is not assigned"
        );
    }

    // -- PFNAR ---------------------------------------------------------------

    #[test]
    fn test_pfnar_fires_subtracting_reduction_variable() {
        let source = "\
function s = f(x)
    s = 0;
    parfor i = 1:10
        s = x(i) - s;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFNAR").is_empty(),
            "PFNAR should fire when a reduction variable is subtracted from an expression"
        );
    }

    #[test]
    fn test_pfnar_no_fire_valid_reduction_subtraction() {
        let source = "\
function s = f(x)
    s = 0;
    parfor i = 1:10
        s = s - x(i);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFNAR").is_empty(),
            "PFNAR should NOT fire for a valid reduction subtraction"
        );
    }

    // -- PFRFH ---------------------------------------------------------------

    #[test]
    fn test_pfrfh_fires_unknown_reduction_function() {
        let source = "\
function s = f(x)
    s = 0;
    parfor i = 1:10
        fn = i;
        s = fn(s, x(i));
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFRFH").is_empty(),
            "PFRFH should fire when the reduction function is not a name or broadcast variable"
        );
    }

    #[test]
    fn test_pfrfh_no_fire_known_reduction_function() {
        let source = "\
function s = f(x)
    s = 0;
    parfor i = 1:10
        s = plus(s, x(i));
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFRFH").is_empty(),
            "PFRFH should NOT fire for a known reduction function"
        );
    }

    #[test]
    fn test_pfrfh_no_fire_broadcast_reduction_function() {
        let source = "\
function s = f(x, fn)
    s = 0;
    parfor i = 1:10
        s = fn(s, x(i));
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFRFH").is_empty(),
            "PFRFH should NOT fire for a broadcast reduction function"
        );
    }

    // -- PFRNG ---------------------------------------------------------------

    // -- Parfor / SPMD probe regressions (MATLAB R2026b checkcode) ----------

    #[test]
    fn test_pfslw_fires_on_last_subscript_difference() {
        // Probed (pfslw_06): same subscript count, only the second subscript differs.
        let source = "function f()\n    a = zeros(10, 2);\n    parfor i = 1:10\n        a(i, 1) = 1;\n        a(i, 2) = 2;\n    end\nend\n";
        let diags = check_source(source, "f.m");
        let pfslw = filter_by_id(&diags, "PFSLW");
        assert_eq!(pfslw.len(), 1, "got: {diags:?}");
        assert_eq!(pfslw[0].line, 4);
        assert!(filter_by_id(&diags, "PFSLO").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_pfslw_silent_on_repeated_identical_subscripts() {
        // Probed (pfslw_04): identical subscript lists are consistent.
        let source = "function f()\n    a = zeros(10, 1);\n    parfor i = 1:10\n        a(i) = 1;\n        a(i) = 2;\n    end\nend\n";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "PFSLW").is_empty(), "got: {diags:?}");
        assert!(filter_by_id(&diags, "PFSLO").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_pfbrk_reports_first_break_of_parfor_only() {
        // Probed (pfbrk_13, pfbrk_20): one PFBRK per parfor loop, at its first break.
        let source = "function f()\n    parfor i = 1:10\n        if i > 2\n            break\n        end\n        if i > 3\n            break\n        end\n    end\nend\n";
        let diags = check_source(source, "f.m");
        let pfbrk = filter_by_id(&diags, "PFBRK");
        assert_eq!(pfbrk.len(), 1, "got: {diags:?}");
        assert_eq!(pfbrk[0].line, 4);
    }

    #[test]
    fn test_pfbrk_silent_for_break_in_nested_loop_or_switch() {
        // Probed (pfbrk_16, pfbrk_18, pfbrk_21, pfbrk_22): breaks owned by a nested loop
        // or a switch are not parfor breaks.
        let source = "function f()\n    parfor i = 1:10\n        for j = 1:3\n            break\n        end\n        switch i\n            case 2\n                break\n        end\n    end\nend\n";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "PFBRK").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_pfxst_fires_on_indexed_assignment_to_loop_variable() {
        // Probed (pfxst_03): `i(1) = 2` assigns the loop variable, reported at `i`.
        let source = "function f()\n    parfor i = 1:10\n        i(1) = 2;\n    end\nend\n";
        let diags = check_source(source, "f.m");
        let pfxst = filter_by_id(&diags, "PFXST");
        assert_eq!(pfxst.len(), 1, "got: {diags:?}");
        assert_eq!((pfxst[0].line, pfxst[0].column), (3, 9));
    }

    #[test]
    fn test_pfxst_silent_on_indexed_write_with_loop_variable_subscript() {
        // Probed (pfous_03): the loop variable as a subscript is not an assignment to it.
        let source = "function f()\n    parfor i = 1:10\n        x(i) = 1;\n    end\nend\n";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "PFXST").is_empty(), "got: {diags:?}");
    }

    #[test]
    fn test_pfincr_reports_at_first_differing_operation() {
        // Probed (pfincr_01): reported at the statement whose operator differs from the first.
        let source = "function f()\n    s = 0;\n    parfor i = 1:10\n        s = s + i;\n        s = s - 1;\n    end\nend\n";
        let diags = check_source(source, "f.m");
        let pfincr = filter_by_id(&diags, "PFINCR");
        assert_eq!(pfincr.len(), 1, "got: {diags:?}");
        assert_eq!(pfincr[0].line, 5);
    }

    #[test]
    fn test_spgp_fires_on_field_store_to_global() {
        // Probed (spgp_02): `G.a = 1` in spmd sets the global G on the worker.
        let source = "function f()\n    global G\n    spmd\n        G.a = 1;\n    end\nend\n";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "SPGP").len(), 1, "got: {diags:?}");
    }

    #[test]
    fn test_spbfn_fires_on_evalc_and_clear_command_in_spmd() {
        // Probed (spbfn_02, spbfn_03): evalc and `clear x` are workspace-non-transparent.
        let source =
            "function f()\n    spmd\n        s = evalc('1');\n        clear x\n    end\nend\n";
        let diags = check_source(source, "f.m");
        assert_eq!(filter_by_id(&diags, "SPBFN").len(), 2, "got: {diags:?}");
    }

    // -- PFSLO ---------------------------------------------------------------

    #[test]
    fn test_pfslo_silent_temp_indexed_with_loop_variable() {
        let source = "\
function f()
    parfor i = 1:10
        t = i * 2;
        y = t(i);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFSLO").is_empty(),
            "PFSLO should NOT fire for a temporary indexed with the loop variable (probed: pfbns_17)"
        );
    }

    #[test]
    fn test_pfslo_no_fire_valid_sliced_output() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
        y = x(i);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFSLO").is_empty(),
            "PFSLO should NOT fire for a valid sliced output"
        );
    }

    // -- PFSLRD --------------------------------------------------------------

    #[test]
    fn test_pfslrd_fires_whole_read_of_sliced_variable() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
        y = x;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFSLRD").is_empty(),
            "PFSLRD should fire for a non-indexed read of a sliced variable"
        );
    }

    #[test]
    fn test_pfslrd_no_fire_indexed_read_only() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
        y = x(i);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFSLRD").is_empty(),
            "PFSLRD should NOT fire for indexed reads only"
        );
    }

    // -- PFSLW ---------------------------------------------------------------

    #[test]
    fn test_pfslo_fires_on_subscript_count_mismatch() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
        y = x(i, 1);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFSLO").is_empty(),
            "PFSLO should fire for a subscript count mismatch (probed: pfslw_01)"
        );
    }

    #[test]
    fn test_pfslw_no_fire_same_subscripts() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
        y = x(i);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFSLW").is_empty(),
            "PFSLW should NOT fire for consistent subscript lists"
        );
    }

    // -- PFUNK ---------------------------------------------------------------

    #[test]
    fn test_pfunk_silent_sliced_then_whole_assignment() {
        // Probed (pfunk_01): MATLAB reports nothing for a sliced variable that is then
        // assigned as a whole.
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
        x = 5;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFUNK").is_empty(),
            "PFUNK should NOT fire for a sliced variable assigned as a whole"
        );
    }

    #[test]
    fn test_pfunk_no_fire_pure_sliced() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFUNK").is_empty(),
            "PFUNK should NOT fire for a pure sliced output"
        );
    }

    // -- PFUTMP --------------------------------------------------------------

    #[test]
    fn test_pfutmp_fires_temp_used_before_set() {
        let source = "\
function f()
    parfor i = 1:10
        y = t + i;
        t = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFUTMP").is_empty(),
            "PFUTMP should fire when a temporary is used before it is set"
        );
    }

    #[test]
    fn test_pfunk_fires_on_sliced_write_through_temporary() {
        // Probed (pu01, u20, u32): a sliced write whose subscript is a body temporary is
        // PFUNK, whatever the temporary holds, and the write is reported.
        for body in [
            "k = i;\n        v(k) = 1;",
            "k = 2;\n        v(k) = i;",
            "k = i;\n        v(k) = 1;\n        k = 2;",
        ] {
            let source = format!("function v = f(n)\n    v = zeros(1, n);\n    parfor i = 1:n\n        {body}\n    end\nend\n");
            let diags = check_source(&source, "f.m");
            assert_eq!(filter_by_id(&diags, "PFUNK").len(), 1, "{body}");
        }
        // Probed: a sliced write by the loop index itself is not PFUNK.
        let source = "function v = f(n)\n    v = zeros(1, n);\n    parfor i = 1:n\n        k = i;\n        v(i) = k;\n    end\nend\n";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "PFUNK").is_empty());
    }

    #[test]
    fn test_pfutmp_no_fire_read_after_separator_on_same_line() {
        // Probed (q08): `x = data(1:3); r(i) = x(1);` on one line is set before use.
        let source = "\
function f(data)
    parfor i = 1:10
        x = data(1:3); r(i) = x(1);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFUTMP").is_empty(),
            "PFUTMP should NOT fire when the read follows a separator"
        );
    }

    #[test]
    fn test_pfutmp_no_fire_set_before_use() {
        let source = "\
function f()
    parfor i = 1:10
        t = i;
        y = t + 1;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFUTMP").is_empty(),
            "PFUTMP should NOT fire when the temporary is set before use"
        );
    }

    #[test]
    fn test_pfutmp_no_fire_predefined_variable() {
        let source = "\
function f(t)
    parfor i = 1:10
        y = t + i;
        t = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFUTMP").is_empty(),
            "PFUTMP should NOT fire for a function input"
        );
    }

    // -- PFUTVR --------------------------------------------------------------

    #[test]
    fn test_putvr_fires_uninitialized_reduction() {
        let source = "parfor i = 1:10\n    s = s + x(i);\nend\n";
        let diags = check_source_all_presets(source, "script.m");
        assert!(
            !filter_by_id(&diags, "PFUTVR").is_empty(),
            "PFUTVR should fire for an uninitialized reduction variable"
        );
    }

    #[test]
    fn test_pfutvr_no_fire_initialized_reduction() {
        let source = "\
function s = f(x)
    s = 0;
    parfor i = 1:10
        s = s + x(i);
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFUTVR").is_empty(),
            "PFUTVR should NOT fire for an initialized reduction variable"
        );
    }

    // -- PFVARS --------------------------------------------------------------

    #[test]
    fn test_pfvars_fires_too_many_variables() {
        let mut body = String::new();
        for k in 0..1100 {
            body.push_str(&format!("v{k} = i;\n"));
        }
        let source = format!("function f()\nparfor i = 1:10\n{body}end\nend\n");
        let diags = check_source(&source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFVARS").is_empty(),
            "PFVARS should fire for more than 1024 variables"
        );
    }

    #[test]
    fn test_pfvars_no_fire_few_variables() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFVARS").is_empty(),
            "PFVARS should NOT fire for a few variables"
        );
    }

    // -- PFVSUB --------------------------------------------------------------

    #[test]
    fn test_pfvsub_fires_indexing_parfor_variable() {
        let source = "\
function f()
    parfor i = 1:10
        for j = 1:5
            y = i(j);
        end
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFVSUB").is_empty(),
            "PFVSUB should fire when the parfor loop variable is indexed"
        );
    }

    #[test]
    fn test_pfvsub_no_fire_sliced_indexing() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            filter_by_id(&diags, "PFVSUB").is_empty(),
            "PFVSUB should NOT fire for indexing a sliced variable"
        );
    }

    // -- plan MUST-fire / MUST-NOT-fire examples -----------------------------

    #[test]
    fn test_c1_plan_must_fire_example() {
        let source = "\
function f()
    parfor i = 1:10
        evalin('caller', 'x');
        x(i) = i;
    end
    parfor ans = 1:10
        y = ans;
    end
end
";
        let diags = check_source(source, "f.m");
        assert!(
            !filter_by_id(&diags, "PFEVC").is_empty(),
            "PFEVC should fire on the MUST-fire example"
        );
        assert!(
            !filter_by_id(&diags, "PFANSLP").is_empty(),
            "PFANSLP should fire on the MUST-fire example"
        );
    }

    #[test]
    fn test_c1_plan_must_not_fire_example() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
    end
    for i = 1:10
        evalin('base', 'x');
    end
end
";
        let diags = check_source(source, "f.m");
        let c1_diags: Vec<_> = diags
            .iter()
            .filter(|d| {
                matches!(
                    d.rule_id,
                    "FPFORP"
                        | "FWFORP"
                        | "PFANON"
                        | "PFANSLP"
                        | "PFANSNS"
                        | "PFCEL"
                        | "PFCTXT"
                        | "PFEVC"
                        | "PFFRNG"
                        | "PFFSUB"
                        | "PFINCR"
                        | "PFINPT"
                        | "PFLD"
                        | "PFMLTI"
                        | "PFNACK"
                        | "PFNAIO"
                        | "PFNAR"
                        | "PFRFH"
                        | "PFRNG"
                        | "PFSLO"
                        | "PFSLRD"
                        | "PFSLW"
                        | "PFSV"
                        | "PFUNK"
                        | "PFUTMP"
                        | "PFUTVR"
                        | "PFVARS"
                        | "PFVSUB"
                )
            })
            .collect();
        assert!(
            c1_diags.is_empty(),
            "No C1 parfor checks should fire on the MUST-NOT-fire example, got {:?}",
            c1_diags.iter().map(|d| d.rule_id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_c1_checks_respect_disabled_config() {
        let mut rule_config = LanguageSpecConfig::default();
        rule_config.disabled_checks.push("PFEVC".to_string());
        rule_config.disabled_checks.push("PFANSLP".to_string());
        let engine = LanguageSpecEngine {
            config: rule_config,
        };
        let source = "\
function f()
    parfor i = 1:10
        evalin('caller', 'x');
        x(i) = i;
    end
    parfor ans = 1:10
        y = ans;
    end
end
";
        let tree = parse_matlab(source);
        let path = Path::new("f.m");
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: path,
            project: None,
        };
        let diags = engine.check_file(&ctx);
        assert!(
            filter_by_id(&diags, "PFEVC").is_empty(),
            "PFEVC should be disabled via config disabled_checks"
        );
        assert!(
            filter_by_id(&diags, "PFANSLP").is_empty(),
            "PFANSLP should be disabled via config disabled_checks"
        );
    }
}

#[cfg(test)]
mod r8_recovery_tests {
    use crate::language_spec::tests::{check_source, filter_by_id};

    #[test]
    fn pfutmp_fires_for_indexed_read_before_whole_set() {
        let source = "function p1(n)\nparfor i = 1:n\n  k = k(i) + 1;\nend\nend\n";
        let diags = check_source(source, "p1.m");
        assert!(!filter_by_id(&diags, "PFUTMP").is_empty());
        assert!(filter_by_id(&diags, "PFSLO").is_empty());
    }

    #[test]
    fn pfutmp_does_not_fire_for_sliced_output() {
        let source = "function p2(n)\nk = zeros(n,1);\nparfor i = 1:n\n  k(i) = i;\nend\nend\n";
        let diags = check_source(source, "p2.m");
        assert!(filter_by_id(&diags, "PFUTMP").is_empty());
    }

    #[test]
    fn fvdan_and_fvordn_treat_struct_dot_question_as_name_value() {
        let source = "function f(a, opts)\n    arguments\n        a\n        opts.alpha = 1\n        opts.?double\n    end\n    r = 1;\nend\n";
        let diags = check_source(source, "f.m");
        assert!(filter_by_id(&diags, "FVDAN").is_empty());
        assert!(filter_by_id(&diags, "FVORDN").is_empty());
    }

    #[test]
    fn test_pfutmp_fires_on_indexed_read_before_set() {
        // Probed (xcv23, xcv24): `x = u(1)` reads `u` before its first set in the body.
        let source = "function f(n)\nparfor k = 1:n\n    x = u(1);\n    u = k;\nend\nend\n";
        assert!(!filter_by_id(&check_source(source, "f.m"), "PFUTMP").is_empty());
    }

    #[test]
    fn test_pfutmp_fires_when_variable_was_set_before_loop() {
        // Probed (xcv10, xcp39): a value set before the loop does not make the read a set.
        let source = "function f(n)\nu = 0;\nparfor k = 1:n\n    w = u;\n    u = k;\nend\nend\n";
        assert!(!filter_by_id(&check_source(source, "f.m"), "PFUTMP").is_empty());
    }

    #[test]
    fn test_pfutmp_silent_on_conditional_read_or_self_update() {
        // Probed (xcv22, xcv20): a read under `if` is not reported; `u = u + 1` in the body makes
        // the variable PFUTVR instead.
        let source =
            "function f(n)\nparfor k = 1:n\n    if k > 1, w = u; end\n    u = k;\nend\nend\n";
        assert!(filter_by_id(&check_source(source, "f.m"), "PFUTMP").is_empty());
        let source =
            "function f(n)\nparfor k = 1:n\n    w = u;\n    u = k;\n    u = u + 1;\nend\nend\n";
        assert!(filter_by_id(&check_source(source, "f.m"), "PFUTMP").is_empty());
    }

    #[test]
    fn test_pfunk_fires_on_sliced_write_by_inner_loop_variable() {
        // Probed (xcu01, xcu02, xcu08): `y(j) = i` in a nested `for j` is not classified, with or
        // without an earlier value of `y`.
        let source = "function f(n)\ny = zeros(1, 3);\nparfor i = 1:n\n    for j = 1:3\n        y(j) = i;\n    end\nend\nend\n";
        assert!(!filter_by_id(&check_source(source, "f.m"), "PFUNK").is_empty());
        let source = "function f(n)\nparfor i = 1:n\n    for j = 1:3\n        y(j) = 1;\n    end\nend\nend\n";
        assert!(!filter_by_id(&check_source(source, "f.m"), "PFUNK").is_empty());
    }

    #[test]
    fn test_pfunk_silent_when_loop_variable_also_indexes_or_body_assigns() {
        // Probed (xcu14, xcp29): `y(i) = 1` classifies `y`; a whole assignment of `y` in the body
        // does too.
        let source = "function f(n)\ny = zeros(1, 3);\nparfor i = 1:n\n    for j = 1:3\n        y(j) = i;\n        y(i) = 1;\n    end\nend\nend\n";
        assert!(filter_by_id(&check_source(source, "f.m"), "PFUNK").is_empty());
        let source = "function f(n)\nparfor i = 1:n\n    y = zeros(1, 3);\n    for j = 1:3\n        y(j) = i;\n    end\nend\nend\n";
        assert!(filter_by_id(&check_source(source, "f.m"), "PFUNK").is_empty());
    }
}
