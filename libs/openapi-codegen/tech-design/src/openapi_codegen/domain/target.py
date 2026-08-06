from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from enum import Enum

from openapi_codegen.domain.errors import (
    MissingPolicyKey,
    PolicyLanguageMismatch,
    TargetLanguageMismatch,
    UnknownTargetProfile,
)
from openapi_codegen.domain.lang import Lang


class PythonTarget(Enum):
    PY311 = "3.11"
    PY312 = "3.12"
    PY313 = "3.13"
    PY314 = "3.14"

    @property
    def minimum_version(self) -> str:
        return self.value

    @property
    def uses_pep695_type_aliases(self) -> bool:
        return self != PythonTarget.PY311


class TypeScriptTarget(Enum):
    TS50 = "5.0"

    @property
    def minimum_version(self) -> str:
        return self.value


class RustTarget(Enum):
    RUST2021 = "2021"
    RUST2024 = "2024"

    @property
    def edition(self) -> str:
        return self.value

    @property
    def minimum_version(self) -> str:
        if self == RustTarget.RUST2021:
            return "1.56"
        return "1.85"

    @property
    def reserves_gen(self) -> bool:
        return self == RustTarget.RUST2024


@dataclass(frozen=True)
class TargetRequirements:
    target: str
    language: Lang
    compiler: str
    minimum_version: str
    language_standard: str
    module_system: str | None
    module_resolution: str | None
    strict: bool | None
    transport: str | None
    runtime_dependencies: tuple[str, ...]


@dataclass(frozen=True)
class TargetProfile:
    python: PythonTarget | None = None
    typescript: TypeScriptTarget | None = None
    rust: RustTarget | None = None

    def __post_init__(self) -> None:
        inhabited = tuple(
            name
            for name, value in (
                ("python", self.python),
                ("typescript", self.typescript),
                ("rust", self.rust),
            )
            if value is not None
        )
        if len(inhabited) != 1:
            raise ValueError(
                "TargetProfile must inhabit exactly one variant, got "
                + (", ".join(inhabited) if inhabited else "none")
            )


KNOWN_TARGET_IDS: tuple[str, ...] = (
    "python-3.11",
    "python-3.12",
    "python-3.13",
    "python-3.14",
    "typescript-5.0",
    "rust-2021",
    "rust-2024",
)


def profile_id(profile: TargetProfile) -> str:
    if profile.python is not None:
        return f"python-{profile.python.value}"
    if profile.typescript is not None:
        return f"typescript-{profile.typescript.value}"
    if profile.rust is not None:
        return f"rust-{profile.rust.value}"
    raise AssertionError("unreachable: TargetProfile invariant guarantees one variant")


def profile_lang(profile: TargetProfile) -> Lang:
    if profile.python is not None:
        return Lang.PY
    if profile.typescript is not None:
        return Lang.TS
    if profile.rust is not None:
        return Lang.RUST
    raise AssertionError("unreachable: TargetProfile invariant guarantees one variant")


def profile_from_id(value: str) -> TargetProfile | UnknownTargetProfile:
    if value == "python-3.11":
        return TargetProfile(python=PythonTarget.PY311)
    if value == "python-3.12":
        return TargetProfile(python=PythonTarget.PY312)
    if value == "python-3.13":
        return TargetProfile(python=PythonTarget.PY313)
    if value == "python-3.14":
        return TargetProfile(python=PythonTarget.PY314)
    if value == "typescript-5.0":
        return TargetProfile(typescript=TypeScriptTarget.TS50)
    if value == "rust-2021":
        return TargetProfile(rust=RustTarget.RUST2021)
    if value == "rust-2024":
        return TargetProfile(rust=RustTarget.RUST2024)
    return UnknownTargetProfile(value)


def default_profile_for(lang: Lang) -> TargetProfile:
    if lang == Lang.PY:
        return TargetProfile(python=PythonTarget.PY311)
    if lang == Lang.TS:
        return TargetProfile(typescript=TypeScriptTarget.TS50)
    if lang == Lang.RUST:
        return TargetProfile(rust=RustTarget.RUST2021)
    raise ValueError(f"Unknown Lang: {lang}")


def profile_requirements(profile: TargetProfile) -> TargetRequirements:
    pid = profile_id(profile)
    lang = profile_lang(profile)
    if profile.python is not None:
        ver = profile.python.value
        return TargetRequirements(
            target=pid,
            language=lang,
            compiler="python",
            minimum_version=ver,
            language_standard=ver,
            module_system=None,
            module_resolution=None,
            strict=None,
            transport="generated-h2c-and-tls-alpn-h2",
            runtime_dependencies=("pydantic>=2",),
        )
    if profile.typescript is not None:
        return TargetRequirements(
            target=pid,
            language=lang,
            compiler="typescript",
            minimum_version="5.0",
            language_standard="ES2022",
            module_system="ESNext",
            module_resolution="Bundler",
            strict=True,
            transport="fetch-or-axios",
            runtime_dependencies=(),
        )
    if profile.rust is not None:
        min_ver = profile.rust.minimum_version
        ed = profile.rust.edition
        return TargetRequirements(
            target=pid,
            language=lang,
            compiler="rustc",
            minimum_version=min_ver,
            language_standard=ed,
            module_system=None,
            module_resolution=None,
            strict=None,
            transport="reqwest-blocking",
            runtime_dependencies=("reqwest", "serde", "serde_json"),
        )
    raise AssertionError("unreachable: TargetProfile invariant guarantees one variant")


POLICY_KEYS: tuple[str, ...] = ("typescript", "python", "rust")


@dataclass(frozen=True)
class TargetPolicy:
    typescript: TargetProfile
    python: TargetProfile
    rust: TargetProfile

    def for_lang(self, lang: Lang) -> TargetProfile:
        if lang == Lang.TS:
            return self.typescript
        if lang == Lang.PY:
            return self.python
        if lang == Lang.RUST:
            return self.rust
        raise ValueError(f"Unknown Lang: {lang}")

    def resolve(
        self, lang: Lang, explicit: str | None
    ) -> TargetProfile | UnknownTargetProfile | TargetLanguageMismatch:
        if explicit is None:
            target = self.for_lang(lang)
        else:
            target = profile_from_id(explicit)
            if isinstance(target, UnknownTargetProfile):
                return target
        if profile_lang(target) != lang:
            return TargetLanguageMismatch(
                profile_id=profile_id(target),
                profile_lang=profile_lang(target),
                requested=lang,
            )
        return target


_KEY_TO_LANG: dict[str, Lang] = {
    "typescript": Lang.TS,
    "python": Lang.PY,
    "rust": Lang.RUST,
}


def policy_from_mapping(
    raw: Mapping[str, str]
) -> TargetPolicy | MissingPolicyKey | UnknownTargetProfile | PolicyLanguageMismatch:
    profiles: dict[str, TargetProfile] = {}
    for key in POLICY_KEYS:
        if key not in raw:
            return MissingPolicyKey(key)
        val = raw[key]
        prof = profile_from_id(val)
        if isinstance(prof, UnknownTargetProfile):
            return prof
        expected_lang = _KEY_TO_LANG[key]
        got_lang = profile_lang(prof)
        if got_lang != expected_lang:
            return PolicyLanguageMismatch(
                key="targets." + key,
                expected=expected_lang,
                got=profile_id(prof),
            )
        profiles[key] = prof
    return TargetPolicy(
        typescript=profiles["typescript"],
        python=profiles["python"],
        rust=profiles["rust"],
    )
