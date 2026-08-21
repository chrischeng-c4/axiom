"""Behavior contract for third-party certifi package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import certifi
import os

# Rule 1: where() returns a valid file path to a PEM file
_path1 = certifi.where()
assert isinstance(_path1, str), f"where() is str: {type(_path1)!r}"
assert os.path.isfile(_path1), f"CA bundle file exists: {_path1!r}"
assert _path1.endswith(".pem") or _path1.endswith(".crt"), \
    f"PEM/CRT extension: {_path1!r}"

# Rule 2: where() returns the same path on repeated calls
_path2a = certifi.where()
_path2b = certifi.where()
assert _path2a == _path2b, f"where() stable: {_path2a!r} vs {_path2b!r}"

# Rule 3: contents() reads the CA bundle as str
_str3 = certifi.contents()
assert isinstance(_str3, str), f"contents() is str: {type(_str3)!r}"
assert len(_str3) > 10_000, f"CA bundle non-trivially sized: {len(_str3)!r}"
assert "-----BEGIN CERTIFICATE-----" in _str3, "PEM header in contents"

# Rule 4: contents() str matches file contents at where() path
_file_str = open(certifi.where(), "r", encoding="ascii").read()
assert certifi.contents() == _file_str, "contents() matches file on disk"

# Rule 5: Module attributes are identity-stable across repeated reads
_where_ref = certifi.where
assert certifi.where is _where_ref, "where stable identity"
_contents_ref = certifi.contents
assert certifi.contents is _contents_ref, "contents stable identity"
_core_ref = certifi.core
assert certifi.core is _core_ref, "core stable identity"

# Rule 6: certifi.core exposes where and contents
assert hasattr(certifi.core, "where"), "core.where"
assert hasattr(certifi.core, "contents"), "core.contents"
assert certifi.core.where() == certifi.where(), "core.where() matches certifi.where()"

print("behavior OK")
