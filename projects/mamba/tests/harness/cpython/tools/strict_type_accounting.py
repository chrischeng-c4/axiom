#!/usr/bin/env python3.12
"""Strict-type replacement accounting for #704.

This is the machine-readable bridge between the typeshed-derived type wall,
the executable type fixtures, and declared behavior-denominator exclusions.
It is intentionally strict: sampled runs are useful development evidence, but
only a full run can go green.
"""

from __future__ import annotations

import argparse
import ast
import io
import importlib.util
import json
import os
import re
import shlex
import subprocess
import sys
import tokenize
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from functools import cache
from pathlib import Path
from typing import Any

import harness_lib
from type_wall_gen import SENTINEL, WRONG_VALUE


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]
REPO_ROOT = MAMBA_DIR.parents[1]
FIXTURES_DIR = MAMBA_DIR / "tests" / "cpython"
TYPE_DIR = FIXTURES_DIR / "type"
SOUND_DIR = FIXTURES_DIR / "behavior" / "core"
GENERATED_SPECS = MAMBA_DIR / "src" / "types" / "stdlib_specs_generated.json"
DEFAULT_TYPESHED_STDLIB = MAMBA_DIR / "vendor" / "typeshed" / "stdlib"
TYPE_DIVERGENCES = TOOLS_DIR.parent / "config" / "type_divergences.txt"

EXIT_NOT_READY = 70
EXPECTED_PYTHON_VERSION = (3, 12)
NON_RUNTIME_STUB_TYPE_LIB_PREFIXES = ("_typeshed",)
NON_STDLIB_BACKPORT_TYPE_LIBS = {"typing_extensions"}
PLATFORM_SPECIFIC_TYPE_LIBS = {
    "_winapi": "win32",
    "msilib": "win32",
    "ossaudiodev": ("linux", "freebsd"),
}
OPTIONAL_STDLIB_EXTENSION_TYPE_LIBS = {
    "_tkinter": "_tkinter",
    "tkinter": "_tkinter",
    "tkinter_colorchooser": "_tkinter",
    "tkinter_commondialog": "_tkinter",
    "tkinter_dialog": "_tkinter",
    "tkinter_dnd": "_tkinter",
    "tkinter_filedialog": "_tkinter",
    "tkinter_font": "_tkinter",
    "tkinter_messagebox": "_tkinter",
    "tkinter_scrolledtext": "_tkinter",
    "tkinter_simpledialog": "_tkinter",
    "tkinter_tix": "_tkinter",
    "tkinter_ttk": "_tkinter",
    "turtle": "_tkinter",
}
VERSION_SPECIFIC_TYPE_LIBS = {
    "_zstd": (3, 14),
    "annotationlib": (3, 14),
    "asyncio_graph": (3, 14),
    "asyncio_tools": (3, 14),
    "compression_zstd": (3, 14),
    "compression_zstd__zstdfile": (3, 14),
    "concurrent_interpreters": (3, 14),
    "concurrent_interpreters__crossinterp": (3, 14),
    "concurrent_interpreters__queues": (3, 14),
}
VERSION_REMOVED_TYPE_LIBS = {
    "asyncore": (3, 12),
    "asynchat": (3, 12),
    "smtpd": (3, 12),
}
VERSION_SPECIFIC_TYPE_FIXTURES = {
    "std-libs/ast/Interpolation__init__value_as_expr_wrong.py": (3, 14),
    "std-libs/ast/TemplateStr__init__values_as_list_wrong.py": (3, 14),
    "std-libs/base64/z85decode__s_as_typed_wrong.py": (3, 13),
    "std-libs/base64/z85encode__s_as_ReadableBuffer_wrong.py": (3, 13),
    "std-libs/datetime/date__strptime__date_string_as_str_wrong.py": (3, 14),
    "std-libs/datetime/date__strptime__string_as_str_wrong.py": (3, 14),
    "std-libs/datetime/time__strptime__date_string_as_str_wrong.py": (3, 14),
    "std-libs/datetime/time__strptime__string_as_str_wrong.py": (3, 14),
    "std-libs/imaplib/Idler____exit____exc_val_as_Unused_wrong.py": (3, 14),
    "std-libs/imaplib/Idler__burst__interval_as_float_wrong.py": (3, 14),
    "std-libs/imaplib/Idler__init__imap_as_IMAP4_wrong.py": (3, 14),
}
VERSION_REMOVED_TYPE_FIXTURES = {
    "std-libs/asyncio_coroutines/coroutine__func_as__FunctionT_wrong.py": (3, 12),
    "std-libs/importlib_metadata/Deprecated____getitem____name_as__KT_wrong.py": (3, 12),
    "std-libs/importlib_metadata/Deprecated__get__name_as__KT_wrong.py": (3, 12),
    "std-libs/importlib_metadata/SelectableGroups__load__eps_as_Iterable_wrong.py": (3, 12),
    "std-libs/importlib_readers/remove_duplicates__items_as_Iterable_wrong.py": (3, 12),
    "std-libs/importlib_resources__common/get_package__package_as_Package_wrong.py": (3, 12),
}

SOUND_FAMILIES = [
    "float_return_inference",
    "builtin_numeric_inference",
    "comprehension_float_inference",
    "generator_float_inference",
    "container_float_roundtrip",
    "value_equality_inference",
    "mixed_numeric_inference",
]

BEHAVIOR_FACETS = {"behavior", "surface", "errors", "real_world"}

TYPE_REJECTION_MARKERS = (
    "type mismatch",
    "type error",
    "requires numeric type",
    "requires numeric types",
    "require int types",
    "TypeError:",
)
NON_TYPE_REJECTION_MARKERS = (
    "undefined name",
    "unknown type",
    "unknown generic type",
)
TYPE_FIXTURE_SUBJECT_RE = re.compile(r'#\s*subject\s*=\s*"(?P<subject>[^"]+)"')
TYPE_FIXTURE_SUBJECT_CALL_RE = re.compile(
    r"^(?P<call>.+)\((?P<param>[^():,\s]+):\s*(?P<label>[^()]*)\)$"
)
TYPE_FIXTURE_SUBJECT_PROPERTY_SET_RE = re.compile(
    r"^(?P<call>.+)\s*=\s*\((?P<param>[^():,\s]+):\s*(?P<label>[^()]*)\)$"
)
GENERATED_SIG_BLOCK_RE = re.compile(r"StdlibSig\s*\{\n(?P<body>.*?)\n    \},", re.S)
GENERATED_SIG_FIELD_RE = re.compile(r'\b(?P<field>module|qualifier|name):\s*"(?P<value>[^"]*)"')
GENERATED_PARAM_RE = re.compile(
    r'p\("(?P<name>[^"]+)",\s*CoreTy::(?P<ty>\w+),\s*(?P<star>true|false)\)'
)
GENERATED_ENFORCEABLE_RE = re.compile(r"\benforceable:\s*(?P<value>true|false)")
TYPEVAR_STAYS_UNWALLED_MARKER = "TypeVar param must stay unwalled"


@dataclass
class Divergence:
    path: str
    owner_refs: list[str]


@dataclass(frozen=True)
class FixtureCallShape:
    access: str
    positional_count: int
    keyword_names: tuple[str, ...]
    target: tuple[str, int | str]
    subject_param: str


def default_mamba_bin() -> str:
    if env := os.environ.get("MAMBA_BIN"):
        return env
    debug = (MAMBA_DIR / "../../target/debug/mamba").resolve()
    if debug.exists():
        return str(debug)
    release = (MAMBA_DIR / "../../target/release/mamba").resolve()
    if release.exists():
        return str(release)
    return "mamba"


def selected(paths: list[Path], limit: int) -> tuple[list[Path], bool]:
    if limit <= 0 or len(paths) <= limit:
        return paths, False
    step = max(1, len(paths) // limit)
    return paths[::step][:limit], True


def repo_rel(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def is_non_runtime_stub_type_fixture(path: Path) -> bool:
    lib = type_fixture_lib(path)
    if lib is None:
        return False
    return any(
        lib == prefix or lib.startswith(f"{prefix}_")
        for prefix in NON_RUNTIME_STUB_TYPE_LIB_PREFIXES
    )


def is_non_stdlib_backport_type_fixture(path: Path) -> bool:
    lib = type_fixture_lib(path)
    return lib in NON_STDLIB_BACKPORT_TYPE_LIBS


def type_fixture_lib(path: Path) -> str | None:
    try:
        rel = path.relative_to(TYPE_DIR).parts
    except ValueError:
        return None
    if len(rel) < 3 or rel[0] != "std-libs":
        return None
    return rel[1]


def is_platform_specific_unavailable_type_fixture(path: Path) -> bool:
    lib = type_fixture_lib(path)
    if lib is None:
        return False
    required = PLATFORM_SPECIFIC_TYPE_LIBS.get(lib)
    if required is None:
        return False
    if isinstance(required, str):
        required = (required,)
    return not any(
        sys.platform == item or sys.platform.startswith(item) for item in required
    )


def is_optional_stdlib_extension_unavailable_type_fixture(path: Path) -> bool:
    lib = type_fixture_lib(path)
    if lib is None:
        return False
    module = OPTIONAL_STDLIB_EXTENSION_TYPE_LIBS.get(lib)
    return module is not None and importlib.util.find_spec(module) is None


def is_version_specific_unavailable_type_fixture(path: Path) -> bool:
    try:
        rel = "/".join(path.relative_to(TYPE_DIR).parts)
    except ValueError:
        rel = ""
    required = VERSION_SPECIFIC_TYPE_FIXTURES.get(rel)
    if required is not None:
        return sys.version_info[:2] < required
    removed = VERSION_REMOVED_TYPE_FIXTURES.get(rel)
    if removed is not None:
        return sys.version_info[:2] >= removed
    lib = type_fixture_lib(path)
    if lib is None:
        return False
    required = VERSION_SPECIFIC_TYPE_LIBS.get(lib)
    if required is not None and sys.version_info[:2] < required:
        return True
    removed = VERSION_REMOVED_TYPE_LIBS.get(lib)
    return removed is not None and sys.version_info[:2] >= removed


def is_generated_typespec_inactive_type_fixture(path: Path) -> bool:
    if not any(
        (TYPE_DIR / bucket) in path.parents
        for bucket in ("builtin-libs", "std-libs")
    ):
        return False
    parsed = parse_type_fixture_contract(path)
    if parsed is None:
        return False
    branches = generated_callable_branch_index()
    return _generated_contract_is_inactive(parsed[0], parsed[2], branches)


def is_excluded_type_fixture(path: Path) -> bool:
    return (
        is_non_runtime_stub_type_fixture(path)
        or is_non_stdlib_backport_type_fixture(path)
        or is_platform_specific_unavailable_type_fixture(path)
        or is_optional_stdlib_extension_unavailable_type_fixture(path)
        or is_version_specific_unavailable_type_fixture(path)
        or is_generated_typespec_inactive_type_fixture(path)
    )


def executable_type_fixtures(paths: list[Path]) -> list[Path]:
    return [path for path in paths if not is_excluded_type_fixture(path)]


def load_generated_typespec_manifest() -> dict[str, Any]:
    manifest = json.loads(GENERATED_SPECS.read_text(encoding="utf-8"))
    if manifest.get("schema") != 2:
        raise ValueError(f"unsupported generated TypeSpec schema: {manifest.get('schema')!r}")
    return manifest


def _expand_generated_callable_exports(
    manifest: dict[str, Any], index: dict[tuple[str, str, str], Any]
) -> None:
    strings = manifest["strings"]
    for export in manifest.get("callable_exports", []):
        alias = (strings[export[0]], "", strings[export[1]])
        target = (strings[export[2]], "", strings[export[3]])
        if target in index:
            index[alias] = index[target]

    methods_by_owner: dict[tuple[str, str], list[tuple[str, Any]]] = {}
    for (module, qualifier, name), value in list(index.items()):
        if qualifier:
            methods_by_owner.setdefault((module, qualifier), []).append((name, value))
    for export in manifest.get("class_exports", []):
        class_row = manifest["classes"][export[2]]
        owner = (strings[class_row[0]], strings[class_row[1]])
        for name, value in methods_by_owner.get(owner, []):
            index[(strings[export[0]], strings[export[1]], name)] = value


def _generated_callable_rows(
    manifest: dict[str, Any], *, include_class_inventory: bool
) -> list[list[Any]]:
    rows = list(manifest["callables"])
    if not include_class_inventory:
        return rows
    strings = manifest["strings"]
    public_keys = {
        tuple(strings[row[index]] for index in range(3)) for row in rows
    }
    rows.extend(
        row
        for row in manifest.get("class_callables", [])
        if tuple(strings[row[index]] for index in range(3)) not in public_keys
    )
    return rows


@cache
def generated_callable_branch_index() -> dict[tuple[str, str, str], dict[str, bool]]:
    manifest = load_generated_typespec_manifest()
    strings = manifest["strings"]
    states: dict[tuple[str, str, str], dict[str, list[bool]]] = {}
    for row in _generated_callable_rows(manifest, include_class_inventory=True):
        key = tuple(strings[row[i]] for i in range(3))
        states.setdefault(key, {}).setdefault(row[3], []).append(bool(row[12]))
    _expand_generated_callable_exports(manifest, states)
    return {
        key: {
            kind: any(kind_branches)
            for kind, kind_branches in branches.items()
        }
        for key, branches in states.items()
    }


def _generated_contract_is_inactive(
    call: str,
    binding: str,
    branches: dict[tuple[str, str, str], dict[str, bool]],
) -> bool:
    key = resolve_generated_sig_key(call, branches)
    if key is None:
        return False
    required_kinds = (
        {"t"} if binding == "property_set" else {"m", "i", "c", "s"}
    )
    if binding == "call" and not required_kinds.intersection(branches[key]):
        # Older generated setter fixtures used call-shaped subjects. If the
        # manifest has no callable branch for this key, follow its setter
        # activity instead of letting an active getter decide availability.
        if "t" in branches[key]:
            required_kinds = {"t"}
    activity = [
        active
        for kind, active in branches[key].items()
        if kind in required_kinds
    ]
    return bool(activity) and not any(activity)


def _typespec_variant(node: Any) -> tuple[str, Any]:
    if isinstance(node, str):
        return node, None
    if isinstance(node, dict) and len(node) == 1:
        return next(iter(node.items()))
    return "Unsupported", None


def _generated_alias_decl(
    manifest: dict[str, Any], module: str, name: str
) -> dict[str, Any] | None:
    index = manifest.get("_alias_decl_index")
    if index is None:
        strings = manifest["strings"]
        index = {
            (strings[row["module"]], strings[row["name"]]): row
            for row in manifest.get("aliases", [])
            if not strings[row["qualifier"]]
        }
        manifest["_alias_decl_index"] = index
    return index.get((module, name))


def _generated_alias_target(
    manifest: dict[str, Any], module: str, name: str
) -> int | None:
    decl = _generated_alias_decl(manifest, module, name)
    return None if decl is None else decl["target"]


def _generated_class_row(
    manifest: dict[str, Any], module: str, name: str
) -> list[Any] | None:
    index = manifest.get("_class_export_index")
    if index is None:
        strings = manifest["strings"]
        index = {
            (strings[row[0]], strings[row[1]]): row[2]
            for row in manifest.get("class_exports", [])
        }
        manifest["_class_export_index"] = index
    class_id = index.get((module, name))
    return None if class_id is None else manifest["classes"][class_id]


def _generated_protocol_status(
    manifest: dict[str, Any], module: str, name: str
) -> str:
    cache = manifest.setdefault("_protocol_status_cache", {})
    key = (module, name)
    if key in cache:
        return cache[key]
    visiting = manifest.setdefault("_protocol_status_visiting", set())
    if key in visiting:
        return "supported"
    visiting.add(key)
    try:
        row = _generated_class_row(manifest, module, name)
        if row is None or row[3] != "p" or not row[8]:
            status = "unsupported"
        else:
            method_start, method_length = row[6]
            type_param_start, type_param_length = row[4]
            class_type_params = set(
                manifest["type_param_edges"][
                    type_param_start : type_param_start + type_param_length
                ]
            )
            method_ids = manifest["class_method_edges"][
                method_start : method_start + method_length
            ]
            methods = [
                manifest["class_callables"][method_id]
                for method_id in method_ids
                if manifest["class_callables"][method_id][12]
                and manifest["class_callables"][method_id][3] == "i"
            ]
            supported = True
            unconstrained = False
            for method in methods:
                method_param_start, method_param_length = method[5]
                method_type_params = set(
                    manifest["type_param_edges"][
                        method_param_start : method_param_start + method_param_length
                    ]
                )
                if not supported or not method_type_params <= class_type_params:
                    supported = False
                    break
                param_start, param_length = method[4]
                params = [
                    param
                    for param in manifest["params"][
                        param_start : param_start + param_length
                    ]
                    if not param[4]
                ]
                if any(param[1] in {"v", "w"} for param in params):
                    supported = False
                    break
                nodes = [manifest["type_uses"][param[2]][0] for param in params]
                nodes.append(manifest["type_uses"][method[6]][0])
                statuses = [
                    _generated_typespec_status(manifest, node) for node in nodes
                ]
                if "unsupported" in statuses:
                    supported = False
                    break
                unconstrained |= "unconstrained" in statuses
            if supported:
                base_start, base_length = row[5]
                for node in manifest["edges"][base_start : base_start + base_length]:
                    node_kind, node_value = _typespec_variant(manifest["nodes"][node])
                    if node_kind == "Apply":
                        node_kind, node_value = _typespec_variant(
                            manifest["nodes"][node_value["base"]]
                        )
                    if node_kind == "Name":
                        strings = manifest["strings"]
                        marker = (strings[node_value["module"]], strings[node_value["name"]])
                        if marker in {("typing", "Protocol"), ("typing", "Generic")}:
                            continue
                    base_status = _generated_typespec_status(manifest, node)
                    if base_status == "unsupported":
                        supported = False
                        break
                    unconstrained |= base_status == "unconstrained"
            status = (
                "unconstrained"
                if supported and unconstrained
                else "supported"
                if supported
                else "unsupported"
            )
        cache[key] = status
        return status
    finally:
        visiting.remove(key)


def _materializable_status(statuses: list[str], *, outer_constraint: bool) -> str:
    """Combine child statuses using the checker materializer's failure rules."""
    if "unsupported" in statuses:
        return "unsupported"
    if outer_constraint:
        return "supported"
    if "unconstrained" in statuses:
        return "unconstrained"
    return "supported"


def _generated_type_param_status(
    manifest: dict[str, Any],
    decl_id: int,
    visiting: set[int],
    *,
    alias_frames: dict[tuple[str, str], bool],
    substitutions: dict[int, int],
) -> str:
    decl = manifest["type_params"][decl_id]
    start, length = decl["constraints"]
    constraints = manifest["edges"][start : start + length]
    bound = decl.get("bound")
    default = decl.get("default")
    constraint_nodes = ([] if bound is None else [bound]) + constraints
    constraint_statuses = [
        _generated_typespec_status(
            manifest,
            item,
            visiting,
            alias_frames=alias_frames,
            substitutions=substitutions,
        )
        for item in constraint_nodes
    ]
    if default is not None:
        default_status = _generated_typespec_status(
            manifest,
            default,
            visiting,
            alias_frames=alias_frames,
            substitutions=substitutions,
        )
        if default_status == "unsupported":
            return "unsupported"
    if "unsupported" in constraint_statuses:
        return "unsupported"
    if not constraint_statuses or "unconstrained" in constraint_statuses:
        return "unconstrained"
    return "supported"


def _typespec_contains_parameter_pack(
    manifest: dict[str, Any], node_id: int, visiting: set[int] | None = None
) -> bool:
    visiting = set() if visiting is None else visiting
    if node_id in visiting:
        return False
    visiting.add(node_id)
    try:
        kind, value = _typespec_variant(manifest["nodes"][node_id])
        if kind == "TypeParam":
            return manifest["type_params"][value]["kind"] != "t"
        if kind in {"ParamSpecArgs", "ParamSpecKwargs", "Unpack", "ParamList"}:
            return True
        if kind == "ForwardRef":
            return _typespec_contains_parameter_pack(
                manifest, value["target"], visiting
            )
        if kind in {"Union", "Tuple"}:
            start, length = value
            return any(
                _typespec_contains_parameter_pack(manifest, item, visiting)
                for item in manifest["edges"][start : start + length]
            )
        if kind == "Apply":
            start, length = value["args"]
            return any(
                _typespec_contains_parameter_pack(manifest, item, visiting)
                for item in manifest["edges"][start : start + length]
            )
        return False
    finally:
        visiting.remove(node_id)


def _callable_paramspec_shape(
    manifest: dict[str, Any], node_id: int
) -> tuple[list[int], int] | None:
    kind, value = _typespec_variant(manifest["nodes"][node_id])
    if kind != "Apply":
        return None
    base_kind, base = _typespec_variant(manifest["nodes"][value["base"]])
    if base_kind != "Name":
        return None
    strings = manifest["strings"]
    if (strings[base["module"]], strings[base["name"]]) not in {
        ("typing", "Callable"),
        ("collections.abc", "Callable"),
    }:
        return None
    start, length = value["args"]
    args = manifest["edges"][start : start + length]
    if len(args) != 2:
        return None
    param_kind, param_value = _typespec_variant(manifest["nodes"][args[0]])
    if param_kind == "TypeParam":
        return (
            ([], param_value)
            if manifest["type_params"][param_value]["kind"] == "p"
            else None
        )
    if param_kind != "Apply":
        return None
    concat_base_kind, concat_base = _typespec_variant(
        manifest["nodes"][param_value["base"]]
    )
    concat_key = (
        (strings[concat_base["module"]], strings[concat_base["name"]])
        if concat_base_kind == "Name"
        else None
    )
    concat_start, concat_length = param_value["args"]
    concat_args = manifest["edges"][
        concat_start : concat_start + concat_length
    ]
    if concat_key not in {
        ("typing", "Concatenate"),
        ("typing_extensions", "Concatenate"),
    } or len(concat_args) < 2:
        return None
    tail_kind, tail_value = _typespec_variant(manifest["nodes"][concat_args[-1]])
    prefix = concat_args[:-1]
    if (
        tail_kind != "TypeParam"
        or manifest["type_params"][tail_value]["kind"] != "p"
        or any(_typespec_contains_parameter_pack(manifest, item) for item in prefix)
    ):
        return None
    return prefix, tail_value


def _generated_typespec_guards_alias(
    manifest: dict[str, Any], kind: str, value: Any
) -> bool:
    if kind == "Tuple":
        return True
    if kind != "Apply":
        return False
    base_kind, base = _typespec_variant(manifest["nodes"][value["base"]])
    if base_kind != "Name" or base["kind"] == "a":
        return False
    strings = manifest["strings"]
    base_key = (strings[base["module"]], strings[base["name"]])
    return base_key not in {
        ("typing", "Optional"), ("typing", "Union"),
        ("typing", "Annotated"), ("typing_extensions", "Annotated"),
        ("typing", "ClassVar"), ("typing", "Final"),
        ("typing", "Required"), ("typing", "NotRequired"),
        ("typing_extensions", "ClassVar"),
        ("typing_extensions", "Final"),
        ("typing_extensions", "Required"),
        ("typing_extensions", "NotRequired"),
        ("typing_extensions", "ReadOnly"),
    }


def _generated_typespec_status(
    manifest: dict[str, Any],
    node_id: int,
    visiting: set[int] | None = None,
    *,
    alias_frames: dict[tuple[str, str], bool] | None = None,
    substitutions: dict[int, int] | None = None,
    allow_active_alias_ref: bool = False,
) -> str:
    """Return checker-parity materialization status for one TypeSpec node."""
    visiting = set() if visiting is None else visiting
    alias_frames = {} if alias_frames is None else alias_frames
    substitutions = {} if substitutions is None else substitutions
    kind, value = _typespec_variant(manifest["nodes"][node_id])
    strings = manifest["strings"]

    alias_key = None
    alias_args: list[int] = []
    if kind == "Name" and value["kind"] == "a":
        alias_key = (strings[value["module"]], strings[value["name"]])
    elif kind == "Apply":
        base_kind, base = _typespec_variant(manifest["nodes"][value["base"]])
        if base_kind == "Name" and base["kind"] == "a":
            alias_key = (strings[base["module"]], strings[base["name"]])
            start, length = value["args"]
            alias_args = manifest["edges"][start : start + length]

    if alias_key in alias_frames:
        if alias_args:
            decl = _generated_alias_decl(manifest, *alias_key)
            if decl is None:
                return "unsupported"
            _, param_length = decl["type_params"]
            if len(alias_args) != param_length:
                return "unsupported"
        arg_statuses = [
            _generated_typespec_status(
                manifest,
                item,
                visiting,
                alias_frames=alias_frames,
                substitutions=substitutions,
                allow_active_alias_ref=True,
            )
            for item in alias_args
        ]
        cycle_status = (
            "supported"
            if allow_active_alias_ref or alias_frames[alias_key]
            else "unsupported"
        )
        return _materializable_status(
            [cycle_status, *arg_statuses], outer_constraint=False
        )

    if node_id in visiting:
        return (
            "supported"
            if allow_active_alias_ref
            or any(alias_frames.values())
            or (
                bool(alias_frames)
                and _generated_typespec_guards_alias(manifest, kind, value)
            )
            else "unsupported"
        )
    visiting.add(node_id)
    try:
        if kind in {
            "Missing", "Unsupported", "LiteralBytes", "Unpack", "ParamList",
            "ParamSpecArgs", "ParamSpecKwargs",
        }:
            return "unsupported"
        if kind == "Any":
            return "unconstrained"
        if kind in {
            "Never", "None", "SelfType", "LiteralNone",
            "LiteralInt", "LiteralStr", "LiteralBool",
        }:
            return "supported"
        if kind == "Ellipsis":
            return "unconstrained"
        if kind == "ForwardRef":
            return _generated_typespec_status(
                manifest,
                value["target"],
                visiting,
                alias_frames=alias_frames,
                substitutions=substitutions,
                allow_active_alias_ref=allow_active_alias_ref,
            )
        if kind == "TypeParam":
            if value in substitutions and substitutions[value] != node_id:
                return _generated_typespec_status(
                    manifest,
                    substitutions[value],
                    visiting,
                    alias_frames=alias_frames,
                    substitutions=substitutions,
                    allow_active_alias_ref=allow_active_alias_ref,
                )
            return _generated_type_param_status(
                manifest,
                value,
                visiting,
                alias_frames=alias_frames,
                substitutions=substitutions,
            )
        if kind == "Name":
            key = (strings[value["module"]], strings[value["name"]])
            if value["kind"] == "a":
                if key == ("builtins", "_ClassInfo"):
                    return "supported"
                decl = _generated_alias_decl(manifest, *key)
                if decl is None:
                    return "unsupported"
                nested_frames = dict(alias_frames)
                nested_frames[key] = False
                return _generated_typespec_status(
                    manifest,
                    decl["target"],
                    visiting,
                    alias_frames=nested_frames,
                    substitutions=substitutions,
                )
            if value["kind"] == "p":
                return _generated_protocol_status(manifest, *key)
            if key in {("builtins", "object"), ("typing", "Any")}:
                return "unconstrained"
            supported = {
                ("builtins", "bool"), ("builtins", "int"),
                ("builtins", "float"), ("builtins", "str"),
                ("builtins", "list"), ("builtins", "set"),
                ("builtins", "frozenset"), ("builtins", "dict"),
                ("builtins", "tuple"), ("typing", "Never"),
                ("typing", "NoReturn"), ("typing", "Self"),
                ("typing_extensions", "Never"),
                ("typing_extensions", "NoReturn"),
                ("typing_extensions", "Self"), ("typing", "LiteralString"),
                ("typing_extensions", "LiteralString"),
            }
            return (
                "supported"
                if key in supported or value["kind"] in {"b", "n"}
                else "unsupported"
            )
        if kind in {"Union", "Tuple"}:
            start, length = value
            child_frames = (
                {key: True for key in alias_frames}
                if kind == "Tuple"
                else alias_frames
            )
            statuses = [
                _generated_typespec_status(
                    manifest,
                    item,
                    visiting,
                    alias_frames=child_frames,
                    substitutions=substitutions,
                    allow_active_alias_ref=allow_active_alias_ref,
                )
                for item in manifest["edges"][start : start + length]
            ]
            return _materializable_status(
                statuses, outer_constraint=kind == "Tuple"
            )
        if kind == "Apply":
            base_kind, base = _typespec_variant(manifest["nodes"][value["base"]])
            if base_kind != "Name":
                return "unsupported"
            base_key = (strings[base["module"]], strings[base["name"]])
            if base["kind"] == "a":
                decl = _generated_alias_decl(manifest, *base_key)
                if decl is None:
                    return "unsupported"
                param_start, param_length = decl["type_params"]
                type_params = manifest["type_param_edges"][
                    param_start : param_start + param_length
                ]
                if alias_args and len(alias_args) != len(type_params):
                    return "unsupported"
                arg_statuses = [
                    _generated_typespec_status(
                        manifest,
                        item,
                        visiting,
                        alias_frames=alias_frames,
                        substitutions=substitutions,
                        allow_active_alias_ref=True,
                    )
                    for item in alias_args
                ]
                if "unsupported" in arg_statuses:
                    return "unsupported"
                nested_frames = dict(alias_frames)
                nested_frames[base_key] = False
                nested_substitutions = dict(substitutions)
                if alias_args:
                    nested_substitutions.update(zip(type_params, alias_args))
                target_status = _generated_typespec_status(
                    manifest,
                    decl["target"],
                    visiting,
                    alias_frames=nested_frames,
                    substitutions=nested_substitutions,
                )
                return _materializable_status(
                    [target_status, *arg_statuses], outer_constraint=False
                )
            allowed = {
                ("builtins", "list"), ("builtins", "set"),
                ("builtins", "frozenset"), ("builtins", "dict"),
                ("builtins", "tuple"), ("typing", "Optional"),
                ("typing", "Union"), ("typing", "Literal"),
                ("typing_extensions", "Literal"), ("typing", "Callable"),
                ("collections.abc", "Callable"),
                ("typing", "Annotated"), ("typing_extensions", "Annotated"),
                ("typing", "ClassVar"), ("typing", "Final"),
                ("typing", "Required"), ("typing", "NotRequired"),
                ("typing", "TypeGuard"), ("typing", "TypeIs"),
                ("typing_extensions", "ClassVar"),
                ("typing_extensions", "Final"),
                ("typing_extensions", "Required"),
                ("typing_extensions", "NotRequired"),
                ("typing_extensions", "ReadOnly"),
                ("typing_extensions", "TypeGuard"),
                ("typing_extensions", "TypeIs"),
            }
            if base["kind"] == "p":
                if _generated_protocol_status(manifest, *base_key) == "unsupported":
                    return "unsupported"
            elif base_key not in allowed and base["kind"] not in {"b", "n"}:
                return "unsupported"
            start, length = value["args"]
            args = manifest["edges"][start : start + length]
            if base_key in {
                ("typing", "Literal"), ("typing_extensions", "Literal"),
            }:
                literal_kinds = {
                    _typespec_variant(manifest["nodes"][item])[0] for item in args
                }
                return (
                    "supported"
                    if literal_kinds <= {"LiteralInt", "LiteralStr", "LiteralBool"}
                    else "unsupported"
                )
            if base_key in {
                ("typing", "Callable"), ("collections.abc", "Callable"),
            }:
                if len(args) != 2:
                    return "unsupported"
                callable_frames = {key: True for key in alias_frames}
                param_kind, param_value = _typespec_variant(manifest["nodes"][args[0]])
                if param_kind == "ParamList":
                    param_start, param_length = param_value
                    param_statuses = [
                        _generated_typespec_status(
                            manifest,
                            item,
                            visiting,
                            alias_frames=callable_frames,
                            substitutions=substitutions,
                            allow_active_alias_ref=allow_active_alias_ref,
                        )
                        for item in manifest["edges"][
                            param_start : param_start + param_length
                        ]
                    ]
                elif param_kind == "Ellipsis":
                    param_statuses = []
                elif (shape := _callable_paramspec_shape(manifest, node_id)) is None:
                    return "unsupported"
                else:
                    prefix, param_id = shape
                    param_statuses = [
                        _generated_typespec_status(
                            manifest,
                            item,
                            visiting,
                            alias_frames=callable_frames,
                            substitutions=substitutions,
                            allow_active_alias_ref=allow_active_alias_ref,
                        )
                        for item in prefix
                    ]
                    param_statuses.append(
                        _generated_type_param_status(
                            manifest,
                            param_id,
                            visiting,
                            alias_frames=callable_frames,
                            substitutions=substitutions,
                        )
                    )
                    if "unsupported" in param_statuses:
                        return "unsupported"
                statuses = param_statuses + [
                    _generated_typespec_status(
                        manifest,
                        args[1],
                        visiting,
                        alias_frames=callable_frames,
                        substitutions=substitutions,
                        allow_active_alias_ref=allow_active_alias_ref,
                    )
                ]
                return _materializable_status(statuses, outer_constraint=True)
            if base_key in {
                ("typing", "Annotated"), ("typing_extensions", "Annotated"),
            }:
                return (
                    _generated_typespec_status(
                        manifest,
                        args[0],
                        visiting,
                        alias_frames=alias_frames,
                        substitutions=substitutions,
                        allow_active_alias_ref=allow_active_alias_ref,
                    )
                    if args
                    else "unsupported"
                )
            if base_key in {
                ("typing", "TypeGuard"), ("typing", "TypeIs"),
                ("typing_extensions", "TypeGuard"),
                ("typing_extensions", "TypeIs"),
            }:
                return "supported" if len(args) == 1 else "unsupported"
            if base_key in {
                ("typing", "ClassVar"), ("typing", "Final"),
                ("typing", "Required"), ("typing", "NotRequired"),
                ("typing_extensions", "ClassVar"),
                ("typing_extensions", "Final"),
                ("typing_extensions", "Required"),
                ("typing_extensions", "NotRequired"),
                ("typing_extensions", "ReadOnly"),
            }:
                return (
                    _generated_typespec_status(
                        manifest,
                        args[0],
                        visiting,
                        alias_frames=alias_frames,
                        substitutions=substitutions,
                        allow_active_alias_ref=allow_active_alias_ref,
                    )
                    if len(args) == 1
                    else "unsupported"
                )
            outer_constraint = base_key not in {
                ("typing", "Optional"), ("typing", "Union"),
            }
            child_frames = (
                {key: True for key in alias_frames}
                if outer_constraint
                else alias_frames
            )
            statuses = [
                _generated_typespec_status(
                    manifest,
                    item,
                    visiting,
                    alias_frames=child_frames,
                    substitutions=substitutions,
                    allow_active_alias_ref=allow_active_alias_ref,
                )
                for item in args
            ]
            return _materializable_status(
                statuses,
                outer_constraint=outer_constraint,
            )
        return "unsupported"
    finally:
        visiting.remove(node_id)


def parse_generated_signature_param_index(
    *, expand_exports: bool = True
) -> dict[tuple[str, str, str], dict[str, Any]]:
    manifest = load_generated_typespec_manifest()
    strings = manifest["strings"]
    grouped: dict[tuple[str, str, str], list[dict[str, Any]]] = {}
    for callable_row in _generated_callable_rows(
        manifest, include_class_inventory=expand_exports
    ):
        if not callable_row[12]:
            continue
        key = tuple(strings[callable_row[i]] for i in range(3))
        kind = callable_row[3]
        binding_supported = kind in {"m", "i", "c", "s", "t"}
        start, length = callable_row[4]
        callable_params = manifest["params"][start : start + length]
        captured_param_specs = {
            shape[1]
            for param in callable_params
            if (
                shape := _callable_paramspec_shape(
                    manifest, manifest["type_uses"][param[2]][0]
                )
            )
            is not None
        }
        branch: dict[str, str] = {}
        branch_reasons: dict[str, str] = {}
        ordered_params: list[dict[str, Any]] = []
        for param in callable_params:
            param_name = strings[param[0]]
            node_id = manifest["type_uses"][param[2]][0]
            node_kind, node_value = _typespec_variant(manifest["nodes"][node_id])
            reason = None
            if not binding_supported:
                status = "unsupported"
                reason = "structured_binding_unsupported"
            else:
                component_kind = {
                    "ParamSpecArgs": "v",
                    "ParamSpecKwargs": "w",
                }.get(node_kind)
                status = (
                    "supported"
                    if component_kind is not None
                    and param[1] == component_kind
                    and node_value in captured_param_specs
                    and manifest["type_params"][node_value]["kind"] == "p"
                    else _generated_typespec_status(manifest, node_id)
                )
                if status == "unsupported":
                    reason = "structured_param_type_unsupported"
            ordered_params.append(
                {
                    "name": param_name,
                    "kind": param[1],
                    "has_default": bool(param[3]),
                    "implicit_receiver": bool(param[4]),
                    "status": status,
                    "reason": reason,
                }
            )
            if param[4]:
                continue
            branch[param_name] = status
            if reason is not None:
                branch_reasons[param_name] = reason
        grouped.setdefault(key, []).append(
            {
                "params": branch,
                "reasons": branch_reasons,
                "ordered_params": ordered_params,
                "binding_supported": binding_supported,
                "kind": kind,
            }
        )

    if expand_exports:
        _expand_generated_callable_exports(manifest, grouped)

    out: dict[tuple[str, str, str], dict[str, Any]] = {}
    for key, manifest_branches in grouped.items():
        property_setters = [
            branch for branch in manifest_branches if branch["kind"] == "t"
        ]
        property_only = all(
            branch["kind"] in {"g", "t"} for branch in manifest_branches
        )
        if property_only and property_setters:
            branches = property_setters
        else:
            wired_branches = [
                branch for branch in manifest_branches if branch["binding_supported"]
            ]
            branches = wired_branches or manifest_branches
        names = {name for branch in branches for name in branch["params"]}
        params: dict[str, str] = {}
        param_reasons: dict[str, str] = {}
        for name in names:
            statuses = [branch["params"].get(name, "missing") for branch in branches]
            if "missing" in statuses:
                status = "partial"
                param_reasons[name] = "structured_param_partial"
            elif "unsupported" in statuses:
                status = "unsupported"
                reasons = {
                    branch["reasons"].get(name)
                    for branch in branches
                    if branch["params"].get(name) == "unsupported"
                }
                for reason in (
                    "structured_binding_unsupported",
                    "structured_param_type_unsupported",
                ):
                    if reason in reasons:
                        param_reasons[name] = reason
                        break
            elif "unconstrained" in statuses:
                status = "unconstrained"
            else:
                status = "supported"
            params[name] = status
        out[key] = {
            "params": params,
            "param_reasons": param_reasons,
            "branch_specs": branches,
            "enforceable": "supported" in params.values(),
            "branches": len(branches),
            "manifest_branches": len(manifest_branches),
        }
    return out


def parse_type_fixture_contract(path: Path) -> tuple[str, str, str] | None:
    text = path.read_text(encoding="utf-8", errors="replace")
    subject_match = TYPE_FIXTURE_SUBJECT_RE.search(text)
    if subject_match is None:
        return None
    subject = subject_match.group("subject")
    call_match = TYPE_FIXTURE_SUBJECT_PROPERTY_SET_RE.match(subject)
    binding = "property_set"
    if call_match is None:
        call_match = TYPE_FIXTURE_SUBJECT_CALL_RE.match(subject)
        binding = "call"
    if call_match is None:
        return None
    return (
        call_match.group("call").strip(),
        call_match.group("param"),
        binding,
    )


def parse_type_fixture_subject(path: Path) -> tuple[str, str] | None:
    parsed = parse_type_fixture_contract(path)
    return None if parsed is None else parsed[:2]


def _fixture_dotted_name(node: ast.expr) -> str | None:
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        base = _fixture_dotted_name(node.value)
        return f"{base}.{node.attr}" if base else None
    return None


def _fixture_module_binding_events(
    tree: ast.Module, name: str, before_lineno: int
) -> list[tuple[str, str | None, str | None]]:
    events: list[tuple[str, str | None, str | None]] = []

    class BindingVisitor(ast.NodeVisitor):
        def before(self, node: ast.AST) -> bool:
            return getattr(node, "lineno", before_lineno) < before_lineno

        def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
            if self.before(node) and node.name == name:
                events.append(("store", None, None))
            for decorator in node.decorator_list:
                self.visit(decorator)
            for default in (*node.args.defaults, *node.args.kw_defaults):
                if default is not None:
                    self.visit(default)
            annotations = [
                *(arg.annotation for arg in node.args.posonlyargs),
                *(arg.annotation for arg in node.args.args),
                *(arg.annotation for arg in node.args.kwonlyargs),
                node.args.vararg.annotation if node.args.vararg else None,
                node.args.kwarg.annotation if node.args.kwarg else None,
                node.returns,
                *getattr(node, "type_params", []),
            ]
            for annotation in annotations:
                if annotation is not None:
                    self.visit(annotation)

        visit_AsyncFunctionDef = visit_FunctionDef

        def visit_ClassDef(self, node: ast.ClassDef) -> None:
            if self.before(node) and node.name == name:
                events.append(("store", None, None))
            if self.before(node) and any(
                isinstance(item, ast.Global) and name in item.names
                for item in ast.walk(node)
            ):
                events.append(("store", None, None))
            for expression in (
                *node.decorator_list,
                *node.bases,
                *(keyword.value for keyword in node.keywords),
                *getattr(node, "type_params", []),
            ):
                self.visit(expression)

        def visit_Lambda(self, node: ast.Lambda) -> None:
            for default in (*node.args.defaults, *node.args.kw_defaults):
                if default is not None:
                    self.visit(default)

        def visit_ListComp(self, node: ast.ListComp) -> None:
            self.generic_visit(node)

        visit_SetComp = visit_ListComp
        visit_DictComp = visit_ListComp
        visit_GeneratorExp = visit_ListComp

        def visit_ImportFrom(self, node: ast.ImportFrom) -> None:
            if not self.before(node):
                return
            module = node.module if node.level == 0 else None
            for alias in node.names:
                if alias.name == "*":
                    events.append(("store", None, None))
                    continue
                if (alias.asname or alias.name) == name:
                    events.append(("import", module, alias.name))

        def visit_Import(self, node: ast.Import) -> None:
            if not self.before(node):
                return
            for alias in node.names:
                local = alias.asname or alias.name.split(".")[0]
                if local == name:
                    events.append(("import", alias.name, None))

        def visit_Name(self, node: ast.Name) -> None:
            if (
                self.before(node)
                and node.id == name
                and isinstance(node.ctx, (ast.Store, ast.Del))
            ):
                events.append(("store", None, None))

        def visit_ExceptHandler(self, node: ast.ExceptHandler) -> None:
            if self.before(node) and node.name == name:
                events.append(("store", None, None))
            self.generic_visit(node)

        def visit_MatchAs(self, node: ast.MatchAs) -> None:
            if self.before(node) and node.name == name:
                events.append(("store", None, None))
            self.generic_visit(node)

        def visit_MatchStar(self, node: ast.MatchStar) -> None:
            if self.before(node) and node.name == name:
                events.append(("store", None, None))

        def visit_MatchMapping(self, node: ast.MatchMapping) -> None:
            if self.before(node) and node.rest == name:
                events.append(("store", None, None))
            self.generic_visit(node)

    BindingVisitor().visit(tree)
    return events


def _fixture_binding_matches(
    tree: ast.Module,
    local: str,
    module: str,
    imported: str,
    before_lineno: int,
) -> bool:
    events = _fixture_module_binding_events(tree, local, before_lineno)
    if events:
        direct_imports = []
        for node in tree.body:
            if (
                not isinstance(node, ast.ImportFrom)
                or node.lineno >= before_lineno
                or node.level != 0
                or node.module is None
            ):
                continue
            for alias in node.names:
                if (alias.asname or alias.name) == local:
                    direct_imports.append(("import", node.module, alias.name))
        expected = [("import", module, imported)]
        return events == expected and direct_imports == expected
    return (
        module == "builtins"
        and local == imported
    )


def _fixture_receiver_identity(node: ast.expr) -> tuple[str, str] | str | None:
    literal = {
        ast.List: ("builtins", "list"),
        ast.Tuple: ("builtins", "tuple"),
        ast.Dict: ("builtins", "dict"),
        ast.Set: ("builtins", "set"),
    }
    for kind, identity in literal.items():
        if isinstance(node, kind):
            return identity
    if isinstance(node, ast.Constant):
        if isinstance(node.value, bool):
            return "builtins", "bool"
        for py_type, name in (
            (int, "int"),
            (float, "float"),
            (complex, "complex"),
            (str, "str"),
            (bytes, "bytes"),
        ):
            if isinstance(node.value, py_type):
                return "builtins", name
        return None
    if not isinstance(node, ast.Call):
        return None
    callee = _fixture_dotted_name(node.func)
    if callee == "object.__new__":
        if len(node.args) != 1 or node.keywords:
            return None
        return _fixture_dotted_name(node.args[0])
    if node.args or node.keywords:
        return None
    return callee


def _fixture_has_inert_sentinel(tree: ast.Module, before_lineno: int) -> bool:
    classes = [
        node
        for node in tree.body
        if isinstance(node, ast.ClassDef)
        and node.name == "_W"
        and node.lineno < before_lineno
    ]
    if len(classes) != 1:
        return False
    sentinel = classes[0]
    return (
        _fixture_module_binding_events(tree, "_W", before_lineno)
        == [("store", None, None)]
        and not sentinel.bases
        and not sentinel.keywords
        and not sentinel.decorator_list
        and not getattr(sentinel, "type_params", [])
        and len(sentinel.body) == 1
        and isinstance(sentinel.body[0], ast.Pass)
    )


def _fixture_bound_receiver_matches(
    tree: ast.Module, module: str, qualifier: str, before_lineno: int
) -> bool:
    events = _fixture_module_binding_events(tree, "obj", before_lineno)
    if events != [("store", None, None)]:
        return False
    values = []
    for node in tree.body:
        if getattr(node, "lineno", before_lineno) >= before_lineno:
            continue
        if isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == "obj"
            for target in node.targets
        ):
            values.append(node.value)
        elif (
            isinstance(node, ast.AnnAssign)
            and isinstance(node.target, ast.Name)
            and node.target.id == "obj"
            and node.value is not None
        ):
            values.append(node.value)
    if len(values) != 1:
        return False
    value = values[0]
    if (
        isinstance(value, ast.Call)
        and _fixture_dotted_name(value.func) == "object.__new__"
        and not _fixture_binding_matches(
            tree, "object", "builtins", "object", before_lineno
        )
    ):
        return False
    identity = _fixture_receiver_identity(value)
    expected = qualifier.split(".")[0]
    if isinstance(identity, tuple):
        return identity == (module, expected)
    if not isinstance(identity, str) or "." in identity:
        return False
    return _fixture_binding_matches(
        tree, identity, module, expected, before_lineno
    )


def _fixture_has_canonical_execution(
    tree: ast.Module,
    call: ast.Call,
    expected_binding: tuple[str, str, str],
) -> bool:
    containers = [
        node
        for node in tree.body
        if node.lineno <= call.lineno <= (node.end_lineno or node.lineno)
    ]
    if len(containers) != 1 or containers[0] is not tree.body[-1]:
        return False
    container = containers[0]

    imports: list[tuple[str, str, str]] = []
    for index, node in enumerate(tree.body[:-1]):
        if (
            index == 0
            and isinstance(node, ast.Expr)
            and isinstance(node.value, ast.Constant)
            and isinstance(node.value.value, str)
        ):
            continue
        if isinstance(node, ast.ClassDef) and node.name == "_W":
            continue
        if (
            isinstance(node, ast.ImportFrom)
            and node.level == 0
            and node.module is not None
            and len(node.names) == 1
            and node.names[0].name != "*"
        ):
            alias = node.names[0]
            imports.append((alias.asname or alias.name, node.module, alias.name))
            continue
        return False

    local, module, imported = expected_binding
    expected_import = (local, module, imported)
    if module == "builtins" and local == imported:
        if imports not in ([], [expected_import]):
            return False
    elif imports != [expected_import]:
        return False

    if isinstance(container, ast.Expr):
        return container.value is call
    if not isinstance(container, (ast.Try, ast.TryStar)):
        return False
    if (
        container.orelse
        or container.finalbody
        or len(container.body) != 2
        or len(container.handlers) != 2
    ):
        return False
    operation, no_typeerror = container.body
    if not (
        isinstance(operation, ast.Expr)
        and operation.value is call
        and isinstance(no_typeerror, ast.Expr)
        and isinstance(no_typeerror.value, ast.Call)
    ):
        return False
    print_call = no_typeerror.value
    if not (
        isinstance(print_call.func, ast.Name)
        and print_call.func.id == "print"
        and len(print_call.args) == 1
        and not print_call.keywords
        and isinstance(print_call.args[0], ast.Constant)
        and isinstance(print_call.args[0].value, str)
        and print_call.args[0].value.startswith("no_typeerror:")
    ):
        return False

    def canonical_handler(
        handler: ast.ExceptHandler, exception: str, prefix: str
    ) -> bool:
        if not (
            isinstance(handler.type, ast.Name)
            and handler.type.id == exception
            and handler.name == "e"
            and len(handler.body) == 1
            and isinstance(handler.body[0], ast.Expr)
            and isinstance(handler.body[0].value, ast.Call)
        ):
            return False
        output = handler.body[0].value
        if not (
            isinstance(output.func, ast.Name)
            and output.func.id == "print"
            and len(output.args) == 2
            and not output.keywords
            and isinstance(output.args[0], ast.Constant)
            and output.args[0].value == prefix
        ):
            return False
        type_name = output.args[1]
        return (
            isinstance(type_name, ast.Attribute)
            and type_name.attr == "__name__"
            and isinstance(type_name.value, ast.Call)
            and isinstance(type_name.value.func, ast.Name)
            and type_name.value.func.id == "type"
            and len(type_name.value.args) == 1
            and not type_name.value.keywords
            and isinstance(type_name.value.args[0], ast.Name)
            and type_name.value.args[0].id == "e"
        )

    return canonical_handler(
        container.handlers[0], "TypeError", "typeerror:"
    ) and canonical_handler(
        container.handlers[1], "Exception", "setup_or_other:"
    )


def parse_type_fixture_call_shape(
    path: Path, key: tuple[str, str, str]
) -> FixtureCallShape | None:
    text = path.read_text(encoding="utf-8", errors="replace")
    subject_match = TYPE_FIXTURE_SUBJECT_RE.search(text)
    if subject_match is None:
        return None
    subject = subject_match.group("subject")
    subject_call = TYPE_FIXTURE_SUBJECT_CALL_RE.match(subject)
    if subject_call is None:
        return None
    label = subject_call.group("label").strip()
    subject_param = subject_call.group("param")
    try:
        tree = ast.parse(text)
        expected = ast.parse(WRONG_VALUE.get(label, SENTINEL), mode="eval").body
        comments = [
            token
            for token in tokenize.generate_tokens(io.StringIO(text).readline)
            if token.type == tokenize.COMMENT and "<- wrong-typed" in token.string
        ]
    except (SyntaxError, tokenize.TokenError):
        return None
    if len(comments) != 1:
        return None
    marker = comments[0]
    marker_pattern = (
        rf"#\s*{re.escape(subject_param)}\s*:\s*{re.escape(label)}"
        rf"\s*<-\s*wrong-typed\s*"
    )
    if re.fullmatch(marker_pattern, marker.string) is None:
        return None
    marker_line = marker.start[0]
    operations = []

    def collect_module_operations(body: list[ast.stmt]) -> None:
        for node in body:
            if (
                isinstance(node, ast.Expr)
                and isinstance(node.value, ast.Call)
                and node.lineno <= marker_line <= (node.end_lineno or node.lineno)
            ):
                operations.append(node.value)
            elif isinstance(node, (ast.Try, ast.TryStar)):
                collect_module_operations(node.body)

    collect_module_operations(tree.body)
    if len(operations) != 1:
        return None
    call = operations[0]
    has_sentinel_class = any(
        isinstance(node, ast.ClassDef)
        and node.name == "_W"
        and node.lineno < call.lineno
        for node in tree.body
    )
    if (
        WRONG_VALUE.get(label, SENTINEL) == SENTINEL or has_sentinel_class
    ) and not _fixture_has_inert_sentinel(tree, call.lineno):
        return None
    if any(isinstance(arg, ast.Starred) for arg in call.args):
        return None
    if any(keyword.arg is None for keyword in call.keywords):
        return None
    keyword_names = tuple(keyword.arg for keyword in call.keywords if keyword.arg)
    if len(set(keyword_names)) != len(keyword_names):
        return None

    expected_dump = ast.dump(expected, include_attributes=False)
    targets: list[tuple[str, int | str]] = []
    for index, arg in enumerate(call.args):
        if ast.dump(arg, include_attributes=False) == expected_dump:
            targets.append(("pos", index))
    for keyword in call.keywords:
        if ast.dump(keyword.value, include_attributes=False) == expected_dump:
            targets.append(("kw", keyword.arg or ""))
    if len(targets) != 1:
        return None

    callee = _fixture_dotted_name(call.func)
    if callee is None:
        return None
    module, qualifier, name = key
    if not qualifier:
        if not isinstance(call.func, ast.Name) or callee != name:
            return None
        if not _fixture_binding_matches(tree, callee, module, name, call.lineno):
            return None
        access = "module"
        expected_binding = (callee, module, name)
    elif name in {"__init__", "__new__"} and isinstance(call.func, ast.Name):
        if call.func.id != qualifier.split(".")[-1]:
            return None
        if not _fixture_binding_matches(
            tree, call.func.id, module, qualifier.split(".")[0], call.lineno
        ):
            return None
        access = "constructor"
        expected_binding = (
            call.func.id,
            module,
            qualifier.split(".")[0],
        )
    elif isinstance(call.func, ast.Attribute) and call.func.attr == name:
        owner = _fixture_dotted_name(call.func.value)
        if owner is None:
            return None
        if owner == "obj":
            if not _fixture_bound_receiver_matches(
                tree, module, qualifier, call.lineno
            ):
                return None
            access = "bound"
            expected_binding = None
        else:
            owner_parts = owner.split(".")
            qualifier_parts = qualifier.split(".")
            local = owner_parts[0]
            if owner_parts[1:] != qualifier_parts[1:]:
                return None
            if not _fixture_binding_matches(
                tree, local, module, qualifier_parts[0], call.lineno
            ):
                return None
            access = "class"
            expected_binding = (local, module, qualifier_parts[0])
    else:
        return None
    if expected_binding is not None and not _fixture_has_canonical_execution(
        tree, call, expected_binding
    ):
        return None
    return FixtureCallShape(
        access=access,
        positional_count=len(call.args),
        keyword_names=keyword_names,
        target=targets[0],
        subject_param=subject_param,
    )


def _bind_fixture_branch_target(
    branch: dict[str, Any], shape: FixtureCallShape
) -> dict[str, Any] | None:
    hide_implicit_receiver = shape.access != "module" and not (
        shape.access == "class" and branch["kind"] == "i"
    )
    visible = [
        param
        for param in branch["ordered_params"]
        if not (hide_implicit_receiver and param["implicit_receiver"])
    ]
    positional = [
        index for index, param in enumerate(visible) if param["kind"] in {"p", "r"}
    ]
    var_pos = next(
        (index for index, param in enumerate(visible) if param["kind"] == "v"),
        None,
    )
    var_kw = next(
        (index for index, param in enumerate(visible) if param["kind"] == "w"),
        None,
    )
    bound: set[int] = set()
    target_param = None
    for position in range(shape.positional_count):
        param_index = positional[position] if position < len(positional) else var_pos
        if param_index is None:
            return None
        param = visible[param_index]
        if param["kind"] not in {"v", "w"} and param_index in bound:
            return None
        bound.add(param_index)
        if shape.target == ("pos", position):
            target_param = param
    for name in shape.keyword_names:
        param_index = next(
            (
                index
                for index, param in enumerate(visible)
                if param["kind"] in {"r", "k"} and param["name"] == name
            ),
            var_kw,
        )
        if param_index is None:
            return None
        param = visible[param_index]
        if param["kind"] not in {"v", "w"} and param_index in bound:
            return None
        bound.add(param_index)
        if shape.target == ("kw", name):
            target_param = param
    if any(
        param["kind"] not in {"v", "w"}
        and not param["has_default"]
        and index not in bound
        for index, param in enumerate(visible)
    ):
        return None
    return target_param


def _status_for_fixture_call(
    signature: dict[str, Any], shape: FixtureCallShape
) -> tuple[str, str | None] | None:
    if shape.access == "bound":
        return None
    branches = [
        branch
        for branch in signature.get("branch_specs", [])
        if (
            (shape.access == "module" and branch["kind"] == "m")
            or (shape.access != "module" and branch["kind"] in {"i", "c", "s"})
        )
    ]
    if not branches:
        return None
    if shape.access in {"bound", "class"} and len(
        {branch["kind"] for branch in branches}
    ) > 1:
        return None
    targets = [
        target
        for branch in branches
        if (target := _bind_fixture_branch_target(branch, shape)) is not None
    ]
    if not targets:
        return None
    if not all(target["name"] == shape.subject_param for target in targets):
        return None
    unsupported = [target for target in targets if target["status"] == "unsupported"]
    if unsupported:
        reasons = {target["reason"] for target in unsupported}
        for reason in (
            "structured_binding_unsupported",
            "structured_param_type_unsupported",
        ):
            if reason in reasons:
                return "unsupported", reason
        return "unsupported", "structured_param_type_unsupported"
    if (
        len(targets) != 1
        or shape.positional_count + len(shape.keyword_names) != 1
    ):
        return None
    if targets[0]["status"] == "unconstrained":
        return "unconstrained", None
    if targets[0]["status"] == "supported":
        return "supported", None
    return None


def resolve_generated_sig_key(
    call: str, sigs: dict[tuple[str, str, str], dict[str, Any]]
) -> tuple[str, str, str] | None:
    parts = call.split(".")
    if len(parts) < 2:
        return None
    for split in range(len(parts) - 1, 0, -1):
        module = ".".join(parts[:split])
        rest = parts[split:]
        if not rest:
            continue
        qualifier = ".".join(rest[:-1])
        name = rest[-1]
        key = (module, qualifier, name)
        if key in sigs:
            return key
    return None


def unenforceable_generated_param_reason(
    path: Path, sigs: dict[tuple[str, str, str], dict[str, Any]]
) -> str | None:
    text = path.read_text(encoding="utf-8", errors="replace")
    has_typevar_marker = TYPEVAR_STAYS_UNWALLED_MARKER in text
    parsed = parse_type_fixture_subject(path)
    if parsed is None:
        return (
            "stale_typevar_unwalled_marker_unparseable"
            if has_typevar_marker
            else None
        )
    call, param = parsed
    key = resolve_generated_sig_key(call, sigs)
    if key is None:
        if has_typevar_marker:
            return "stale_typevar_unwalled_marker_missing_signature"
        if (TYPE_DIR / "std-libs") in path.parents:
            return "structured_signature_missing"
        return None
    params = sigs[key]["params"]
    if param not in params:
        reason = "structured_param_missing"
        return (
            f"stale_typevar_unwalled_marker_{reason}"
            if has_typevar_marker
            else reason
        )
    status = params[param]
    resolved_reason = None
    if status == "partial":
        shape = parse_type_fixture_call_shape(path, key)
        resolved = (
            _status_for_fixture_call(sigs[key], shape)
            if shape is not None
            else None
        )
        if resolved is not None:
            status, resolved_reason = resolved
    if status == "unconstrained":
        return "contract_unconstrained"
    reason = resolved_reason or sigs[key].get("param_reasons", {}).get(param)
    if reason is None:
        reason = (
            "structured_param_partial"
            if status == "partial"
            else "structured_param_unsupported"
        )
    if has_typevar_marker:
        return f"stale_typevar_unwalled_marker_{reason}"
    if status in {"partial", "unsupported"}:
        return reason
    return None


def partition_generated_contract_coverage(
    paths: list[Path], sigs: dict[tuple[str, str, str], dict[str, Any]]
) -> tuple[list[dict[str, str]], list[dict[str, str]]]:
    """Separate intentionally unconstrained contracts from implementation gaps."""
    unconstrained_contracts: list[dict[str, str]] = []
    unresolved_contracts: list[dict[str, str]] = []
    for path in paths:
        reason = unenforceable_generated_param_reason(path, sigs)
        if reason == "contract_unconstrained":
            unconstrained_contracts.append({"path": repo_rel(path), "reason": reason})
        elif reason is not None:
            unresolved_contracts.append({"path": repo_rel(path), "reason": reason})
    return unconstrained_contracts, unresolved_contracts


def run_mamba(mamba_bin: str, fixture: Path, timeout: int) -> tuple[int | None, str, str]:
    inner = (
        f"ulimit -t {timeout} 2>/dev/null; "
        f"ulimit -c 0 2>/dev/null; "
        f"exec {shlex.quote(mamba_bin)} run {shlex.quote(str(fixture))}"
    )
    return harness_lib.run_fixture(["/bin/sh", "-c", inner], timeout + 5)


def is_type_rejection(stdout: str, stderr: str) -> bool:
    blob = f"{stderr}\n{stdout}"
    if any(marker in blob for marker in NON_TYPE_REJECTION_MARKERS):
        return False
    return any(marker in blob for marker in TYPE_REJECTION_MARKERS)


def parse_generated_signature_counts(typeshed_stdlib: Path) -> dict[str, Any]:
    manifest = load_generated_typespec_manifest()
    index = parse_generated_signature_param_index(expand_exports=False)
    statuses = Counter(
        status for signature in index.values() for status in signature["params"].values()
    )
    unsupported_reasons = Counter(
        reason
        for signature in index.values()
        for reason in signature["param_reasons"].values()
    )
    py312_callables = [row for row in manifest["callables"] if row[12]]
    callable_kind_names = {
        "m": "module",
        "i": "instance",
        "c": "class",
        "s": "static",
        "g": "property_get",
        "t": "property_set",
    }
    callable_kind_branches = Counter(
        callable_kind_names[row[3]] for row in py312_callables
    )
    structured_wired_kinds = [
        "module",
        "instance",
        "class",
        "static",
        "property_get",
        "property_set",
    ]
    structured_wired_branches = sum(
        callable_kind_branches[kind] for kind in structured_wired_kinds
    )
    enforceable = sum(signature["enforceable"] for signature in index.values())
    return {
        "source": "structured_typespec_manifest",
        "schema": manifest["schema"],
        "rows": len(index),
        "branches": len(py312_callables),
        "callable_kind_branches": dict(sorted(callable_kind_branches.items())),
        "structured_module_branches": callable_kind_branches["module"],
        "structured_wired_kinds": structured_wired_kinds,
        "structured_wired_branches": structured_wired_branches,
        "unhandled_binding_branches": len(py312_callables) - structured_wired_branches,
        "enforceable": enforceable,
        "structured_enforceable_callables": enforceable,
        "supported_params": statuses["supported"],
        "unconstrained_params": statuses["unconstrained"],
        "unsupported_params": statuses["unsupported"],
        "partial_overload_params": statuses["partial"],
        "unsupported_param_reasons": dict(sorted(unsupported_reasons.items())),
        "unsupported_or_partial_params": (
            statuses["unsupported"] + statuses["partial"]
        ),
        "vendor_typeshed_available": typeshed_stdlib.is_dir(),
    }


def verify_generated_signature_snapshot(typeshed_stdlib: Path) -> dict[str, Any]:
    if not typeshed_stdlib.is_dir():
        return {
            "current": False,
            "exit_code": None,
            "problem": f"missing typeshed stdlib directory: {typeshed_stdlib}",
        }

    command = [
        sys.executable,
        str(TOOLS_DIR / "type_wall_gen.py"),
        "--check-rust",
        "--typeshed-stdlib",
        str(typeshed_stdlib),
    ]
    try:
        result = subprocess.run(
            command,
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        return {
            "current": False,
            "exit_code": None,
            "problem": str(error),
        }
    output = "\n".join(part.strip() for part in (result.stdout, result.stderr) if part.strip())
    return {
        "current": result.returncode == 0,
        "exit_code": result.returncode,
        "problem": None if result.returncode == 0 else output[:500],
    }


def grade_enforcement(mamba_bin: str, fixture: Path, timeout: int) -> tuple[str, str, str]:
    rc, out, err = run_mamba(mamba_bin, fixture, timeout)
    rel = fixture.relative_to(TYPE_DIR).parts
    bucket = rel[0] if len(rel) > 1 else "core"
    if rc is None:
        return bucket, repo_rel(fixture), "ungradable"
    if rc != 0:
        return (
            bucket,
            repo_rel(fixture),
            "enforced" if is_type_rejection(out, err) else "ungradable",
        )
    if "no_typeerror:" in out:
        return bucket, repo_rel(fixture), "leaked"
    if "typeerror:" in out:
        return bucket, repo_rel(fixture), "enforced"
    return bucket, repo_rel(fixture), "ungradable"


def grade_soundness(mamba_bin: str, fixture: Path, timeout: int) -> tuple[str, str]:
    orc, oout, _ = harness_lib.run_fixture(["python3.12", str(fixture)], timeout)
    rel = repo_rel(fixture)
    if orc != 0:
        return rel, "oracle_skip"
    mrc, mout, _ = run_mamba(mamba_bin, fixture, timeout)
    if mrc == 0 and mout.strip() == oout.strip():
        return rel, "passed"
    return rel, "failed"


def load_divergences() -> list[Divergence]:
    if not TYPE_DIVERGENCES.exists():
        return []
    out: list[Divergence] = []
    current_owner_refs: list[str] = []
    for raw in TYPE_DIVERGENCES.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.startswith("#"):
            if "owner:" in line:
                current_owner_refs = re.findall(r"#\d+", line)
            continue
        out.append(Divergence(path=line, owner_refs=current_owner_refs))
        current_owner_refs = []
    return out


def validate_divergence(
    divergence: Divergence, mamba_bin: str, timeout: int
) -> dict[str, Any]:
    fixture = REPO_ROOT / divergence.path
    problems: list[str] = []
    if not divergence.owner_refs:
        problems.append("missing owner")
    if not fixture.exists():
        problems.append("fixture missing")
        return {
            "path": divergence.path,
            "owner_refs": divergence.owner_refs,
            "valid": False,
            "problems": problems,
        }
    try:
        facet = fixture.relative_to(FIXTURES_DIR).parts[0]
    except ValueError:
        facet = ""
    if facet not in BEHAVIOR_FACETS:
        problems.append(f"not a behavior-denominator facet: {facet or '<outside>'}")
    orc, _oout, oerr = harness_lib.run_fixture(["python3.12", str(fixture)], timeout)
    if orc != 0:
        problems.append(f"cpython oracle failed: rc={orc} stderr={oerr[:160]}")
    mrc, mout, merr = run_mamba(mamba_bin, fixture, timeout)
    if mrc is None:
        problems.append("mamba timed out or could not run")
    elif mrc == 0:
        problems.append("mamba did not reject")
    elif not is_type_rejection(mout, merr):
        problems.append("mamba rejected, but not with a verified type rejection")
    return {
        "path": divergence.path,
        "owner_refs": divergence.owner_refs,
        "valid": not problems,
        "problems": problems,
    }


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    mamba_bin = args.mamba_bin or default_mamba_bin()
    typeshed_stdlib = (
        args.typeshed_stdlib
        or os.environ.get("MAMBA_TYPESHED_STDLIB")
        or DEFAULT_TYPESHED_STDLIB
    )
    typeshed_stdlib = Path(typeshed_stdlib).resolve()
    type_fixture_candidates = sorted(TYPE_DIR.rglob("*.py")) if TYPE_DIR.exists() else []
    generated_param_sigs = parse_generated_signature_param_index()
    excluded_non_runtime_stubs = [
        path for path in type_fixture_candidates if is_non_runtime_stub_type_fixture(path)
    ]
    excluded_non_stdlib_backports = [
        path
        for path in type_fixture_candidates
        if is_non_stdlib_backport_type_fixture(path)
    ]
    excluded_platform_specific = [
        path
        for path in type_fixture_candidates
        if is_platform_specific_unavailable_type_fixture(path)
    ]
    excluded_optional_extensions = [
        path
        for path in type_fixture_candidates
        if is_optional_stdlib_extension_unavailable_type_fixture(path)
    ]
    excluded_version_specific = [
        path
        for path in type_fixture_candidates
        if is_version_specific_unavailable_type_fixture(path)
    ]
    excluded_inactive_typespec = [
        path
        for path in type_fixture_candidates
        if is_generated_typespec_inactive_type_fixture(path)
    ]
    type_fixture_wall_candidates = executable_type_fixtures(type_fixture_candidates)
    excluded_unconstrained_contracts, unresolved_generated_contracts = (
        partition_generated_contract_coverage(
            type_fixture_wall_candidates, generated_param_sigs
        )
    )
    excluded_unconstrained_contract_paths = {
        REPO_ROOT / item["path"] for item in excluded_unconstrained_contracts
    }
    type_fixtures_all = [
        path
        for path in type_fixture_wall_candidates
        if path not in excluded_unconstrained_contract_paths
    ]
    unresolved_generated_contract_reasons = Counter(
        item["reason"] for item in unresolved_generated_contracts
    )
    type_fixtures, enforcement_sampled = selected(type_fixtures_all, args.limit)
    sound_fixtures_all = sorted(
        path for family in SOUND_FAMILIES for path in (SOUND_DIR / family).glob("*.py")
    )
    sound_fixtures, sound_sampled = selected(sound_fixtures_all, args.limit)

    by_bucket: dict[str, Counter] = {}
    enforcement_blockers: list[dict[str, str]] = []
    with ThreadPoolExecutor(max_workers=max(1, args.jobs)) as executor:
        for bucket, rel, verdict in executor.map(
            lambda path: grade_enforcement(mamba_bin, path, args.timeout),
            type_fixtures,
        ):
            by_bucket.setdefault(bucket, Counter())[verdict] += 1
            if verdict != "enforced" and len(enforcement_blockers) < args.show:
                enforcement_blockers.append({"path": rel, "verdict": verdict})

    enforcement_counts = Counter()
    for counter in by_bucket.values():
        enforcement_counts.update(counter)
    enforcement_gradable = enforcement_counts["enforced"] + enforcement_counts["leaked"]
    enforcement_rate = (
        100.0 * enforcement_counts["enforced"] / enforcement_gradable
        if enforcement_gradable
        else 0.0
    )

    sound_counts: Counter = Counter()
    sound_blockers: list[dict[str, str]] = []
    with ThreadPoolExecutor(max_workers=max(1, args.jobs)) as executor:
        for rel, verdict in executor.map(
            lambda path: grade_soundness(mamba_bin, path, args.timeout), sound_fixtures
        ):
            sound_counts[verdict] += 1
            if verdict != "passed" and len(sound_blockers) < args.show:
                sound_blockers.append({"path": rel, "verdict": verdict})
    sound_gradable = sound_counts["passed"] + sound_counts["failed"]
    sound_rate = (
        100.0 * sound_counts["passed"] / sound_gradable if sound_gradable else 0.0
    )

    divergence_entries = [
        validate_divergence(item, mamba_bin, args.timeout) for item in load_divergences()
    ]
    invalid_divergences = [item for item in divergence_entries if not item["valid"]]
    missing_owner = [item for item in divergence_entries if not item["owner_refs"]]

    typeshed = parse_generated_signature_counts(typeshed_stdlib)
    generated_snapshot = verify_generated_signature_snapshot(typeshed_stdlib)
    host_python_version = sys.version_info[:2]
    sampled = enforcement_sampled or sound_sampled
    ready = (
        not sampled
        and host_python_version == EXPECTED_PYTHON_VERSION
        and typeshed["vendor_typeshed_available"]
        and generated_snapshot["current"]
        and typeshed["enforceable"] > 0
        and not unresolved_generated_contracts
        and enforcement_counts["leaked"] == 0
        and enforcement_counts["ungradable"] == 0
        and enforcement_counts["enforced"] == len(type_fixtures_all)
        and sound_counts["failed"] == 0
        and sound_counts["oracle_skip"] == 0
        and sound_counts["passed"] == len(sound_fixtures_all)
        and not invalid_divergences
        and not missing_owner
    )

    blockers: list[dict[str, Any]] = []
    if host_python_version != EXPECTED_PYTHON_VERSION:
        blockers.append(
            {
                "kind": "wrong_accounting_python",
                "reason": (
                    f"requires Python {EXPECTED_PYTHON_VERSION[0]}."
                    f"{EXPECTED_PYTHON_VERSION[1]}, got "
                    f"{host_python_version[0]}.{host_python_version[1]}"
                ),
            }
        )
    if not typeshed["vendor_typeshed_available"]:
        blockers.append(
            {
                "kind": "missing_typeshed",
                "reason": f"typeshed stdlib directory is unavailable: {typeshed_stdlib}",
            }
        )
    elif not generated_snapshot["current"]:
        blockers.append(
            {
                "kind": "stale_generated_signature_snapshot",
                "reason": generated_snapshot["problem"] or "typeshed snapshot check failed",
            }
        )
    if unresolved_generated_contracts:
        blockers.append(
            {
                "kind": "unsupported_generated_contracts",
                "count": len(unresolved_generated_contracts),
                "reasons": dict(sorted(unresolved_generated_contract_reasons.items())),
                "examples": unresolved_generated_contracts[: args.show],
            }
        )
    if sampled:
        blockers.append(
            {
                "kind": "sampled_runtime_accounting",
                "reason": "sampled runs are development evidence, not replacement proof",
            }
        )
    if enforcement_counts["leaked"] or enforcement_counts["ungradable"]:
        blockers.extend(
            {"kind": "type_enforcement", **item} for item in enforcement_blockers
        )
    if sound_counts["failed"] or sound_counts["oracle_skip"]:
        blockers.extend({"kind": "type_soundness", **item} for item in sound_blockers)
    blockers.extend(
        {
            "kind": "invalid_type_divergence",
            "path": item["path"],
            "problems": item["problems"],
        }
        for item in invalid_divergences[: args.show]
    )

    return {
        "schema_version": 1,
        "profile": "strict-type-accounting",
        "status": "green" if ready else "red",
        "ready": ready,
        "sampled": sampled,
        "mamba_bin": mamba_bin,
        "typeshed": {
            **typeshed,
            "stdlib_path": str(typeshed_stdlib),
            "generated_snapshot": generated_snapshot,
            "type_fixture_wall": len(type_fixtures_all),
            "measured_type_fixtures": len(type_fixtures),
            "excluded_non_runtime_stub_fixtures": len(excluded_non_runtime_stubs),
            "excluded_non_runtime_stub_lib_prefixes": list(NON_RUNTIME_STUB_TYPE_LIB_PREFIXES),
            "excluded_non_stdlib_backport_type_fixtures": len(
                excluded_non_stdlib_backports
            ),
            "non_stdlib_backport_type_libs": sorted(NON_STDLIB_BACKPORT_TYPE_LIBS),
            "excluded_platform_specific_type_fixtures": len(excluded_platform_specific),
            "platform_specific_type_libs": PLATFORM_SPECIFIC_TYPE_LIBS,
            "host_platform": sys.platform,
            "excluded_optional_stdlib_extension_type_fixtures": len(
                excluded_optional_extensions
            ),
            "optional_stdlib_extension_type_libs": OPTIONAL_STDLIB_EXTENSION_TYPE_LIBS,
            "excluded_version_specific_type_fixtures": len(excluded_version_specific),
            "version_specific_type_libs": VERSION_SPECIFIC_TYPE_LIBS,
            "version_removed_type_libs": VERSION_REMOVED_TYPE_LIBS,
            "version_specific_type_fixture_cases": VERSION_SPECIFIC_TYPE_FIXTURES,
            "version_removed_type_fixture_cases": VERSION_REMOVED_TYPE_FIXTURES,
            "excluded_inactive_typespec_fixtures": len(excluded_inactive_typespec),
            "excluded_unconstrained_contract_fixtures": len(
                excluded_unconstrained_contracts
            ),
            "excluded_unconstrained_contract_examples": (
                excluded_unconstrained_contracts[: args.show]
            ),
            "unresolved_generated_contract_type_fixtures": len(
                unresolved_generated_contracts
            ),
            "unresolved_generated_contract_reasons": dict(
                sorted(unresolved_generated_contract_reasons.items())
            ),
            "unresolved_generated_contract_examples": unresolved_generated_contracts[
                : args.show
            ],
            "host_python_version": list(host_python_version),
            "required_python_version": list(EXPECTED_PYTHON_VERSION),
        },
        "enforcement": {
            "fixtures": len(type_fixtures_all),
            "measured": len(type_fixtures),
            "sampled": enforcement_sampled,
            "gradable": enforcement_gradable,
            "enforced": enforcement_counts["enforced"],
            "leaked": enforcement_counts["leaked"],
            "ungradable": enforcement_counts["ungradable"],
            "rate": round(enforcement_rate, 3),
            "by_bucket": {
                bucket: dict(sorted(counter.items())) for bucket, counter in by_bucket.items()
            },
            "blockers": enforcement_blockers,
        },
        "soundness": {
            "fixtures": len(sound_fixtures_all),
            "measured": len(sound_fixtures),
            "sampled": sound_sampled,
            "gradable": sound_gradable,
            "passed": sound_counts["passed"],
            "failed": sound_counts["failed"],
            "oracle_skip": sound_counts["oracle_skip"],
            "rate": round(sound_rate, 3),
            "blockers": sound_blockers,
        },
        "divergences": {
            "declared": len(divergence_entries),
            "valid": len(divergence_entries) - len(invalid_divergences),
            "invalid": len(invalid_divergences),
            "missing_owner": len(missing_owner),
            "entries": divergence_entries[: args.show],
        },
        "blockers": blockers[: args.show],
    }


def print_human(report: dict[str, Any]) -> None:
    print(f"strict-type accounting: {report['status']}")
    print(f"  mamba: {report['mamba_bin']}")
    print(f"  sampled: {report['sampled']}")
    typeshed = report["typeshed"]
    print(
        "  typeshed: "
        f"schema={typeshed['schema']} rows={typeshed['rows']} "
        f"branches={typeshed['branches']} enforceable={typeshed['enforceable']} "
        f"structured_wired_branches={typeshed['structured_wired_branches']} "
        f"unhandled_binding_branches={typeshed['unhandled_binding_branches']} "
        f"supported_params={typeshed['supported_params']} "
        f"unconstrained_params={typeshed['unconstrained_params']} "
        f"unsupported_params={typeshed['unsupported_params']} "
        f"partial_params={typeshed['partial_overload_params']} "
        f"fixtures={typeshed['type_fixture_wall']} "
        "unresolved_contracts="
        f"{typeshed['unresolved_generated_contract_type_fixtures']} "
        "unconstrained_contracts="
        f"{typeshed['excluded_unconstrained_contract_fixtures']} "
        f"snapshot_current={typeshed['generated_snapshot']['current']}"
    )
    enforcement = report["enforcement"]
    print(
        "  enforcement: "
        f"measured={enforcement['measured']} gradable={enforcement['gradable']} "
        f"enforced={enforcement['enforced']} leaked={enforcement['leaked']} "
        f"ungradable={enforcement['ungradable']} rate={enforcement['rate']:.1f}%"
    )
    soundness = report["soundness"]
    print(
        "  soundness: "
        f"measured={soundness['measured']} gradable={soundness['gradable']} "
        f"passed={soundness['passed']} failed={soundness['failed']} "
        f"oracle_skip={soundness['oracle_skip']} rate={soundness['rate']:.1f}%"
    )
    divergences = report["divergences"]
    print(
        "  divergences: "
        f"declared={divergences['declared']} valid={divergences['valid']} "
        f"invalid={divergences['invalid']} missing_owner={divergences['missing_owner']}"
    )
    for blocker in report["blockers"]:
        label = blocker.get("path") or blocker.get("kind")
        reason = blocker.get("verdict") or blocker.get("reason") or blocker.get("problems")
        print(f"  blocker: {label} - {reason}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--show", type=int, default=10)
    parser.add_argument("--jobs", type=int, default=max(1, min(8, os.cpu_count() or 1)))
    parser.add_argument("--limit", type=int, default=0, help="sample N type/soundness fixtures")
    parser.add_argument("--timeout", type=int, default=10)
    parser.add_argument("--mamba-bin")
    parser.add_argument(
        "--typeshed-stdlib",
        type=Path,
        help="typeshed stdlib directory (or set MAMBA_TYPESHED_STDLIB)",
    )
    args = parser.parse_args(argv)

    report = build_report(args)
    if args.json:
        print(json.dumps(report, sort_keys=True))
    else:
        print_human(report)
    return 0 if report["ready"] else EXIT_NOT_READY


if __name__ == "__main__":
    raise SystemExit(main())
