use crate::codegen::cranelift::jit::CraneliftJitBackend;
use crate::codegen::CodegenBackend;
use crate::codegen::CodegenOutput;
use crate::diagnostic;
use crate::error::MambaError;
use crate::hir::{HirClass, HirFunction};
use crate::lower;
use crate::lower::ReplSymInfo;
use crate::parser;
use crate::resolve::SymbolId;
use crate::runtime::MbValue;
use crate::source::SourceMap;
use crate::types::TypeChecker;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::{HashMap, HashSet};
/// REPL (Read-Eval-Print Loop) for Mamba (#316).
///
/// Provides an interactive Python-like interpreter shell with:
/// - Multi-line input support (auto-detect incomplete blocks)
/// - Expression evaluation with result printing (R1)
/// - Incremental JIT compilation per iteration (R2)
/// - Persistent global state: variables, functions, classes (R3)
/// - Error recovery (don't exit on errors)
use std::io;

/// REPL state.
pub struct Repl {
    source_map: SourceMap,
    checker: TypeChecker,
    line_count: u32,
    verbose: bool,
    /// Exact runtime bindings persisted across iterations (R3).
    known_globals: HashSet<SymbolId>,
    /// Accumulated function definitions from previous iterations (R3).
    accumulated_functions: Vec<HirFunction>,
    /// Class metadata retained for nested classes in accumulated functions.
    accumulated_classes: Vec<HirClass>,
    /// Accumulated SymbolId → (name, type) mapping across iterations.
    accumulated_syms: ReplSymInfo,
    /// Canonical Python name → current binding symbol.
    accumulated_bindings: HashMap<String, SymbolId>,
    /// Class binding names available as runtime base expressions.
    known_class_names: HashSet<String>,
    /// Keep prior JIT code alive because persisted class method tables retain
    /// function pointers into the module that created them.
    jit_backends: Vec<CraneliftJitBackend>,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            source_map: SourceMap::new(),
            checker: TypeChecker::new(),
            line_count: 0,
            verbose: false,
            known_globals: HashSet::new(),
            accumulated_functions: Vec::new(),
            accumulated_classes: Vec::new(),
            accumulated_syms: ReplSymInfo::new(),
            accumulated_bindings: HashMap::new(),
            known_class_names: HashSet::new(),
            jit_backends: Vec::new(),
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Run the REPL loop with rustyline for readline support (history,
    /// line editing, Ctrl-R search).
    pub fn run(&mut self) -> io::Result<()> {
        println!("Mamba 0.1.0 (interactive mode)");
        println!("Type \"exit()\" or Ctrl-D to quit.\n");

        let mut rl = DefaultEditor::new()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        // Load history from ~/.mamba_history if it exists.
        let history_path = dirs_history_path();
        if let Some(ref path) = history_path {
            let _ = rl.load_history(path);
        }

        let mut buffer = String::new();
        let mut continuation = false;

        loop {
            let prompt = if continuation { "... " } else { ">>> " };

            match rl.readline(prompt) {
                Ok(line) => {
                    let trimmed = line.trim_end();

                    if !continuation && (trimmed == "exit()" || trimmed == "quit()") {
                        break;
                    }

                    if !continuation && trimmed.starts_with('%') {
                        rl.add_history_entry(&line).ok();
                        self.handle_magic(trimmed);
                        continue;
                    }

                    buffer.push_str(&line);
                    buffer.push('\n');

                    if needs_continuation(&buffer) {
                        continuation = true;
                        continue;
                    }

                    let input = buffer.trim().to_string();
                    // Add the complete input to history (not individual lines).
                    if !input.is_empty() {
                        rl.add_history_entry(&input).ok();
                    }
                    buffer.clear();
                    continuation = false;

                    if input.is_empty() {
                        continue;
                    }

                    self.eval(&input);
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl-C: cancel current input, restart prompt.
                    if continuation {
                        buffer.clear();
                        continuation = false;
                        println!("KeyboardInterrupt");
                    } else {
                        println!("KeyboardInterrupt");
                    }
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl-D: exit.
                    println!();
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {err}");
                    break;
                }
            }
        }

        // Save history.
        if let Some(ref path) = history_path {
            let _ = rl.save_history(path);
        }

        Ok(())
    }

    /// Evaluate a single input string with persistent globals (R2, R3).
    fn eval(&mut self, input: &str) {
        match self.eval_raw(input) {
            Ok((result, has_echo)) => {
                if has_echo && !MbValue::from_bits(result as u64).is_none() {
                    println!("{result}");
                }
            }
            Err(msg) => {
                eprintln!("{msg}");
            }
        }
    }

    /// Core evaluation returning (raw_result, has_expression_echo).
    fn eval_raw(&mut self, input: &str) -> Result<(i64, bool), String> {
        let checker_snapshot = self.checker.clone();
        let result = self.eval_raw_inner(input);
        if result.is_err() {
            self.checker = checker_snapshot;
        }
        result
    }

    fn eval_raw_inner(&mut self, input: &str) -> Result<(i64, bool), String> {
        self.line_count += 1;
        let file_name = format!("<repl:{}>", self.line_count);
        let file_id = self.source_map.add_file(file_name, input.to_string());

        // Parse
        let mut module = parser::parse(input, file_id)
            .map_err(|e| format!("SyntaxError: {}", self.format_error(&e)))?;
        crate::lower::pep695::desugar_module(&mut module);
        let module = module;

        // Type check (checker persists across iterations)
        let errors = self.checker.check_module(&module);
        if !errors.is_empty() {
            let msgs: Vec<String> = errors
                .iter()
                .map(|e| format!("TypeError: {}", self.format_error(e)))
                .collect();
            return Err(msgs.join("\n"));
        }

        // Lower AST → HIR (REPL-aware: pre-seed known globals from prev iterations)
        let hir = lower::lower_module_repl(
            &module,
            &self.checker,
            &self.accumulated_syms,
            &self.accumulated_bindings,
            &self.known_class_names,
        )
        .map_err(|errs| {
            errs.iter()
                .map(|e| format!("LowerError: {}", self.format_error(e)))
                .collect::<Vec<_>>()
                .join("\n")
        })?;

        // Save new definitions for cross-iteration persistence (R3)
        let new_functions = hir.functions.clone();
        let new_classes = hir.classes.clone();
        // Build ReplSymInfo: merge sym_names with sym_types
        let new_syms: ReplSymInfo = hir
            .sym_names
            .iter()
            .map(|(&id, name)| {
                let ty = hir
                    .sym_types
                    .get(&id)
                    .copied()
                    .unwrap_or_else(|| self.checker.tcx.any());
                (id, (name.clone(), ty))
            })
            .collect();

        // Lower HIR → MIR (REPL-aware: restore/save globals, echo last expr)
        let prev: Vec<SymbolId> = self.known_globals.iter().copied().collect();
        let (mir_module, new_globals, has_echo) = lower::lower_hir_to_mir_repl(
            &hir,
            &self.checker.tcx,
            &self.checker.symbols,
            &prev,
            &self.accumulated_functions,
            &self.accumulated_classes,
        );

        if self.verbose {
            eprintln!("[MIR] {} bodies", mir_module.bodies.len());
        }

        // JIT compile and execute
        let mut backend = CraneliftJitBackend::new().map_err(|e| format!("JIT error: {e}"))?;

        let output = backend
            .codegen(&mir_module, &self.checker.tcx)
            .map_err(|e| format!("CodegenError: {}", self.format_error(&e)))?;

        match output {
            CodegenOutput::Jit { entry } => {
                let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry) };
                let result = main_fn();
                // Update state only after successful execution (avoids ghost globals)
                self.known_globals = new_globals.iter().copied().collect();
                for &symbol in &new_globals {
                    if let Some(name) = hir.sym_names.get(&symbol) {
                        self.accumulated_bindings.insert(name.clone(), symbol);
                    }
                }
                for function in &new_functions {
                    let binding = function.bind_name.unwrap_or(function.name);
                    if let Some(name) = hir.sym_names.get(&binding) {
                        self.accumulated_bindings.insert(name.clone(), binding);
                    }
                }
                for class in &new_classes {
                    if let Some(name) = hir
                        .sym_names
                        .get(&class.bind_name)
                        .or_else(|| hir.sym_names.get(&class.name))
                    {
                        self.accumulated_bindings
                            .insert(name.clone(), class.bind_name);
                        self.known_class_names.insert(name.clone());
                    }
                }
                self.accumulated_functions.extend(new_functions);
                self.accumulated_classes.extend(new_classes);
                self.accumulated_syms.extend(new_syms);
                self.jit_backends.push(backend);
                // Decode NaN-boxed int results to raw i64 for consistent REPL
                // semantics (R7). The R7 fix causes emit_internal_call to NaN-box
                // primitive callee results, so the JIT entry now returns a NaN-boxed
                // MbValue instead of a raw integer when the last expression is a
                // typed function call. Non-int MbValues (including MbValue::none())
                // pass through as raw bits so the None-guard in eval() still works.
                let decoded = MbValue::from_bits(result as u64).as_int().unwrap_or(result);
                Ok((decoded, has_echo))
            }
            _ => Err("Error: unexpected codegen output".to_string()),
        }
    }

    /// Handle magic commands (% prefix).
    fn handle_magic(&mut self, cmd: &str) {
        match cmd {
            "%verbose" => {
                self.verbose = !self.verbose;
                println!("verbose mode: {}", if self.verbose { "on" } else { "off" });
            }
            "%help" => {
                println!("Mamba REPL commands:");
                println!("  exit()    - Exit the REPL");
                println!("  quit()    - Exit the REPL");
                println!("  %help     - Show this help");
                println!("  %verbose  - Toggle verbose output");
            }
            _ => {
                println!("Unknown command: {cmd}");
            }
        }
    }

    fn format_error(&self, err: &MambaError) -> String {
        diagnostic::render_error(err, &self.source_map)
    }
}

/// Return the path to `~/.mamba_history`, or `None` if the home directory
/// cannot be determined.
fn dirs_history_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".mamba_history"))
}

/// Check if the input needs more lines (incomplete block).
fn needs_continuation(input: &str) -> bool {
    let trimmed = input.trim_end();

    // Ends with colon → likely start of a block
    if trimmed.ends_with(':') {
        return true;
    }

    // Ends with backslash → explicit continuation
    if trimmed.ends_with('\\') {
        return true;
    }

    // Unbalanced brackets/parens
    let mut parens = 0i32;
    let mut brackets = 0i32;
    let mut braces = 0i32;
    let mut in_string = false;
    let mut string_char = ' ';

    for ch in trimmed.chars() {
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }
        match ch {
            '\'' | '"' => {
                in_string = true;
                string_char = ch;
            }
            '(' => parens += 1,
            ')' => parens -= 1,
            '[' => brackets += 1,
            ']' => brackets -= 1,
            '{' => braces += 1,
            '}' => braces -= 1,
            _ => {}
        }
    }

    parens > 0 || brackets > 0 || braces > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_global(repl: &Repl, name: &str) -> bool {
        repl.known_globals.iter().any(|symbol| {
            repl.accumulated_syms
                .get(symbol)
                .is_some_and(|(stored_name, _)| stored_name == name)
        })
    }

    #[test]
    fn test_needs_continuation() {
        assert!(needs_continuation("if x > 0:"));
        assert!(needs_continuation("def foo():"));
        assert!(needs_continuation("[1, 2,"));
        assert!(needs_continuation("(1 +"));
        assert!(!needs_continuation("x = 42"));
        assert!(!needs_continuation("print(1)"));
    }

    #[test]
    fn test_repl_creation() {
        let repl = Repl::new();
        assert_eq!(repl.line_count, 0);
        assert!(repl.known_globals.is_empty());
        assert!(repl.accumulated_functions.is_empty());
    }

    // ── Integration tests for REPL persistence semantics ──

    #[test]
    fn test_repl_variable_persistence() {
        let mut repl = Repl::new();
        // Iteration 1: define variable
        let (_, echo) = repl.eval_raw("x: int = 42\n").unwrap();
        assert!(!echo, "assignment should not echo");
        assert!(has_global(&repl, "x"));
        // Iteration 2: use persisted variable in expression — should echo 42
        let (val, echo) = repl.eval_raw("x\n").unwrap();
        assert!(echo, "bare variable should echo");
        assert_eq!(val, 42, "x should persist as 42");
    }

    #[test]
    fn test_repl_function_persistence() {
        let mut repl = Repl::new();
        // Iteration 1: define function
        let r = repl.eval_raw("def double(n: int) -> int:\n    return n * 2\n");
        assert!(r.is_ok(), "define failed: {:?}", r);
        assert_eq!(repl.accumulated_functions.len(), 1);
        // Iteration 2: call persisted function — should echo 42
        let (val, echo) = repl.eval_raw("double(21)\n").unwrap();
        assert!(echo, "function call expression should echo");
        assert_eq!(val, 42, "double(21) should return 42");
    }

    #[test]
    fn test_repl_repeated_classes_keep_runtime_identity() {
        let mut repl = Repl::new();
        repl.eval_raw(
            "class C:\n    def value(self) -> int:\n        return 1\nOld = C\nold = Old()\n",
        )
        .unwrap();
        repl.eval_raw("class C:\n    def value(self) -> int:\n        return 2\nnew = C()\n")
            .unwrap();

        assert_eq!(repl.eval_raw("old.value()\n").unwrap().0, 1);
        assert_eq!(repl.eval_raw("new.value()\n").unwrap().0, 2);
        assert_eq!(
            repl.eval_raw("1 if type(old) is Old else 0\n").unwrap().0,
            1
        );
        assert_eq!(repl.eval_raw("1 if type(old) is C else 0\n").unwrap().0, 0);
    }

    #[test]
    fn test_repl_prior_class_can_be_a_new_base() {
        let mut repl = Repl::new();
        repl.eval_raw("class Base:\n    def value(self) -> int:\n        return 7\n")
            .unwrap();
        repl.eval_raw("class Child(Base):\n    pass\n").unwrap();
        assert_eq!(repl.eval_raw("Child().value()\n").unwrap().0, 7);
    }

    #[test]
    fn test_repl_accumulated_function_keeps_nested_class_metadata() {
        let mut repl = Repl::new();
        repl.eval_raw("def make():\n    class Local:\n        pass\n    return Local\n")
            .unwrap();
        repl.eval_raw("A = make()\nB = make()\n").unwrap();
        assert_eq!(repl.eval_raw("1 if A is B else 0\n").unwrap().0, 0);
        assert_eq!(
            repl.eval_raw("1 if A.__name__ == 'Local' else 0\n")
                .unwrap()
                .0,
            1
        );
    }

    #[test]
    fn test_repl_accumulated_function_keeps_nested_class_body_metadata() {
        let mut repl = Repl::new();
        repl.eval_raw(
            "def make():\n    class Outer:\n        class Inner:\n            pass\n    return Outer.Inner\n",
        )
        .unwrap();
        repl.eval_raw("First = make()\nSecond = make()\n")
            .unwrap();
        assert_eq!(
            repl.eval_raw("1 if First is Second else 0\n").unwrap().0,
            0
        );
        assert_eq!(
            repl.eval_raw("1 if First.__name__ == 'Inner' else 0\n")
                .unwrap()
                .0,
            1
        );
    }

    #[test]
    fn test_repl_class_binding_is_visible_to_same_eval_methods() {
        let mut repl = Repl::new();
        let (value, echo) = repl
            .eval_raw(
                "class C:\n    def owner(self):\n        return C\n1 if C().owner() is C else 0\n",
            )
            .unwrap();
        assert!(echo);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_repl_failed_iteration_no_ghost_state() {
        let mut repl = Repl::new();
        // Iteration 1: define variable
        let r = repl.eval_raw("x: int = 10\n");
        assert!(r.is_ok());
        let globals_before = repl.known_globals.clone();
        let funcs_before = repl.accumulated_functions.len();
        let syms_before = repl.accumulated_syms.len();
        // Iteration 2: syntax error — state should not change
        let r = repl.eval_raw("def bad(\n");
        assert!(r.is_err(), "should fail on syntax error");
        assert_eq!(
            repl.known_globals, globals_before,
            "ghost globals after failure"
        );
        assert_eq!(
            repl.accumulated_functions.len(),
            funcs_before,
            "ghost functions after failure"
        );
        assert_eq!(
            repl.accumulated_syms.len(),
            syms_before,
            "ghost sym_names after failure"
        );
        // Iteration 3: original variable still accessible
        let (val, echo) = repl.eval_raw("x\n").unwrap();
        assert!(echo, "x should echo after failed iteration");
        assert_eq!(val, 10, "x should still be 10");
    }

    #[test]
    fn test_repl_failed_function_definition_rolls_back_checker() {
        let mut repl = Repl::new();
        repl.eval_raw("keep: int = 40\n").unwrap();
        let keep_symbol = repl
            .checker
            .symbols
            .lookup("keep")
            .expect("valid binding must exist before the failed iteration");

        let failed = repl.eval_raw(
            "def ghost_function(value: int) -> int:\n    return 'wrong'\n",
        );
        assert!(failed.is_err(), "invalid function definition must fail");
        assert!(
            repl.checker.symbols.lookup("ghost_function").is_none(),
            "failed function definition leaked a checker symbol"
        );
        assert_eq!(
            repl.checker.symbols.lookup("keep"),
            Some(keep_symbol),
            "rollback must preserve the prior valid binding identity"
        );
        assert_eq!(repl.eval_raw("keep + 2\n").unwrap().0, 42);
    }

    #[test]
    fn test_repl_failed_class_definition_rolls_back_checker() {
        let mut repl = Repl::new();
        repl.eval_raw("keep: int = 39\n").unwrap();
        let keep_symbol = repl
            .checker
            .symbols
            .lookup("keep")
            .expect("valid binding must exist before the failed iteration");

        let failed = repl.eval_raw(
            "class GhostClass:\n    def broken(self) -> int:\n        return 'wrong'\n",
        );
        assert!(failed.is_err(), "invalid class definition must fail");
        assert!(
            repl.checker.symbols.lookup("GhostClass").is_none(),
            "failed class definition leaked a checker symbol"
        );
        assert_eq!(
            repl.checker.symbols.lookup("keep"),
            Some(keep_symbol),
            "rollback must preserve the prior valid binding identity"
        );
        assert_eq!(repl.eval_raw("keep + 3\n").unwrap().0, 42);
    }

    #[test]
    fn test_repl_multiple_variables_across_iterations() {
        let mut repl = Repl::new();
        repl.eval_raw("a: int = 1\n").unwrap();
        repl.eval_raw("b: int = 2\n").unwrap();
        assert!(has_global(&repl, "a"));
        assert!(has_global(&repl, "b"));
        // Use both persisted variables — should echo 3
        let (val, echo) = repl.eval_raw("a + b\n").unwrap();
        assert!(echo, "expression should echo");
        assert_eq!(val, 3, "a + b should be 3");
    }

    #[test]
    fn test_repl_expression_echo() {
        let mut repl = Repl::new();
        let (val, echo) = repl.eval_raw("42\n").unwrap();
        assert!(echo, "bare expression should echo");
        assert_eq!(val, 42, "expression echo for 42");
    }

    #[test]
    fn test_repl_expression_echo_zero() {
        let mut repl = Repl::new();
        let (val, echo) = repl.eval_raw("0\n").unwrap();
        assert!(echo, "expression `0` should echo");
        assert_eq!(val, 0, "expression echo for 0");
    }

    #[test]
    fn test_repl_assignment_no_echo() {
        let mut repl = Repl::new();
        let (_, echo) = repl.eval_raw("x: int = 99\n").unwrap();
        assert!(!echo, "assignment should not echo");
    }

    #[test]
    fn test_repl_print_no_echo() {
        let mut repl = Repl::new();
        // print() is an expression statement, so has_echo must be true
        let (val, has_echo) = repl.eval_raw("print(42)\n").unwrap();
        assert!(
            has_echo,
            "print() call expression should have has_echo = true"
        );
        // print() returns MbValue::none() — the None-guard in eval must suppress it
        assert!(
            MbValue::from_bits(val as u64).is_none(),
            "print() result must be TAG_NONE so the None-guard fires (got {val})"
        );
    }
}
