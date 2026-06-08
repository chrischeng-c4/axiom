# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_csv_logging_mimetypes_html_ast_dis_token_value_ops"
# subject = "cpython321.test_csv_logging_mimetypes_html_ast_dis_token_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_csv_logging_mimetypes_html_ast_dis_token_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_csv_logging_mimetypes_html_ast_dis_token_value_ops: execute CPython 3.12 seed test_csv_logging_mimetypes_html_ast_dis_token_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 229 pass conformance — csv/logging/mimetypes/html/ast/dis/token/
# keyword/subprocess/asyncio hasattr + value ops that match between
# CPython 3.12 and mamba.
import csv
import logging
import mimetypes
import html
import ast
import dis
import token
import keyword
import subprocess
import asyncio

_ledger: list[int] = []

# 1) csv — core surface hasattr
assert hasattr(csv, "reader") == True; _ledger.append(1)
assert hasattr(csv, "writer") == True; _ledger.append(1)
assert hasattr(csv, "DictReader") == True; _ledger.append(1)
assert hasattr(csv, "DictWriter") == True; _ledger.append(1)
assert hasattr(csv, "Dialect") == True; _ledger.append(1)
assert hasattr(csv, "excel") == True; _ledger.append(1)
assert hasattr(csv, "excel_tab") == True; _ledger.append(1)
assert hasattr(csv, "unix_dialect") == True; _ledger.append(1)
assert hasattr(csv, "register_dialect") == True; _ledger.append(1)
assert hasattr(csv, "unregister_dialect") == True; _ledger.append(1)
assert hasattr(csv, "get_dialect") == True; _ledger.append(1)
assert hasattr(csv, "list_dialects") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_ALL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_MINIMAL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONNUMERIC") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONE") == True; _ledger.append(1)
assert hasattr(csv, "Error") == True; _ledger.append(1)

# 2) logging — common surface hasattr
assert hasattr(logging, "getLogger") == True; _ledger.append(1)
assert hasattr(logging, "basicConfig") == True; _ledger.append(1)
assert hasattr(logging, "DEBUG") == True; _ledger.append(1)
assert hasattr(logging, "INFO") == True; _ledger.append(1)
assert hasattr(logging, "WARNING") == True; _ledger.append(1)
assert hasattr(logging, "ERROR") == True; _ledger.append(1)
assert hasattr(logging, "CRITICAL") == True; _ledger.append(1)
assert hasattr(logging, "debug") == True; _ledger.append(1)
assert hasattr(logging, "info") == True; _ledger.append(1)
assert hasattr(logging, "warning") == True; _ledger.append(1)
assert hasattr(logging, "error") == True; _ledger.append(1)
assert hasattr(logging, "critical") == True; _ledger.append(1)

# 3) mimetypes — core surface + value-op
assert hasattr(mimetypes, "guess_type") == True; _ledger.append(1)
assert hasattr(mimetypes, "guess_extension") == True; _ledger.append(1)
assert hasattr(mimetypes, "guess_all_extensions") == True; _ledger.append(1)
assert hasattr(mimetypes, "add_type") == True; _ledger.append(1)
assert hasattr(mimetypes, "init") == True; _ledger.append(1)
assert hasattr(mimetypes, "MimeTypes") == True; _ledger.append(1)
assert hasattr(mimetypes, "knownfiles") == True; _ledger.append(1)
assert hasattr(mimetypes, "inited") == True; _ledger.append(1)
assert hasattr(mimetypes, "suffix_map") == True; _ledger.append(1)
assert hasattr(mimetypes, "encodings_map") == True; _ledger.append(1)
assert hasattr(mimetypes, "types_map") == True; _ledger.append(1)
assert hasattr(mimetypes, "common_types") == True; _ledger.append(1)
assert mimetypes.guess_type("foo.html") == ("text/html", None); _ledger.append(1)

# 4) html — escape/unescape value ops
assert hasattr(html, "escape") == True; _ledger.append(1)
assert hasattr(html, "unescape") == True; _ledger.append(1)
assert html.escape("<b>a</b>") == "&lt;b&gt;a&lt;/b&gt;"; _ledger.append(1)
assert html.unescape("&lt;b&gt;") == "<b>"; _ledger.append(1)
assert html.escape("a & b") == "a &amp; b"; _ledger.append(1)
assert html.unescape("a &amp; b") == "a & b"; _ledger.append(1)

# 5) ast — common surface hasattr
assert hasattr(ast, "parse") == True; _ledger.append(1)
assert hasattr(ast, "literal_eval") == True; _ledger.append(1)
assert hasattr(ast, "dump") == True; _ledger.append(1)
assert hasattr(ast, "walk") == True; _ledger.append(1)
assert hasattr(ast, "unparse") == True; _ledger.append(1)
assert hasattr(ast, "fix_missing_locations") == True; _ledger.append(1)
assert hasattr(ast, "increment_lineno") == True; _ledger.append(1)
assert hasattr(ast, "get_docstring") == True; _ledger.append(1)
assert hasattr(ast, "NodeVisitor") == True; _ledger.append(1)
assert hasattr(ast, "NodeTransformer") == True; _ledger.append(1)
assert hasattr(ast, "Module") == True; _ledger.append(1)
assert hasattr(ast, "Expression") == True; _ledger.append(1)
assert hasattr(ast, "Constant") == True; _ledger.append(1)
assert hasattr(ast, "Name") == True; _ledger.append(1)
assert hasattr(ast, "BinOp") == True; _ledger.append(1)
assert hasattr(ast, "Call") == True; _ledger.append(1)
assert hasattr(ast, "Assign") == True; _ledger.append(1)
assert hasattr(ast, "PyCF_ONLY_AST") == True; _ledger.append(1)
assert hasattr(ast, "PyCF_TYPE_COMMENTS") == True; _ledger.append(1)
assert hasattr(ast, "PyCF_ALLOW_TOP_LEVEL_AWAIT") == True; _ledger.append(1)

# 6) dis — common surface hasattr
assert hasattr(dis, "dis") == True; _ledger.append(1)
assert hasattr(dis, "Instruction") == True; _ledger.append(1)
assert hasattr(dis, "get_instructions") == True; _ledger.append(1)
assert hasattr(dis, "code_info") == True; _ledger.append(1)
assert hasattr(dis, "show_code") == True; _ledger.append(1)
assert hasattr(dis, "findlinestarts") == True; _ledger.append(1)
assert hasattr(dis, "findlabels") == True; _ledger.append(1)
assert hasattr(dis, "opname") == True; _ledger.append(1)
assert hasattr(dis, "opmap") == True; _ledger.append(1)
assert hasattr(dis, "stack_effect") == True; _ledger.append(1)
assert hasattr(dis, "HAVE_ARGUMENT") == True; _ledger.append(1)

# 7) token — common surface hasattr
assert hasattr(token, "NAME") == True; _ledger.append(1)
assert hasattr(token, "NUMBER") == True; _ledger.append(1)
assert hasattr(token, "STRING") == True; _ledger.append(1)
assert hasattr(token, "OP") == True; _ledger.append(1)
assert hasattr(token, "NEWLINE") == True; _ledger.append(1)
assert hasattr(token, "INDENT") == True; _ledger.append(1)
assert hasattr(token, "DEDENT") == True; _ledger.append(1)
assert hasattr(token, "ENDMARKER") == True; _ledger.append(1)
assert hasattr(token, "tok_name") == True; _ledger.append(1)
assert hasattr(token, "ENCODING") == True; _ledger.append(1)
assert hasattr(token, "ERRORTOKEN") == True; _ledger.append(1)
assert hasattr(token, "COMMENT") == True; _ledger.append(1)
assert hasattr(token, "NL") == True; _ledger.append(1)
assert hasattr(token, "ISTERMINAL") == True; _ledger.append(1)
assert hasattr(token, "ISNONTERMINAL") == True; _ledger.append(1)
assert hasattr(token, "ISEOF") == True; _ledger.append(1)

# 8) keyword — full surface + value ops
assert hasattr(keyword, "iskeyword") == True; _ledger.append(1)
assert hasattr(keyword, "issoftkeyword") == True; _ledger.append(1)
assert hasattr(keyword, "kwlist") == True; _ledger.append(1)
assert hasattr(keyword, "softkwlist") == True; _ledger.append(1)
assert keyword.iskeyword("if") == True; _ledger.append(1)
assert keyword.iskeyword("for") == True; _ledger.append(1)
assert keyword.iskeyword("foo") == False; _ledger.append(1)
assert len(keyword.kwlist) == 35; _ledger.append(1)

# 9) subprocess — full hasattr surface
assert hasattr(subprocess, "run") == True; _ledger.append(1)
assert hasattr(subprocess, "Popen") == True; _ledger.append(1)
assert hasattr(subprocess, "call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_call") == True; _ledger.append(1)
assert hasattr(subprocess, "check_output") == True; _ledger.append(1)
assert hasattr(subprocess, "PIPE") == True; _ledger.append(1)
assert hasattr(subprocess, "STDOUT") == True; _ledger.append(1)
assert hasattr(subprocess, "DEVNULL") == True; _ledger.append(1)
assert hasattr(subprocess, "TimeoutExpired") == True; _ledger.append(1)
assert hasattr(subprocess, "CalledProcessError") == True; _ledger.append(1)
assert hasattr(subprocess, "CompletedProcess") == True; _ledger.append(1)

# 10) asyncio — top-level coroutine surface hasattr
assert hasattr(asyncio, "run") == True; _ledger.append(1)
assert hasattr(asyncio, "sleep") == True; _ledger.append(1)
assert hasattr(asyncio, "gather") == True; _ledger.append(1)
assert hasattr(asyncio, "wait") == True; _ledger.append(1)
assert hasattr(asyncio, "wait_for") == True; _ledger.append(1)
assert hasattr(asyncio, "ensure_future") == True; _ledger.append(1)
assert hasattr(asyncio, "create_task") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_csv_logging_mimetypes_html_ast_dis_token_value_ops {sum(_ledger)} asserts")
