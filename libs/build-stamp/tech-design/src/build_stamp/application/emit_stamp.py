from __future__ import annotations

from dataclasses import dataclass

from build_stamp.domain.build_time import format_built_at
from build_stamp.domain.directive import (
    Directive,
    DirectiveKind,
    DirectiveRejection,
    make_directive,
    sanitize_key,
)
from build_stamp.domain.fallback import UNKNOWN
from build_stamp.domain.sha import decode_short_sha
from build_stamp.domain.target import decode_target
from build_stamp.infrastructure.ports import (
    ClockSource,
    PathProbe,
    ShaSource,
    TargetSource,
)


@dataclass(frozen=True)
class StampRequest:
    prefix: str
    git_head_path: str


@dataclass(frozen=True)
class StampPlan:
    directives: tuple[Directive, ...]

    def render(self) -> tuple[str, ...]:
        return tuple(d.render() for d in self.directives)


class StampService:
    def __init__(
        self,
        sha: ShaSource,
        clock: ClockSource,
        target: TargetSource,
        probe: PathProbe,
    ) -> None:
        self._sha = sha
        self._clock = clock
        self._target = target
        self._probe = probe

    def plan(self, request: StampRequest) -> StampPlan:
        directives: list[Directive] = []

        # 1. rerun-if-changed hint comes FIRST and only if the path exists
        if self._probe.exists(request.git_head_path):
            d = make_directive(
                DirectiveKind.RERUN_IF_CHANGED, "", request.git_head_path
            )
            if isinstance(d, Directive):
                directives.append(d)

        # 2. the three env stamps, ALWAYS all three, in this order
        success, stdout = self._sha.read_short_sha()
        sha_decoded = decode_short_sha(success, stdout)
        sha_value = sha_decoded if sha_decoded is not None else UNKNOWN

        seconds = self._clock.epoch_seconds()
        built_at = UNKNOWN if seconds is None else format_built_at(seconds)

        target_value = decode_target(self._target.target_triple())

        env_stamps: list[tuple[str, str]] = [
            ("_GIT_SHA", sha_value),
            ("_BUILT_AT", built_at),
            ("_TARGET", target_value),
        ]

        for suffix, val in env_stamps:
            raw_key = request.prefix + suffix
            sanitized_key = sanitize_key(raw_key)

            if raw_key != sanitized_key:
                # Key itself contained control characters -> key was unusable.
                # Strip control characters from key and emit with UNKNOWN.
                d_fallback = make_directive(
                    DirectiveKind.RUSTC_ENV, sanitized_key, UNKNOWN
                )
                if isinstance(d_fallback, Directive):
                    directives.append(d_fallback)
            else:
                d = make_directive(DirectiveKind.RUSTC_ENV, raw_key, val)
                if isinstance(d, DirectiveRejection):
                    # Value was unusable -> emit sanitized key with UNKNOWN
                    d_fallback = make_directive(
                        DirectiveKind.RUSTC_ENV, sanitized_key, UNKNOWN
                    )
                    if isinstance(d_fallback, Directive):
                        directives.append(d_fallback)
                elif isinstance(d, Directive):
                    directives.append(d)

        return StampPlan(directives=tuple(directives))
