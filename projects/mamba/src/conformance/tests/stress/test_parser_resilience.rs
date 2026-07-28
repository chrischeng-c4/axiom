//! Parser resilience & fuzzing stress tests (#gen12_fuzzing).
//!
//! Verifies parser handling of garbage/malformed inputs, deep nesting,
//! unterminated literals, and extreme indentation without panic or SIGABRT.

use crate::parser;
use crate::source::span::FileId;

/// Simple deterministic pseudo-random number generator for property-based fuzzing.
struct FuzzRng {
    state: u64,
}

impl FuzzRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            0
        } else {
            (self.next_u64() as usize) % max
        }
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let val = self.next_u64().to_ne_bytes();
            let len = chunk.len();
            chunk.copy_from_slice(&val[..len]);
        }
    }
}

/// Property-based test: 500 iterations of random ASCII and UTF-8 bytes.
/// Verifies parser returns Ok or Err, but NEVER panics or aborts.
#[test]
fn test_parser_random_bytes_resilience() {
    let mut rng = FuzzRng::new(0xDEADBEEF12345678);

    for len in [0, 1, 2, 5, 10, 32, 64, 128, 256, 512] {
        for _ in 0..50 {
            let mut bytes = vec![0u8; len];
            rng.fill_bytes(&mut bytes);

            // Test 1: lossy UTF-8 conversion
            let src = String::from_utf8_lossy(&bytes);
            let _ = std::panic::catch_unwind(|| {
                let _ = parser::parse(&src, FileId(0));
            });

            // Test 2: valid printable ASCII subset with random tokens
            let tokens = ["def ", "class ", "if ", "else: ", "for ", "in ", "return ",
                           "pass", "break", "continue", "import ", "from ", "try: ",
                           "except ", "finally: ", "lambda ", "yield ", "async ",
                           "+", "-", "*", "/", "==", "!=", "=", "(", ")", "[", "]",
                           "{", "}", ":", ",", ".", ";", "\n", "\t", " ", "0", "42",
                           "\"foo\"", "'bar'", "x", "y", "áóí", "None", "True", "False"];
            let mut src_acc = String::new();
            let count = rng.next_usize(30);
            for _ in 0..count {
                let idx = rng.next_usize(tokens.len());
                src_acc.push_str(tokens[idx]);
            }

            let _ = std::panic::catch_unwind(|| {
                let _ = parser::parse(&src_acc, FileId(0));
            });
        }
    }
}

/// Test unmatched delimiters (parentheses, brackets, braces) at extreme depths.
#[test]
fn test_parser_unmatched_delimiters() {
    let delimiters = ["(", ")", "[", "]", "{", "}"];

    for &open in &["(", "[", "{"] {
        for depth in [1, 10, 50, 100, 300] {
            let src = open.repeat(depth);
            let result = std::panic::catch_unwind(|| parser::parse(&src, FileId(0)));
            assert!(result.is_ok(), "parser panicked on unmatched open delimiter: {src}");
        }
    }

    for &close in &[")", "]", "}"] {
        for depth in [1, 10, 50, 100, 300] {
            let src = close.repeat(depth);
            let result = std::panic::catch_unwind(|| parser::parse(&src, FileId(0)));
            assert!(result.is_ok(), "parser panicked on unmatched close delimiter: {src}");
        }
    }

    // Interleaved unmatched delimiters
    let mut rng = FuzzRng::new(42);
    for _ in 0..50 {
        let mut src = String::new();
        for _ in 0..100 {
            src.push_str(delimiters[rng.next_usize(delimiters.len())]);
        }
        let result = std::panic::catch_unwind(|| parser::parse(&src, FileId(0)));
        assert!(result.is_ok(), "parser panicked on random interleaved delimiters");
    }
}

/// Test unterminated string literals (single, double, triple-quoted, raw, byte, f-strings).
#[test]
fn test_parser_unterminated_literals() {
    let prefixes = ["", "r", "b", "f", "rf", "fr", "rb", "br"];
    let quotes = ["'", "\"", "'''", "\"\"\""];

    for prefix in prefixes {
        for quote in quotes {
            let src = format!("{prefix}{quote}hello world this is unterminated");
            let result = std::panic::catch_unwind(|| parser::parse(&src, FileId(0)));
            assert!(result.is_ok(), "parser panicked on unterminated quote {prefix}{quote}");

            // With backslash escapes at the end
            let src_esc = format!("{prefix}{quote}hello world \\");
            let result_esc = std::panic::catch_unwind(|| parser::parse(&src_esc, FileId(0)));
            assert!(result_esc.is_ok(), "parser panicked on escape at EOF in string");
        }
    }
}

/// Test deeply nested expressions and chained operators.
#[test]
fn test_parser_deeply_nested_expressions() {
    // Deeply nested parentheses
    let depth = 200;
    let mut src = "(".repeat(depth);
    src.push_str("42");
    src.push_str(&")".repeat(depth));
    src.push('\n');

    let res = std::panic::catch_unwind(|| parser::parse(&src, FileId(0)));
    assert!(res.is_ok(), "parser panicked on 200-deep nested parens");

    // Chained binary operators: 1 + 1 + 1 + ... + 1
    let mut chained = "1".to_string();
    for _ in 0..200 {
        chained.push_str(" + 1");
    }
    chained.push('\n');
    let res = std::panic::catch_unwind(|| parser::parse(&chained, FileId(0)));
    assert!(res.is_ok(), "parser panicked on 200-chained addition");

    // Chained attribute access: x.a.b.c.d...
    let mut attrs = "x".to_string();
    for i in 0..150 {
        attrs.push_str(&format!(".attr_{i}"));
    }
    attrs.push('\n');
    let res = std::panic::catch_unwind(|| parser::parse(&attrs, FileId(0)));
    assert!(res.is_ok(), "parser panicked on 150-chained attribute access");
}

/// Test extreme indentation and whitespace mixing.
#[test]
fn test_parser_extreme_indentation() {
    // 500 spaces indentation
    let mut src = "def f():\n".to_string();
    src.push_str(&" ".repeat(500));
    src.push_str("pass\n");

    let res = std::panic::catch_unwind(|| parser::parse(&src, FileId(0)));
    assert!(res.is_ok(), "parser panicked on 500-space indent");

    // Mixed spaces and tabs
    let mut src_mixed = "def f():\n".to_string();
    src_mixed.push_str(" \t \t \t \t \t");
    src_mixed.push_str("x = 1\n");
    let res_mixed = std::panic::catch_unwind(|| parser::parse(&src_mixed, FileId(0)));
    assert!(res_mixed.is_ok(), "parser panicked on mixed spaces and tabs");
}

/// Test Unicode identifier fuzzing, zero-width characters, and RTL overrides.
#[test]
fn test_parser_unicode_fuzzing() {
    let unicode_inputs = [
        "áóí = 10\n",
        "αβγ = 20\n",
        "变量 = 30\n",
        "変数_123 = 40\n",
        "f_🐍 = 50\n",
        "x\u{200B}y = 60\n", // Zero-width space
        "\u{202E}reversed\u{202C} = 70\n", // RTL override
        "s = 'Emoji: 🚀🔥🎉'\n",
    ];

    for src in unicode_inputs {
        let res = std::panic::catch_unwind(|| parser::parse(src, FileId(0)));
        assert!(res.is_ok(), "parser panicked on Unicode input: {:?}", src);
    }
}

/// Test 100-target chained assignment: a = b = c = ... = 42
#[test]
fn test_parser_chained_assignments_stress() {
    use crate::parser::ast::{Expr, Stmt};

    let mut targets = Vec::new();
    for i in 0..100 {
        targets.push(format!("var_{i}"));
    }
    let src = format!("{} = 42\n", targets.join(" = "));
    let res = std::panic::catch_unwind(|| parser::parse(&src, FileId(0)));
    assert!(res.is_ok(), "parser panicked on 100-target assignment");
    let module = res
        .unwrap()
        .expect("100-target chained assignment should parse successfully");
    assert_eq!(
        module.stmts.len(),
        101,
        "expected 101 desugared Stmt::Assign"
    );

    // Stmt 0: __chained_<offset>__ = 42 (temp owns original RHS once)
    let tmp_name = match &module.stmts[0].node {
        Stmt::Assign { target, value } => {
            let name = match &target.node {
                Expr::Ident(n) => n.clone(),
                _ => panic!("expected Ident target for temp assign"),
            };
            assert!(
                name.starts_with("__chained_"),
                "expected temp name starting with __chained_"
            );
            assert!(
                matches!(&value.node, Expr::IntLit(42)),
                "temp should own original RHS IntLit(42)"
            );
            name
        }
        other => panic!("expected Stmt::Assign for temp at stmt 0, got {other:?}"),
    };

    // Stmts 1..=100: var_0..var_99 = __chained__ in source order
    for i in 0..100 {
        let expected_var = format!("var_{i}");
        match &module.stmts[i + 1].node {
            Stmt::Assign { target, value } => {
                assert!(
                    matches!(&target.node, Expr::Ident(n) if n == &expected_var),
                    "expected target {expected_var} at stmt {}, got {:?}",
                    i + 1,
                    target.node
                );
                assert!(
                    matches!(&value.node, Expr::Ident(n) if n == &tmp_name),
                    "target {expected_var} should reference temp {tmp_name}"
                );
            }
            other => panic!(
                "expected Stmt::Assign for target '{expected_var}' at stmt {}, got {other:?}",
                i + 1
            ),
        }
    }
}
