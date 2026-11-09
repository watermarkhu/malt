//! Function-call deployment checks: MCCD, MCPRD, MCHLP, MCKBD, MCSVP, MCMLR,
//! MCABF, MCMFL, MCTBX, MCLL.

use super::*;

/// Functions that read or write files, for the absolute-path check (MCABF).
const FILE_FUNCTIONS: &[&str] = &[
    "addpath",
    "fopen",
    "load",
    "save",
    "fileread",
    "imread",
    "imwrite",
    "readtable",
    "writetable",
    "readmatrix",
    "csvread",
    "dlmread",
    "xlsread",
    "importdata",
    "textscan",
    "audioread",
    "matfile",
];

/// Whether a `fopen` permission string opens the file for writing/appending.
fn permission_writes(perm: &str) -> bool {
    perm.contains('w') || perm.contains('a') || perm.contains('+')
}

impl DeploymentEngine {
    /// Check a `function_call` node for deployment-restricted functions.
    pub(crate) fn check_function_call<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &'a str,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let Some(func_name) = extract_func_name(node, source) else {
            return diags;
        };
        let args = call_args(node);
        let strings: Vec<&str> = args
            .iter()
            .filter_map(|a| string_literal(*a, source))
            .collect();

        let simple = match func_name {
            "cd" => Some("MCCD"),
            "help" => Some("MCHLP"),
            "keyboard" => Some("MCKBD"),
            "savepath" => Some("MCSVP"),
            "matlabroot" => Some("MCMLR"),
            _ => None,
        };
        if let Some(id) = simple {
            if self.is_enabled(id) {
                diags.push(make_diag(id, node));
            }
        }

        // MCPRD: printdlg accepts exactly one argument.
        if self.is_enabled("MCPRD") && func_name == "printdlg" && args.len() != 1 {
            diags.push(make_diag("MCPRD", node));
        }

        // MCABF: absolute file name passed to a file-accessing function.
        if self.is_enabled("MCABF")
            && FILE_FUNCTIONS.contains(&func_name)
            && strings.iter().any(|s| is_absolute_path(s))
        {
            diags.push(make_diag("MCABF", node));
        }

        // MCTBX: file in the MATLAB toolbox folder.
        if self.is_enabled("MCTBX") && strings.iter().any(|s| is_toolbox_path(s)) {
            diags.push(make_diag("MCTBX", node));
        }

        // MCMFL: fopen of a .m file for writing.
        if self.is_enabled("MCMFL")
            && func_name == "fopen"
            && strings.first().is_some_and(|f| f.ends_with(".m"))
            && strings.get(1).is_some_and(|p| permission_writes(p))
        {
            diags.push(make_diag("MCMFL", node));
        }

        // MCLL: loadlibrary driven by a C/C++ header file.
        if self.is_enabled("MCLL")
            && func_name == "loadlibrary"
            && strings.iter().any(|s| {
                let l = s.to_ascii_lowercase();
                l.ends_with(".h")
                    || l.ends_with(".hpp")
                    || l.ends_with(".hh")
                    || l.ends_with(".hxx")
            })
        {
            diags.push(make_diag("MCLL", node));
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::super::engine;
    use crate::test_util::{has_id, lint_nodes};

    fn ids(src: &str) -> Vec<&'static str> {
        lint_nodes(&*engine(), src)
            .iter()
            .map(|d| d.rule_id)
            .collect()
    }

    #[test]
    fn simple_function_checks() {
        assert!(ids("cd('/x');\n").contains(&"MCCD"));
        assert!(ids("help('plot');\n").contains(&"MCHLP"));
        assert!(ids("keyboard();\n").contains(&"MCKBD"));
        assert!(ids("savepath();\n").contains(&"MCSVP"));
        assert!(ids("r = matlabroot();\n").contains(&"MCMLR"));
    }

    #[test]
    fn simple_function_checks_no_fire() {
        for src in [
            "disp('x');\n",
            "plot(x, y);\n",
            "r = pwd;\n",
            "doc('plot');\n",
        ] {
            assert!(ids(src).is_empty(), "{src}: {:?}", ids(src));
        }
    }

    #[test]
    fn addpath_relative_not_flagged() {
        assert!(ids("addpath('src');\n").is_empty());
        assert!(ids("path(p);\n").is_empty());
    }

    #[test]
    fn mcprd_requires_exactly_one_argument() {
        assert!(has_id(
            &lint_nodes(&*engine(), "printdlg(h, 2);\n"),
            "MCPRD"
        ));
        assert!(has_id(&lint_nodes(&*engine(), "printdlg();\n"), "MCPRD"));
        assert!(!has_id(&lint_nodes(&*engine(), "printdlg(h);\n"), "MCPRD"));
    }

    #[test]
    fn mcabf_fires_on_absolute_paths() {
        assert!(ids("fid = fopen('/home/user/a.txt');\n").contains(&"MCABF"));
        assert!(ids("addpath('C:\\mytools');\n").contains(&"MCABF"));
        assert!(!ids("fid = fopen('data/a.txt');\n").contains(&"MCABF"));
        assert!(!ids("disp('/not/a/file');\n").contains(&"MCABF"));
    }

    #[test]
    fn mcmfl_fires_on_writing_m_file() {
        assert!(ids("fid = fopen('gen.m', 'w');\n").contains(&"MCMFL"));
        assert!(!ids("fid = fopen('gen.m', 'r');\n").contains(&"MCMFL"));
        assert!(!ids("fid = fopen('gen.txt', 'w');\n").contains(&"MCMFL"));
        assert!(!ids("name = mfilename();\n").contains(&"MCMFL"));
    }

    #[test]
    fn mctbx_fires_on_toolbox_paths() {
        assert!(ids("x = fileread('toolbox/matlab/a.txt');\n").contains(&"MCTBX"));
        assert!(ids("x = fileread('/opt/matlab/toolbox/a.txt');\n").contains(&"MCTBX"));
        assert!(!ids("y = fft2(x);\n").contains(&"MCTBX"));
        assert!(!ids("x = fileread('mytoolboxes/a.txt');\n").contains(&"MCTBX"));
    }

    #[test]
    fn mcll_fires_on_header_form_loadlibrary() {
        assert!(ids("loadlibrary('lib', 'lib.h');\n").contains(&"MCLL"));
        assert!(!ids("loadlibrary('lib', @libproto);\n").contains(&"MCLL"));
        assert!(!ids("lic = license('test', 'x');\n").contains(&"MCLL"));
    }
}
