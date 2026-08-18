from __future__ import annotations

from raft_runtime.domain.errors import TopologyError, UnsupportedScheme

ALLOWED_SCHEMES: tuple[str, str] = ("http", "https")


def scheme_problem(scheme: str) -> TopologyError | None:
    if scheme in ALLOWED_SCHEMES:
        return None
    return UnsupportedScheme(scheme=scheme, supported=ALLOWED_SCHEMES)


def peer_host(prefix: str, ordinal: int, service: str) -> str:
    return f"{prefix}-{ordinal}.{service}"


def peer_url(
    scheme: str, prefix: str, ordinal: int, service: str, port: int
) -> str | TopologyError:
    problem = scheme_problem(scheme)
    if problem is not None:
        return problem
    host = peer_host(prefix=prefix, ordinal=ordinal, service=service)
    return f"{scheme}://{host}:{port}"
