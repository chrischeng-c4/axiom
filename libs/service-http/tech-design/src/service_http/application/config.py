from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ServiceIdentity:
    name: str
    version: str


@dataclass(frozen=True)
class ObservabilityConfig:
    log_level: str
    log_format: str
    otlp_endpoint: str | None


@dataclass(frozen=True)
class HttpConfig:
    host: str
    port: int
    log_level: str
    log_format: str
    grace_secs: int
    body_limit_bytes: int
    otlp_endpoint: str | None = None


def bind_addr(config: HttpConfig) -> str:
    return f"{config.host}:{config.port}"


def observability_config(config: HttpConfig) -> ObservabilityConfig:
    return ObservabilityConfig(
        log_level=config.log_level,
        log_format=config.log_format,
        otlp_endpoint=config.otlp_endpoint,
    )


def identity_problem(identity: ServiceIdentity) -> str | None:
    if identity.name.strip() == "":
        return "service name must not be blank"
    if identity.version.strip() == "":
        return "service version must not be blank"
    return None
