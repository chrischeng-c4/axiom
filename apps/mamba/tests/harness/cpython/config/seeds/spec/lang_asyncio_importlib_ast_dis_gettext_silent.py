# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `asyncio` deep
# surface / `importlib` / `ast.literal_eval` non-int forms /
# `dis.Bytecode` / `gc` referrers / `shlex` quoted parsing /
# `pprint.pformat` width / `gettext` / `locale.getdefaultlocale`
# / `pprint` PrettyPrinter / `reprlib.aRepr` ten-pack pinned to
# atomic 238: `asyncio.Future / Task / Lock / Semaphore /
# Event / Queue / iscoroutine / iscoroutinefunction /
# get_event_loop / new_event_loop / TimeoutError /
# CancelledError` (the documented top-level surface — mamba's
# `asyncio` module dict only exposes the entry-point coroutine
# functions and silently drops every class / type-check), `importlib
# .import_module / reload / invalidate_caches` and `importlib.util
# .find_spec / spec_from_file_location / module_from_spec` (the
# documented import-machinery surface — mamba's `importlib` module
# dict does not expose any of them so `hasattr(...)` collapses to
# False), `ast.literal_eval("[1, 2, 3]")` / `ast.literal_eval(
# "{'a': 1}")` (the documented "round-trips list / dict literals"
# value contract — mamba silently returns None instead of the
# evaluated container), `dis.Bytecode` (the documented top-level
# class surface — mamba's `dis` module dict does not expose the
# class), `gc.get_referrers / get_referents / garbage` (the
# documented module-level surface — mamba's `gc` module dict
# does not expose them), `shlex.split("hello world 'quoted
# string' more")` (the documented "single-quoted span groups
# tokens" value contract — mamba treats the single-quote
# characters as literal token characters so the parsed tokens
# include the quote glyphs), `shlex.shlex` class (the
# documented top-level class — mamba's `shlex` module dict
# does not expose it), `pprint.pformat({"a": [1, 2, 3]},
# width=20)` (the documented "respects width" value contract —
# mamba ignores width and emits a different multi-line form),
# `pprint.PrettyPrinter / isreadable / isrecursive` (the
# documented top-level surface — mamba's `pprint` module dict
# does not expose them), `reprlib.aRepr` (the documented
# module-level instance — mamba does not expose it),
# `gettext.gettext / ngettext / translation / install /
# GNUTranslations / NullTranslations` (the documented
# top-level i18n surface — mamba's `gettext` module dict does
# not expose any of them), and `locale.getdefaultlocale /
# Error` (the documented module-level surface — mamba's
# `locale` module dict does not expose them).
#
# Behavioral edges that CONFORM on mamba (asyncio run/sleep/
# gather/create_task/ensure_future/wait/wait_for; ast parse/
# dump/literal_eval-int/walk + Name/Constant/Module/Expression/
# Call/Assign/FunctionDef/ClassDef/NodeVisitor/NodeTransformer;
# dis dis/Instruction/get_instructions/opname/opmap/
# HAVE_ARGUMENT/code_info/show_code; gc collect/disable/enable/
# isenabled/get_count/get_threshold/set_threshold/get_objects/
# get_stats + collect return; atexit register/unregister;
# signal SIGINT/SIGTERM/SIGKILL/SIGUSR1/SIGALRM/signal/
# getsignal/Signals/Handlers/SIG_DFL/SIG_IGN; shlex
# basic split (no quotes) + quote + join + hasattr split/
# quote/join; pprint pprint/pformat hasattr; reprlib repr/
# Repr/recursive_repr; mimetypes guess_type/guess_extension/
# add_type/init/MimeTypes; locale setlocale/getlocale/LC_*)
# are covered in the matching pass fixture
# `test_asyncio_ast_dis_gc_signal_shlex_pprint_mimetypes_value_ops`.
from typing import Any
import asyncio as _asyncio_mod
import importlib as _importlib_mod
import importlib.util as _importlib_util_mod
import ast as _ast_mod
import dis as _dis_mod
import gc as _gc_mod
import shlex as _shlex_mod
import pprint as _pprint_mod
import reprlib as _reprlib_mod
import gettext as _gettext_mod
import locale as _locale_mod

asyncio_mod: Any = _asyncio_mod
importlib_mod: Any = _importlib_mod
importlib_util_mod: Any = _importlib_util_mod
ast_mod: Any = _ast_mod
dis_mod: Any = _dis_mod
gc_mod: Any = _gc_mod
shlex_mod: Any = _shlex_mod
pprint_mod: Any = _pprint_mod
reprlib_mod: Any = _reprlib_mod
gettext_mod: Any = _gettext_mod
locale_mod: Any = _locale_mod


_ledger: list[int] = []

# 1) asyncio deep surface
#    (mamba: missing — module dict only has run/sleep/gather/create_task/
#    ensure_future/wait/wait_for)
assert hasattr(asyncio_mod, "Future") == True; _ledger.append(1)
assert hasattr(asyncio_mod, "Task") == True; _ledger.append(1)
assert hasattr(asyncio_mod, "Lock") == True; _ledger.append(1)
assert hasattr(asyncio_mod, "Semaphore") == True; _ledger.append(1)
assert hasattr(asyncio_mod, "Event") == True; _ledger.append(1)
assert hasattr(asyncio_mod, "Queue") == True; _ledger.append(1)
assert hasattr(asyncio_mod, "iscoroutine") == True; _ledger.append(1)
assert hasattr(asyncio_mod, "iscoroutinefunction") == True; _ledger.append(1)
assert hasattr(asyncio_mod, "get_event_loop") == True; _ledger.append(1)
assert hasattr(asyncio_mod, "new_event_loop") == True; _ledger.append(1)
assert hasattr(asyncio_mod, "TimeoutError") == True; _ledger.append(1)
assert hasattr(asyncio_mod, "CancelledError") == True; _ledger.append(1)

# 2) importlib import machinery
#    (mamba: missing entirely)
assert hasattr(importlib_mod, "import_module") == True; _ledger.append(1)
assert hasattr(importlib_mod, "reload") == True; _ledger.append(1)
assert hasattr(importlib_mod, "invalidate_caches") == True; _ledger.append(1)
assert hasattr(importlib_util_mod, "find_spec") == True; _ledger.append(1)
assert hasattr(importlib_util_mod, "spec_from_file_location") == True; _ledger.append(1)
assert hasattr(importlib_util_mod, "module_from_spec") == True; _ledger.append(1)

# 3) ast.literal_eval container forms — round-trips list/dict
#    (mamba: silently returns None)
assert ast_mod.literal_eval("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert ast_mod.literal_eval("{'a': 1}") == {"a": 1}; _ledger.append(1)

# 4) dis.Bytecode — top-level class
#    (mamba: missing)
assert hasattr(dis_mod, "Bytecode") == True; _ledger.append(1)

# 5) gc.get_referrers / get_referents / garbage
#    (mamba: missing)
assert hasattr(gc_mod, "get_referrers") == True; _ledger.append(1)
assert hasattr(gc_mod, "get_referents") == True; _ledger.append(1)
assert hasattr(gc_mod, "garbage") == True; _ledger.append(1)

# 6) shlex.split with single-quoted span
#    (mamba: treats single quotes as literal characters)
assert shlex_mod.split("hello world 'quoted string' more") == ["hello", "world", "quoted string", "more"]; _ledger.append(1)

# 7) shlex.shlex class
#    (mamba: missing)
assert hasattr(shlex_mod, "shlex") == True; _ledger.append(1)

# 8) pprint.pformat width semantics + class surface
#    (mamba: width is ignored, PrettyPrinter / isreadable / isrecursive missing)
assert pprint_mod.pformat({"a": [1, 2, 3]}, width=20) == "{'a': [1, 2, 3]}"; _ledger.append(1)
assert hasattr(pprint_mod, "PrettyPrinter") == True; _ledger.append(1)
assert hasattr(pprint_mod, "isreadable") == True; _ledger.append(1)
assert hasattr(pprint_mod, "isrecursive") == True; _ledger.append(1)

# 9) reprlib.aRepr — module-level Repr instance
#    (mamba: missing)
assert hasattr(reprlib_mod, "aRepr") == True; _ledger.append(1)

# 10) gettext — i18n surface
#     (mamba: module dict completely empty for documented surface)
assert hasattr(gettext_mod, "gettext") == True; _ledger.append(1)
assert hasattr(gettext_mod, "ngettext") == True; _ledger.append(1)
assert hasattr(gettext_mod, "translation") == True; _ledger.append(1)
assert hasattr(gettext_mod, "install") == True; _ledger.append(1)
assert hasattr(gettext_mod, "GNUTranslations") == True; _ledger.append(1)
assert hasattr(gettext_mod, "NullTranslations") == True; _ledger.append(1)

# 11) locale.getdefaultlocale / Error
#     (mamba: missing)
assert hasattr(locale_mod, "getdefaultlocale") == True; _ledger.append(1)
assert hasattr(locale_mod, "Error") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_asyncio_importlib_ast_dis_gettext_silent {sum(_ledger)} asserts")
