//! The Safe fixes of the second 5.5 pass (`docs/VERIFY.md`, '5.5 Curated safe
//! autofixes'): their text is MATLAB's own, and the cases where malt withholds
//! the fix although MATLAB offers one.

use std::path::Path;

use malt_core::{fix_source, Config, FixMode, Linter, RuleRegistry};

fn fixed(source: &str) -> String {
    let config = Config::default();
    let registry = RuleRegistry::new(malt_rules::active_rules(&config), &config);
    let mut linter = Linter::new(registry);
    fix_source(&mut linter, source, Path::new("case.m"), FixMode::Safe)
        .unwrap()
        .source
}

/// Run `check` on a big stack, as the CLI does.
fn on_big_stack(check: fn()) {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(check)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn feval_with_a_literal_name_becomes_the_direct_call() {
    on_big_stack(|| {
        for (src, want) in [
            ("r = feval('sin', 1);\n", "r = sin(1);\n"),
            ("r = feval( @sin , 1 );\n", "r = sin( 1 );\n"),
            ("feval(@disp)\n", "disp()\n"),
            ("r = feval(@pkg.fn, 1);\n", "r = pkg.fn(1);\n"),
            ("r = feval(\"a.b\", 1, 2);\n", "r = a.b(1, 2);\n"),
        ] {
            assert_eq!(fixed(src), want, "{src}");
        }
    });
}

#[test]
fn feval_keeps_its_call_when_the_direct_call_would_index_a_variable() {
    on_big_stack(|| {
        // MATLAB's own fix writes `psd(1)` here; `psd` is a variable.
        let src = "psd = 5;\nr = feval('psd', 1);\n";
        assert_eq!(fixed(src), src);
        // A variable or an anonymous function as the callee is left alone.
        let src = "f = @sin;\nr = feval(f, 1);\ns = feval(@(x) x + 1, 2);\n";
        assert_eq!(fixed(src), src);
    });
}

/// Only the fixes of one check ID, repeated until stable.
fn fixed_only(source: &str, id: &str) -> String {
    let config = Config::default();
    let registry = RuleRegistry::new(malt_rules::active_rules(&config), &config);
    let mut linter = Linter::new(registry);
    let mut current = source.to_string();
    for _ in 0..5 {
        let found: Vec<_> = linter
            .lint(&current, Path::new("f.m"))
            .unwrap()
            .into_iter()
            .filter(|d| d.rule_id == id)
            .collect();
        let (next, _) = malt_core::apply_fixes_with(&current, &found, FixMode::Safe);
        if next == current {
            break;
        }
        current = next;
    }
    current
}

#[test]
fn msnu_removes_what_matlab_removes() {
    on_big_stack(|| {
        for (src, want) in [
            ("disp(1); %#ok<AGROW>\n", "disp(1); \n"),
            ("disp(1); %#ok<AGROW> remark\n", "disp(1); % remark\n"),
            ("disp(1); %#ok\n", "disp(1); \n"),
            ("y = 2; %#ok<NASGU,AGROW>\n", "y = 2; %#ok<NASGU>\n"),
            ("y = 2; %#ok<AGROW,NASGU>\n", "y = 2; %#ok<NASGU>\n"),
            // Every ID of a list unused: no fix (unprobed).
            (
                "disp(1); %#ok<AGROW,PFBNS>\n",
                "disp(1); %#ok<AGROW,PFBNS>\n",
            ),
        ] {
            let text = format!("function f()\n{src}end\n");
            assert_eq!(
                fixed_only(&text, "MSNU"),
                format!("function f()\n{want}end\n"),
                "{src}"
            );
        }
    });
}

#[test]
fn twocm_strcmp_and_string_fixes() {
    on_big_stack(|| {
        assert_eq!(fixed("x = [1,,2];\n"), "x = [1,2];\n");
        assert_eq!(fixed("x = [1,,,2];\n"), "x = [1,2];\n");
        assert_eq!(
            fixed("s = string('it''s');\nt = string( 'a\"b' );\n"),
            "s = \"it's\";\nt =  \"a\"\"b\" ;\n"
        );
        // A variable holding text is reported but not rewritten.
        let src = "v = 'abc';\ns = string(v);\n";
        assert_eq!(fixed(src), src);
        assert_eq!(fixed("format\n"), "format(\"default\")\n");
    });
}
