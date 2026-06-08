# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `pathlib.PurePath('/a/b').name == 'b'`
# (the documented "PurePath.name is the final path component" — mamba
# returns None — attribute resolves to None placeholder),
# `str(pathlib.PurePath('/a/b').parent) == '/a'` (the documented
# "PurePath.parent strips the final component" — mamba returns 'None'
# — parent attribute resolves to None), `pathlib.PurePath('/a/b.txt').
# suffix == '.txt'` (the documented "PurePath.suffix is the trailing
# extension" — mamba returns None — attribute resolves to None
# placeholder), `pathlib.PurePath('/a/b').parts == ('/', 'a', 'b')`
# (the documented "PurePath.parts splits into a tuple of components"
# — mamba returns None — attribute resolves to None placeholder),
# `pathlib.PurePath('/a/b.txt').stem == 'b'` (the documented
# "PurePath.stem is name without the suffix" — mamba returns None),
# `pathlib.PurePath('/a/b').anchor == '/'` (the documented
# "PurePath.anchor is the root + drive prefix" — mamba returns None),
# `str(pathlib.PurePath('/a') / 'b') == '/a/b'` (the documented
# "PurePath supports `/` join" — mamba returns 'None' — operator
# yields None placeholder), `str(pathlib.PurePath('/a/b')) ==
# '/a/b'` (the documented "str(PurePath) renders the path string" —
# mamba returns '<PurePosixPath instance>' — repr-style placeholder),
# `hasattr(binascii, 'crc32')` (the documented "binascii exposes
# crc32" — mamba returns False), and `hasattr(binascii, 'Error')`
# (the documented "binascii exposes the Error exception class" —
# mamba returns False).
# Ten-pack pinned to atomic 299.
#
# Behavioral edges that CONFORM on mamba (pathlib — hasattr Path/
# PurePath/PurePosixPath/PureWindowsPath + type Path/PurePath.
# tempfile — hasattr full surface + gettempdir str/absolute. glob —
# hasattr full + escape. fnmatch — hasattr full + fnmatch/filter/
# translate. mimetypes — hasattr full + guess_type for html/txt/png.
# quopri — hasattr full + encodestring/decodestring. binascii —
# hasattr hexlify/unhexlify/a2b_hex/b2a_hex/a2b_base64/b2a_base64 +
# hexlify/unhexlify round-trip) are covered in the matching pass
# fixture `test_pathlib_tempfile_glob_fnmatch_value_ops`.
import pathlib
import binascii


_ledger: list[int] = []

# 1) PurePath('/a/b').name == 'b' — final path component
#    (mamba: returns None — attribute resolves to None placeholder)
assert pathlib.PurePath("/a/b").name == "b"; _ledger.append(1)

# 2) str(PurePath('/a/b').parent) == '/a' — parent strips final component
#    (mamba: returns 'None' — parent attribute resolves to None)
assert str(pathlib.PurePath("/a/b").parent) == "/a"; _ledger.append(1)

# 3) PurePath('/a/b.txt').suffix == '.txt' — trailing extension
#    (mamba: returns None — attribute resolves to None placeholder)
assert pathlib.PurePath("/a/b.txt").suffix == ".txt"; _ledger.append(1)

# 4) PurePath('/a/b').parts == ('/', 'a', 'b') — tuple of components
#    (mamba: returns None — attribute resolves to None placeholder)
assert pathlib.PurePath("/a/b").parts == ("/", "a", "b"); _ledger.append(1)

# 5) PurePath('/a/b.txt').stem == 'b' — name without suffix
#    (mamba: returns None — attribute resolves to None placeholder)
assert pathlib.PurePath("/a/b.txt").stem == "b"; _ledger.append(1)

# 6) PurePath('/a/b').anchor == '/' — root + drive prefix
#    (mamba: returns None — attribute resolves to None placeholder)
assert pathlib.PurePath("/a/b").anchor == "/"; _ledger.append(1)

# 7) str(PurePath('/a') / 'b') == '/a/b' — `/` join operator
#    (mamba: returns 'None' — operator yields None placeholder)
assert str(pathlib.PurePath("/a") / "b") == "/a/b"; _ledger.append(1)

# 8) str(PurePath('/a/b')) == '/a/b' — str renders the path
#    (mamba: returns '<PurePosixPath instance>' — repr-style placeholder)
assert str(pathlib.PurePath("/a/b")) == "/a/b"; _ledger.append(1)

# 9) hasattr(binascii, 'crc32') — crc32 function
#    (mamba: returns False)
assert hasattr(binascii, "crc32") == True; _ledger.append(1)

# 10) hasattr(binascii, 'Error') — Error exception class
#     (mamba: returns False)
assert hasattr(binascii, "Error") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pathlib_binascii_silent {sum(_ledger)} asserts")
