"""Surface contract for third-party certifi package.

# type-regime: monomorphic

Probes: certifi.where, certifi.contents, certifi.core.
CPython 3.12 is the oracle.
"""

import certifi
import os

# Core attributes
assert hasattr(certifi, "where"), "where"
assert hasattr(certifi, "contents"), "contents"
assert hasattr(certifi, "core"), "core"

# where() returns the path to the CA bundle file
_path = certifi.where()
assert isinstance(_path, str), f"where() returns str: {type(_path)!r}"
assert _path.endswith(".pem") or _path.endswith(".crt"), \
    f"where() ends with .pem or .crt: {_path!r}"
assert os.path.exists(_path), f"CA bundle exists: {_path!r}"

# contents() returns the CA bundle as str
_contents = certifi.contents()
assert isinstance(_contents, str), f"contents() returns str: {type(_contents)!r}"
assert len(_contents) > 0, "contents not empty"
assert "-----BEGIN CERTIFICATE-----" in _contents, "PEM header in contents"

# core is the internal module
import types
assert isinstance(certifi.core, types.ModuleType), \
    f"core is a module: {type(certifi.core)!r}"

# Module-attribute stability (benched in hot loop)
_where1 = certifi.where
_where2 = certifi.where
assert _where1 is _where2, "where attribute is stable"

_contents1 = certifi.contents
_contents2 = certifi.contents
assert _contents1 is _contents2, "contents attribute is stable"

_core1 = certifi.core
_core2 = certifi.core
assert _core1 is _core2, "core attribute is stable"

print("surface OK")
