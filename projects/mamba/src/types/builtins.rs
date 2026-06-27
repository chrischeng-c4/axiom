use super::check::TypeChecker;
use super::{Ty, TypeId};
use crate::resolve::SymbolKind;

/// Built-in function registration and exception hierarchy (#245, #249).
impl TypeChecker {
    /// Register all built-in functions and types.
    pub(crate) fn register_builtins(&mut self) {
        self.register_builtin_functions();
        self.register_exception_hierarchy();
    }

    fn register_builtin_functions(&mut self) {
        let any = self.tcx.any();
        let int = self.tcx.int();
        let float = self.tcx.float();
        let bool_ty = self.tcx.bool();
        let str_ty = self.tcx.str();
        let none = self.tcx.none();

        // I/O — print(*args), input(prompt='')
        self.def_builtin_variadic("print", &[], none);
        self.def_builtin("input", &[str_ty], str_ty);

        // Type constructors — all accept 0-1 args (optional first param)
        self.def_builtin_variadic("int", &[], int);
        self.def_builtin_variadic("float", &[], float);
        self.def_builtin_variadic("bool", &[], bool_ty);
        self.def_builtin_variadic("str", &[], str_ty);
        self.def_builtin_variadic("list", &[], any);
        self.def_builtin_variadic("dict", &[], any);
        self.def_builtin_variadic("set", &[], any);
        self.def_builtin_variadic("frozenset", &[], any);
        self.def_builtin_variadic("tuple", &[], any);

        // Numeric — min/max accept *args, round(n, ndigits=0)
        self.def_builtin("abs", &[any], any);
        self.def_builtin_variadic("min", &[any], any);
        self.def_builtin_variadic("max", &[any], any);
        // sum returns int OR float depending on the iterable (sum of floats is a
        // float), so its static type must be `any` like min/max — NOT int. With
        // int, `s = sum([0.5, ...])` declared `s` as int and the float result's
        // NaN-box bits leaked as a huge integer when used (broke spectral-norm's
        // math.sqrt(vBv/vv)). `print(sum(...))` was unaffected because print
        // consumes `any`; only the assigned-then-used form leaked. round() is the
        // same shape: round(x) -> int but round(x, ndigits) -> float.
        self.def_builtin_variadic("sum", &[any], any);
        self.def_builtin_variadic("round", &[any], any);
        self.def_builtin_variadic("pow", &[any, any], any);
        let divmod_ret = self.tcx.intern(Ty::Tuple(vec![any, any]));
        self.def_builtin("divmod", &[any, any], divmod_ret);

        // Sequences / iterables
        self.def_builtin("len", &[any], int);
        self.def_builtin_variadic("range", &[any], any);
        // slice(stop), slice(start, stop[, step]) — same variadic form as range
        self.def_builtin_variadic("slice", &[], any);
        self.def_builtin_variadic("enumerate", &[any], any);
        self.def_builtin_variadic("zip", &[], any);
        self.def_builtin_variadic("map", &[any, any], any);
        self.def_builtin("filter", &[any, any], any);
        self.def_builtin_variadic("sorted", &[any], any);
        self.def_builtin("reversed", &[any], any);
        self.def_builtin_variadic("iter", &[any], any);
        self.def_builtin_variadic("next", &[any], any);

        // Introspection — getattr(obj, name[, default])
        self.def_builtin_variadic("type", &[any], any);
        self.def_builtin("isinstance", &[any, any], bool_ty);
        self.def_builtin("issubclass", &[any, any], bool_ty);
        self.def_builtin("hasattr", &[any, str_ty], bool_ty);
        self.def_builtin_variadic("getattr", &[any, str_ty], any);
        self.def_builtin("setattr", &[any, str_ty, any], none);
        self.def_builtin("delattr", &[any, str_ty], none);

        // PEP 695 desugaring intrinsics (see lower::pep695): runtime TypeVar
        // and TypeAliasType construction. Not part of the user-facing builtin
        // surface; only injected by the desugarer.
        self.def_builtin("__mb_pep695_typevar__", &[str_ty, int, any, any], any);
        self.def_builtin("__mb_pep695_type_alias__", &[str_ty, any, any], any);

        // Identity / hashing
        self.def_builtin("id", &[any], int);
        self.def_builtin("hash", &[any], int);
        self.def_builtin("repr", &[any], str_ty);

        // File / System — open(file, mode='r', ...)
        self.def_builtin_variadic("open", &[str_ty], any);

        // Class helpers — super() takes 0 or 2 args
        self.def_builtin_variadic("super", &[], any);
        self.def_builtin_variadic("property", &[], any);
        self.def_builtin("staticmethod", &[any], any);
        self.def_builtin("classmethod", &[any], any);

        // Predicates — any/all accept one iterable argument
        self.def_builtin("any", &[any], bool_ty);
        self.def_builtin("all", &[any], bool_ty);

        // Byte sequences — bytes/bytearray constructors accept 0-3 args
        self.def_builtin_variadic("bytes", &[], any);
        self.def_builtin_variadic("bytearray", &[], any);
        // complex(real=0, imag=0) — accepts 0-2 numeric args
        self.def_builtin_variadic("complex", &[], any);
        // breakpoint(*args, **kwargs) — PEP 553. No pdb runtime, so the
        // stub respects PYTHONBREAKPOINT=0 (silent no-op) and otherwise
        // also returns None — matching CPython when sys.breakpointhook
        // is the default and pdb is unavailable.
        self.def_builtin_variadic("breakpoint", &[], any);
        // memoryview(obj) — view into a bytes-like object.
        self.def_builtin_variadic("memoryview", &[], any);
        // __import__(name, globals=None, locals=None, fromlist=(), level=0)
        // Routes to the runtime import path; only `name` is honored,
        // matching the public CPython contract for `import x` (which is
        // implemented in CPython itself by calling __import__("x")).
        self.def_builtin_variadic("__import__", &[], any);

        // Other — format(value, format_spec=''), vars/dir optional arg
        self.def_builtin("callable", &[any], bool_ty);
        self.def_builtin("ord", &[str_ty], int);
        self.def_builtin("chr", &[int], str_ty);
        self.def_builtin("hex", &[int], str_ty);
        self.def_builtin("oct", &[int], str_ty);
        self.def_builtin("bin", &[int], str_ty);
        self.def_builtin("ascii", &[any], str_ty);
        self.def_builtin_variadic("format", &[any], str_ty);
        self.def_builtin_variadic("vars", &[], any);
        self.def_builtin_variadic("dir", &[], any);
        // HANDWRITE-BEGIN gap="standardize:projects-mamba-src-types-builtins-rs" tracker="standardize-gap-projects-mamba-src-types-builtins-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
        self.def_builtin_variadic("globals", &[], any);
        self.def_builtin_variadic("locals", &[], any);
        // HANDWRITE-END
        // eval / exec / compile (#441 / #976) — runtime stubs already exist
        // and are registered in builtins_mod.rs. Expose them as builtin names
        // so seeds using `eval(repr(v))` etc. type-check (#1565).
        self.def_builtin_variadic("eval", &[], any);
        self.def_builtin_variadic("exec", &[], any);
        self.def_builtin_variadic("compile", &[], any);

        // Builtin constants — NotImplemented, Ellipsis, __debug__
        let sym = self
            .symbols
            .define("NotImplemented".to_string(), SymbolKind::Variable);
        self.set_sym_type(sym.0, any);
        let sym = self
            .symbols
            .define("Ellipsis".to_string(), SymbolKind::Variable);
        self.set_sym_type(sym.0, any);

        // Module-level dunder variables — always available in every module.
        for dunder in &[
            "__name__",
            "__file__",
            "__doc__",
            "__package__",
            "__spec__",
            "__loader__",
            "__builtins__",
        ] {
            let sym = self
                .symbols
                .define(dunder.to_string(), SymbolKind::Variable);
            self.set_sym_type(sym.0, str_ty);
        }
        // `__annotations__` is a dict (PEP 526) auto-created at module init.
        // Typed `any` so `isinstance(__annotations__, dict)` and membership tests
        // route through the runtime dict carried in the global slot.
        let ann_sym = self
            .symbols
            .define("__annotations__".to_string(), SymbolKind::Variable);
        self.set_sym_type(ann_sym.0, any);
    }

    fn def_builtin(&mut self, name: &str, params: &[TypeId], ret: TypeId) {
        let fn_ty = self.tcx.intern(Ty::Fn {
            params: params.to_vec(),
            ret,
            variadic: false,
        });
        let sym = self.symbols.define(name.to_string(), SymbolKind::Function);
        self.set_sym_type(sym.0, fn_ty);
    }

    fn def_builtin_variadic(&mut self, name: &str, params: &[TypeId], ret: TypeId) {
        let fn_ty = self.tcx.intern(Ty::Fn {
            params: params.to_vec(),
            ret,
            variadic: true,
        });
        let sym = self.symbols.define(name.to_string(), SymbolKind::Function);
        self.set_sym_type(sym.0, fn_ty);
    }

    /// Register Python exception class hierarchy (#249).
    fn register_exception_hierarchy(&mut self) {
        let exception_names = [
            "object",
            "BaseException",
            "SystemExit",
            "KeyboardInterrupt",
            "GeneratorExit",
            "Exception",
            "StopIteration",
            "StopAsyncIteration",
            "ArithmeticError",
            "ZeroDivisionError",
            "OverflowError",
            "FloatingPointError",
            "LookupError",
            "IndexError",
            "KeyError",
            "OSError",
            "IOError",
            "FileNotFoundError",
            "PermissionError",
            "FileExistsError",
            "TypeError",
            "ValueError",
            "AttributeError",
            "NameError",
            "RuntimeError",
            "RecursionError",
            "NotImplementedError",
            "ImportError",
            "ModuleNotFoundError",
            "SyntaxError",
            "IndentationError",
            "UnicodeError",
            "UnicodeDecodeError",
            "UnicodeEncodeError",
            "UnicodeTranslateError",
            "AssertionError",
            "BufferError",
            "EOFError",
            "MemoryError",
            "ConnectionError",
            "ConnectionResetError",
            "ConnectionAbortedError",
            "ConnectionRefusedError",
            "BrokenPipeError",
            "IsADirectoryError",
            "NotADirectoryError",
            "InterruptedError",
            "ProcessLookupError",
            "ChildProcessError",
            "BlockingIOError",
            "ReferenceError",
            "TimeoutError",
            "ExceptionGroup",
            "BaseExceptionGroup",
            "Warning",
            "UserWarning",
            "DeprecationWarning",
            "PendingDeprecationWarning",
            "SyntaxWarning",
            "RuntimeWarning",
            "FutureWarning",
            "ImportWarning",
            "UnicodeWarning",
            "BytesWarning",
            "ResourceWarning",
            "EncodingWarning",
        ];
        for name in &exception_names {
            let class_ty = self.tcx.intern(Ty::Class {
                name: name.to_string(),
                fields: vec![],
                match_args: None,
            });
            let sym = self.symbols.define(name.to_string(), SymbolKind::Class);
            self.set_sym_type(sym.0, class_ty);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_print_is_fn() {
        let tc = TypeChecker::new();
        let sym = tc.symbols.lookup("print").expect("print not found");
        let ty = tc.get_sym_type(sym.0);
        match tc.tcx.get(ty) {
            Ty::Fn {
                params,
                ret,
                variadic,
            } => {
                assert_eq!(params.len(), 0); // print(*args)
                assert_eq!(*ret, tc.tcx.none());
                assert!(*variadic);
            }
            other => panic!("expected Fn, got {:?}", other),
        }
    }

    #[test]
    fn test_builtin_len_returns_int() {
        let tc = TypeChecker::new();
        let sym = tc.symbols.lookup("len").unwrap();
        let ty = tc.get_sym_type(sym.0);
        if let Ty::Fn { ret, .. } = tc.tcx.get(ty) {
            assert_eq!(*ret, tc.tcx.int());
        } else {
            panic!("len should be a function");
        }
    }

    #[test]
    fn test_builtin_isinstance_returns_bool() {
        let tc = TypeChecker::new();
        let sym = tc.symbols.lookup("isinstance").unwrap();
        let ty = tc.get_sym_type(sym.0);
        if let Ty::Fn { params, ret, .. } = tc.tcx.get(ty) {
            assert_eq!(params.len(), 2);
            assert_eq!(*ret, tc.tcx.bool());
        } else {
            panic!("isinstance should be a function");
        }
    }

    #[test]
    fn test_builtin_ord_takes_str_returns_int() {
        let tc = TypeChecker::new();
        let sym = tc.symbols.lookup("ord").unwrap();
        let ty = tc.get_sym_type(sym.0);
        if let Ty::Fn { params, ret, .. } = tc.tcx.get(ty) {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], tc.tcx.str());
            assert_eq!(*ret, tc.tcx.int());
        } else {
            panic!("ord should be a function");
        }
    }

    #[test]
    fn test_builtin_chr_takes_int_returns_str() {
        let tc = TypeChecker::new();
        let sym = tc.symbols.lookup("chr").unwrap();
        let ty = tc.get_sym_type(sym.0);
        if let Ty::Fn { params, ret, .. } = tc.tcx.get(ty) {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], tc.tcx.int());
            assert_eq!(*ret, tc.tcx.str());
        } else {
            panic!("chr should be a function");
        }
    }

    #[test]
    fn test_builtin_divmod_returns_tuple() {
        // divmod accepts any numeric (int or float); params/return elements are
        // typed `any` to keep mixed int/float calls type-checking. The shape
        // assertion (arity 2, returns 2-tuple) is the load-bearing part.
        let tc = TypeChecker::new();
        let sym = tc.symbols.lookup("divmod").unwrap();
        let ty = tc.get_sym_type(sym.0);
        if let Ty::Fn { params, ret, .. } = tc.tcx.get(ty) {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], tc.tcx.any());
            assert_eq!(params[1], tc.tcx.any());
            if let Ty::Tuple(ts) = tc.tcx.get(*ret) {
                assert_eq!(ts.len(), 2);
                assert_eq!(ts[0], tc.tcx.any());
                assert_eq!(ts[1], tc.tcx.any());
            } else {
                panic!("divmod return should be a tuple");
            }
        } else {
            panic!("divmod should be a function");
        }
    }

    #[test]
    fn test_builtin_super_no_params() {
        let tc = TypeChecker::new();
        let sym = tc.symbols.lookup("super").unwrap();
        let ty = tc.get_sym_type(sym.0);
        if let Ty::Fn { params, ret, .. } = tc.tcx.get(ty) {
            assert_eq!(params.len(), 0);
            assert_eq!(*ret, tc.tcx.any());
        } else {
            panic!("super should be a function");
        }
    }

    #[test]
    fn test_exception_is_class() {
        let tc = TypeChecker::new();
        let exceptions = [
            "ValueError",
            "TypeError",
            "KeyError",
            "IndexError",
            "RuntimeError",
            "BaseException",
            "Exception",
        ];
        for name in &exceptions {
            let sym = tc
                .symbols
                .lookup(name)
                .unwrap_or_else(|| panic!("{name} not found"));
            let ty = tc.get_sym_type(sym.0);
            match tc.tcx.get(ty) {
                Ty::Class {
                    name: class_name, ..
                } => {
                    assert_eq!(class_name, *name);
                }
                other => panic!("{name} should be a class, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_all_io_builtins_exist() {
        let tc = TypeChecker::new();
        assert!(tc.symbols.lookup("print").is_some());
        assert!(tc.symbols.lookup("input").is_some());
        assert!(tc.symbols.lookup("open").is_some());
    }

    #[test]
    fn test_all_type_constructor_builtins() {
        let tc = TypeChecker::new();
        for name in &[
            "int", "float", "bool", "str", "list", "dict", "set", "tuple",
        ] {
            assert!(
                tc.symbols.lookup(name).is_some(),
                "builtin {name} not found"
            );
        }
    }

    #[test]
    fn test_all_numeric_builtins() {
        let tc = TypeChecker::new();
        for name in &["abs", "min", "max", "sum", "round", "pow", "divmod"] {
            assert!(
                tc.symbols.lookup(name).is_some(),
                "builtin {name} not found"
            );
        }
    }

    #[test]
    fn test_all_introspection_builtins() {
        let tc = TypeChecker::new();
        for name in &[
            "type",
            "isinstance",
            "issubclass",
            "hasattr",
            "getattr",
            "setattr",
            "delattr",
        ] {
            assert!(
                tc.symbols.lookup(name).is_some(),
                "builtin {name} not found"
            );
        }
    }
}
