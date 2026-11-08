//! # LANGUAGE_SPEC_ENGINE: Language Specification Errors
//!
//! ```malt
//! id = "LANGUAGE_SPEC_ENGINE"
//! title = "Language Specification Errors"
//! category = "language-specification"
//! severity = "error"
//! fix = false
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
//! the specific check ID (e.g. `PFPF`, `FVAPN`, `MCFIL`).
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
//! | SPGP | error | no | Setting the GLOBAL or PERSISTENT variable VAR_NAME in an SPMD block might fail because the set happens on a worker machine. |
//! | MCFIL | error | no | Class name VAR_NAME and file name do not agree: VAR_FILE. Update the class name and constructor, if defined, or change the file name to match the class name. |
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
//! | FVAPN | error | no | Move name-value arguments that use the name=value syntax to the end of the argument list. |
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
    ArgValidation, AttributeMeta, ClassMeta, FileMeta, FileType, FunctionMeta,
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
    lambdas: Vec<(usize, Vec<String>)>,
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
        let rule_config: LanguageSpecConfig = config.rule_params("LANGUAGE_SPEC_ENGINE");
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
        let symbol_table = SymbolTable::build(ctx.tree, ctx.source);

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
                // SPBRK (break/continue whose loop is not fully contained in the spmd
                // block) is subsumed by SPRET, which fires for every break/continue
                // inside an spmd block.
                // Check: PFBRK — break inside parfor
                if current_context == LoopContext::Parfor {
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
                // Check: SPRET — break inside spmd
                if current_context == LoopContext::Spmd {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        rule_id: "SPRET",
                        message: "break statement cannot be used inside an SPMD block.".to_string(),
                        severity: Severity::Error,
                        byte_range: node.start_byte()..node.end_byte(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }

            "continue_statement" if current_context == LoopContext::Spmd => {
                // Check: SPRET — continue inside spmd
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    rule_id: "SPRET",
                    message: "continue statement cannot be used inside an SPMD block.".to_string(),
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
                self.visit_children(node, source, context_stack, nested_functions, diagnostics);
                return;
            }

            "identifier" if current_context == LoopContext::Parfor => {
                // C1 checks: record whole (non-indexed) variable reads
                self.check_parfor_identifier(node, source, context_stack);
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
                if let Some(name) = self.assignment_lhs_var_name(node, source) {
                    if self.is_global_or_persistent_in_scope(symbol_table, node.start_byte(), &name)
                    {
                        self.push_diag(
                            node,
                            "SPGP",
                            &format!(
                                "Setting the GLOBAL or PERSISTENT variable {name} in an SPMD block might fail because the set happens on a worker machine."
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

    /// Extract the variable name assigned to, handling both simple (`x = ...`)
    /// and indexed (`x(i) = ...`) left-hand sides.
    fn assignment_lhs_var_name(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        let lhs = node.child_by_field_name("left")?;
        match lhs.kind() {
            "identifier" => Some(node_text(lhs, source).to_string()),
            "function_call" => callee_name(lhs, source),
            _ => None,
        }
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
                if !is_assignment_rhs(node) {
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
            analysis
                .lambdas
                .push((node.start_byte(), refs.into_iter().collect()));
        }
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
        for (var, ops) in &analysis.reductions {
            let mut fns: Vec<String> = ops.iter().map(|(op, _)| op.clone()).collect();
            if let Some(frs) = analysis.func_reductions.get(var) {
                fns.extend(frs.iter().map(|(c, _)| c.clone()));
            }
            fns.sort();
            fns.dedup();
            if fns.len() > 1 {
                let start = ops.iter().map(|(_, s)| *s).min().unwrap_or(0);
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
        for (var, set_starts) in &analysis.temp_assigns {
            if predefined.contains(var) {
                continue;
            }
            let first_set = set_starts.iter().min().copied();
            let mut read_starts: Vec<usize> = Vec::new();
            if let Some(rs) = analysis.whole_reads.get(var) {
                read_starts.extend(rs.iter().copied());
            }
            if let Some(rs) = analysis.indexed_reads.get(var) {
                read_starts.extend(rs.iter().map(|(_, s, _, _)| *s));
            }
            if let (Some(fs), Some(fr)) = (first_set, read_starts.into_iter().min()) {
                if fr < fs {
                    self.push_diag_bytes(
                        source,
                        fr,
                        fr + 1,
                        "PFUTMP",
                        &format!("Temporary variable {var} must be set inside the parfor loop before it is used."),
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

        // PFSLO: indexed with the parfor loop variable but not a sliced output
        for (var, reads) in &analysis.indexed_reads {
            if analysis.temp_assigns.contains_key(var) && !analysis.sliced_lhs.contains_key(var) {
                if let Some(first) = reads
                    .iter()
                    .filter(|(args, _, _, _)| args_tokens(args).contains(&parfor_var))
                    .map(|(_, s, _, _)| *s)
                    .min()
                {
                    self.push_diag_bytes(
                        source,
                        first,
                        first + 1,
                        "PFSLO",
                        &format!("Variable {var} is indexed using the parfor loop variable, but it is not a valid sliced output variable."),
                        diagnostics,
                    );
                }
            }
        }

        // PFSLRD: sliced output variable read without slicing
        for var in analysis.sliced_lhs.keys() {
            if let Some(wrs) = analysis.whole_reads.get(var) {
                if let Some(first) = wrs.iter().min().copied() {
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

        // PFSLW: multiple sliced accesses with different subscripts
        for (var, writes) in &analysis.sliced_lhs {
            let mut sigs: Vec<String> = writes.iter().map(|(args, _, _, _)| args.clone()).collect();
            if let Some(reads) = analysis.indexed_reads.get(var) {
                sigs.extend(reads.iter().map(|(args, _, _, _)| args.clone()));
            }
            sigs.sort();
            sigs.dedup();
            if sigs.len() > 1 {
                let first = writes.iter().map(|(_, s, _, _)| *s).min().unwrap_or(0);
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

        // PFUNK: variable used both as a sliced output and as a temporary
        for var in analysis.sliced_lhs.keys() {
            if let Some(temps) = analysis.temp_assigns.get(var) {
                let start = temps.iter().min().copied().unwrap_or(0);
                self.push_diag_bytes(
                    source,
                    start,
                    start + 1,
                    "PFUNK",
                    &format!("Unable to classify variable {var} in the body of the parfor loop."),
                    diagnostics,
                );
            }
        }

        // PFANON: sliced output variable used inside an anonymous function
        for (start, refs) in &analysis.lambdas {
            if refs.iter().any(|r| analysis.sliced_lhs.contains_key(r)) {
                self.push_diag_bytes(
                    source,
                    *start,
                    *start + 1,
                    "PFANON",
                    "Using a sliced output variable in an anonymous function is not \
                     supported in parfor loops.",
                    diagnostics,
                );
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
        self.check_mceb(class, diagnostics);
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
        self.check_mcscn(class, ctx, diagnostics);
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
}

impl LanguageSpecEngine {
    /// Whether a property is a positional input argument (not ignored, not a
    /// name-value argument, and not `varargin`).
    pub(crate) fn fv_is_positional_prop(&self, prop: tree_sitter::Node, source: &str) -> bool {
        !prop_has_ignored(prop)
            && !prop_has_name_value(prop)
            && prop_plain_name(prop, source)
                .map(|n| n != "varargin")
                .unwrap_or(false)
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
    let mut attrs = Vec::new();
    let mut cursor = block_node.walk();
    for child in block_node.children(&mut cursor) {
        if child.kind() == "attributes" {
            let mut inner = child.walk();
            for attr in child.children(&mut inner) {
                if attr.kind() == "identifier" {
                    attrs.push(node_text(attr, source).to_string());
                }
            }
            break;
        }
    }
    attrs
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
pub(crate) fn prop_has_name_value(prop: tree_sitter::Node) -> bool {
    find_child_kind(prop, "property_name").is_some()
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

/// The field name of a name-value property (after the last dot).
pub(crate) fn prop_name_value_field(prop: tree_sitter::Node, source: &str) -> Option<String> {
    let pn = find_child_kind(prop, "property_name")?;
    let mut last = None;
    let mut cursor = pn.walk();
    for child in pn.children(&mut cursor) {
        if child.kind() == "identifier" {
            last = Some(node_text(child, source).to_string());
        }
    }
    last
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

/// Whether the property declares a size constraint.
pub(crate) fn prop_has_dimensions(prop: tree_sitter::Node) -> bool {
    find_child_kind(prop, "dimensions").is_some()
}

/// Collect the `property` and `class_property` children of an arguments block.
pub(crate) fn block_properties(block_node: tree_sitter::Node) -> Vec<tree_sitter::Node> {
    let mut props = Vec::new();
    let mut cursor = block_node.walk();
    for child in block_node.children(&mut cursor) {
        if child.kind() == "property" || child.kind() == "class_property" {
            props.push(child);
        }
    }
    props
}

/// Whether an arguments block declares any attribute value (must be logical constants).
pub(crate) fn attributes_have_value(block_node: tree_sitter::Node) -> bool {
    let mut cursor = block_node.walk();
    for child in block_node.children(&mut cursor) {
        if child.kind() != "attributes" {
            continue;
        }
        let mut inner = child.walk();
        for attr in child.children(&mut inner) {
            if attr.kind() == "=" || attr.is_error() || attr.is_missing() {
                return true;
            }
        }
        return false;
    }
    false
}

/// A reference made inside a validation function call.
#[derive(Debug, Clone)]
enum CallReference {
    /// A plain identifier reference.
    Identifier(String),
    /// A dotted field reference such as `opts.Name`.
    Field,
}

/// A validation function call extracted from an arguments block property.
#[derive(Debug, Clone)]
pub(crate) struct ValidatorCall {
    /// References to arguments made inside the call.
    references: Vec<CallReference>,
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
    if let Some(args_node) = find_child_kind(call_node, "arguments") {
        let mut cursor = args_node.walk();
        for arg in args_node.children(&mut cursor) {
            match arg.kind() {
                "identifier" => {
                    references.push(CallReference::Identifier(
                        node_text(arg, source).to_string(),
                    ));
                }
                "field_expression" => {
                    references.push(CallReference::Field);
                }
                _ => {}
            }
        }
    }
    Some(ValidatorCall { references })
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

/// Build the declaration sequence of an arguments block group.
///
/// Consecutive name-value struct names are deduplicated: multiple fields of
/// the same structure map to a single function-line argument.
pub(crate) fn block_declaration_sequence(
    blocks: &[ArgumentsBlockMeta],
    source: &str,
    outputs: bool,
) -> Vec<String> {
    let mut sequence = Vec::new();
    for block in blocks {
        let relevant = if outputs {
            block.role.is_output() || block.is_output_repeating()
        } else {
            block.role.is_input() || block.is_input_repeating()
        };
        if !relevant {
            continue;
        }
        for prop in block_properties(block.node) {
            let name = if prop_has_ignored(prop) {
                Some("~".to_string())
            } else if prop_has_name_value(prop) {
                prop_name_value_struct(prop, source)
            } else {
                prop_plain_name(prop, source)
            };
            let Some(name) = name else {
                continue;
            };
            if sequence.last() == Some(&name) {
                continue;
            }
            sequence.push(name);
        }
    }
    sequence
}

/// Whether `sequence` is a prefix of `full`.
pub(crate) fn is_prefix_of(sequence: &[String], full: &[String]) -> bool {
    sequence.len() <= full.len() && sequence.iter().zip(full.iter()).all(|(a, b)| a == b)
}

/// Whether a name is a known MATLAB class name.
pub(crate) fn is_known_class_name(name: &str) -> bool {
    matches!(
        name,
        "double"
            | "single"
            | "logical"
            | "char"
            | "string"
            | "cell"
            | "struct"
            | "function_handle"
            | "int8"
            | "int16"
            | "int32"
            | "int64"
            | "uint8"
            | "uint16"
            | "uint32"
            | "uint64"
            | "table"
            | "datetime"
            | "duration"
            | "categorical"
            | "calendarDuration"
    )
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
const NON_SUBCLASSABLE_BUILTINS: &[&str] = &[
    "double", "single", "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64",
    "char", "logical", "cell", "struct",
];

/// Get the callee name of a `function_call` node.
pub(crate) fn callee_name(node: tree_sitter::Node, source: &str) -> Option<String> {
    callee_node(node).map(|n| node_text(n, source).to_string())
}

/// Get the callee identifier node of a `function_call`.
pub(crate) fn callee_node(node: tree_sitter::Node) -> Option<tree_sitter::Node> {
    node.child_by_field_name("name")
        .or_else(|| node.child(0))
        .filter(|n| n.kind() == "identifier")
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
mod check_end_operator;
mod check_error_warning_message;
mod check_fconv;
mod check_fv_ordering;
mod check_fv_structure;
mod check_fv_validation;
mod check_fvapn;
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
mod check_fvonv;
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
mod check_fvvin_fvocon;
mod check_fvvin_fvvcon_fvubd;
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
        parser.parse(source, None).expect("failed to parse")
    }

    /// Run the language spec engine on source code with a given file path.
    pub(crate) fn check_source(source: &str, file_path: &str) -> Vec<Diagnostic> {
        let config = Config::default();
        let rule = LanguageSpecEngine::from_config(&config);
        let tree = parse_matlab(source);
        let path = Path::new(file_path);
        let ctx = FileContext {
            tree: &tree,
            source,
            file_path: path,
        };
        rule.check_file(&ctx)
    }

    /// Filter diagnostics by rule_id.
    pub(crate) fn filter_by_id<'a>(diagnostics: &'a [Diagnostic], id: &str) -> Vec<&'a Diagnostic> {
        diagnostics.iter().filter(|d| d.rule_id == id).collect()
    }

    // ===== Function validation checks (FV* / TIN* / TTOO* group) =====

    const FV_CHECK_IDS: &[&str] = &[
        "FVAPN",
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
    fn test_spbrk_covered_by_spret() {
        // SPBRK (break/continue not fully contained in spmd) is subsumed by SPRET,
        // which fires for every break/continue inside an spmd block.
        let source = "\
function foo()
    spmd
        break;
    end
end
";
        let diags = check_source(source, "foo.m");
        let spret = filter_by_id(&diags, "SPRET");
        assert!(
            !spret.is_empty(),
            "SPRET should fire for break inside spmd (covers SPBRK)"
        );
        let spbrk = filter_by_id(&diags, "SPBRK");
        assert!(
            spbrk.is_empty(),
            "SPBRK is deferred — no separate diagnostic emitted"
        );
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
    fn test_pfanon_fires_sliced_var_in_lambda() {
        let source = "\
function f()
    parfor i = 1:10
        x(i) = i;
        g = @(t) x(t);
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

    // -- PFSLO ---------------------------------------------------------------

    #[test]
    fn test_pfslo_fires_temp_indexed_with_loop_variable() {
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
            !filter_by_id(&diags, "PFSLO").is_empty(),
            "PFSLO should fire when a temporary is indexed with the loop variable"
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
    fn test_pfslw_fires_different_subscripts() {
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
            !filter_by_id(&diags, "PFSLW").is_empty(),
            "PFSLW should fire for mismatched subscript lists"
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
    fn test_pfunk_fires_sliced_and_temp_mix() {
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
            !filter_by_id(&diags, "PFUNK").is_empty(),
            "PFUNK should fire when a variable is both sliced and a temporary"
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
        let diags = check_source(source, "script.m");
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
