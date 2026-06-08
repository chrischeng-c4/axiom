# Atomic 238 pass conformance — asyncio / ast / dis / gc / atexit / signal /
# shlex / pprint / reprlib / mimetypes / locale surface + value ops that
# match between CPython 3.12 and mamba.
import asyncio
import ast
import dis
import gc
import atexit
import signal
import shlex
import pprint
import reprlib
import mimetypes
import locale


_ledger: list[int] = []

# 1) asyncio partial surface — entry points that exist on mamba
assert hasattr(asyncio, "run") == True; _ledger.append(1)
assert hasattr(asyncio, "sleep") == True; _ledger.append(1)
assert hasattr(asyncio, "gather") == True; _ledger.append(1)
assert hasattr(asyncio, "create_task") == True; _ledger.append(1)
assert hasattr(asyncio, "ensure_future") == True; _ledger.append(1)
assert hasattr(asyncio, "wait") == True; _ledger.append(1)
assert hasattr(asyncio, "wait_for") == True; _ledger.append(1)

# 2) ast surface + literal_eval int conform
assert hasattr(ast, "parse") == True; _ledger.append(1)
assert hasattr(ast, "dump") == True; _ledger.append(1)
assert hasattr(ast, "literal_eval") == True; _ledger.append(1)
assert hasattr(ast, "walk") == True; _ledger.append(1)
assert hasattr(ast, "Name") == True; _ledger.append(1)
assert hasattr(ast, "Constant") == True; _ledger.append(1)
assert hasattr(ast, "Module") == True; _ledger.append(1)
assert hasattr(ast, "Expression") == True; _ledger.append(1)
assert hasattr(ast, "Call") == True; _ledger.append(1)
assert hasattr(ast, "Assign") == True; _ledger.append(1)
assert hasattr(ast, "FunctionDef") == True; _ledger.append(1)
assert hasattr(ast, "ClassDef") == True; _ledger.append(1)
assert hasattr(ast, "NodeVisitor") == True; _ledger.append(1)
assert hasattr(ast, "NodeTransformer") == True; _ledger.append(1)
assert ast.literal_eval("42") == 42; _ledger.append(1)

# 3) dis surface
assert hasattr(dis, "dis") == True; _ledger.append(1)
assert hasattr(dis, "Instruction") == True; _ledger.append(1)
assert hasattr(dis, "get_instructions") == True; _ledger.append(1)
assert hasattr(dis, "opname") == True; _ledger.append(1)
assert hasattr(dis, "opmap") == True; _ledger.append(1)
assert hasattr(dis, "HAVE_ARGUMENT") == True; _ledger.append(1)
assert hasattr(dis, "code_info") == True; _ledger.append(1)
assert hasattr(dis, "show_code") == True; _ledger.append(1)

# 4) gc surface + collect value op
assert hasattr(gc, "collect") == True; _ledger.append(1)
assert hasattr(gc, "disable") == True; _ledger.append(1)
assert hasattr(gc, "enable") == True; _ledger.append(1)
assert hasattr(gc, "isenabled") == True; _ledger.append(1)
assert hasattr(gc, "get_count") == True; _ledger.append(1)
assert hasattr(gc, "get_threshold") == True; _ledger.append(1)
assert hasattr(gc, "set_threshold") == True; _ledger.append(1)
assert hasattr(gc, "get_objects") == True; _ledger.append(1)
assert hasattr(gc, "get_stats") == True; _ledger.append(1)
assert type(gc.collect()).__name__ == "int"; _ledger.append(1)

# 5) atexit surface
assert hasattr(atexit, "register") == True; _ledger.append(1)
assert hasattr(atexit, "unregister") == True; _ledger.append(1)

# 6) signal full surface
assert hasattr(signal, "SIGINT") == True; _ledger.append(1)
assert hasattr(signal, "SIGTERM") == True; _ledger.append(1)
assert hasattr(signal, "SIGKILL") == True; _ledger.append(1)
assert hasattr(signal, "SIGUSR1") == True; _ledger.append(1)
assert hasattr(signal, "SIGALRM") == True; _ledger.append(1)
assert hasattr(signal, "signal") == True; _ledger.append(1)
assert hasattr(signal, "getsignal") == True; _ledger.append(1)
assert hasattr(signal, "Signals") == True; _ledger.append(1)
assert hasattr(signal, "Handlers") == True; _ledger.append(1)
assert hasattr(signal, "SIG_DFL") == True; _ledger.append(1)
assert hasattr(signal, "SIG_IGN") == True; _ledger.append(1)

# 7) shlex value ops without single-quoted parsing (see spec fixture for
#    the quoted-string divergence)
assert shlex.split("hello world more") == ["hello", "world", "more"]; _ledger.append(1)
assert shlex.split("a b c") == ["a", "b", "c"]; _ledger.append(1)
assert shlex.quote("hello") == "hello"; _ledger.append(1)
assert shlex.quote("hello world") == "'hello world'"; _ledger.append(1)
assert shlex.join(["a", "b", "c"]) == "a b c"; _ledger.append(1)
assert shlex.join(["hello world", "more"]) == "'hello world' more"; _ledger.append(1)
assert hasattr(shlex, "split") == True; _ledger.append(1)
assert hasattr(shlex, "quote") == True; _ledger.append(1)
assert hasattr(shlex, "join") == True; _ledger.append(1)

# 8) pprint partial surface
assert hasattr(pprint, "pprint") == True; _ledger.append(1)
assert hasattr(pprint, "pformat") == True; _ledger.append(1)

# 9) reprlib partial surface + truncating repr
assert reprlib.repr(list(range(100))) == "[0, 1, 2, 3, 4, 5, ...]"; _ledger.append(1)
assert hasattr(reprlib, "Repr") == True; _ledger.append(1)
assert hasattr(reprlib, "repr") == True; _ledger.append(1)
assert hasattr(reprlib, "recursive_repr") == True; _ledger.append(1)

# 10) mimetypes full surface
assert hasattr(mimetypes, "guess_type") == True; _ledger.append(1)
assert hasattr(mimetypes, "guess_extension") == True; _ledger.append(1)
assert hasattr(mimetypes, "add_type") == True; _ledger.append(1)
assert hasattr(mimetypes, "init") == True; _ledger.append(1)
assert hasattr(mimetypes, "MimeTypes") == True; _ledger.append(1)

# 11) locale partial surface
assert hasattr(locale, "setlocale") == True; _ledger.append(1)
assert hasattr(locale, "getlocale") == True; _ledger.append(1)
assert hasattr(locale, "LC_ALL") == True; _ledger.append(1)
assert hasattr(locale, "LC_CTYPE") == True; _ledger.append(1)
assert hasattr(locale, "LC_NUMERIC") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_asyncio_ast_dis_gc_signal_shlex_pprint_mimetypes_value_ops {sum(_ledger)} asserts")
