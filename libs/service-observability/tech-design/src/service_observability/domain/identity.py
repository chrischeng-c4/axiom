from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ServiceIdentity:
    name: str
    version: str


class IdentityError(ValueError):
    pass


def make_identity(name: str, version: str) -> ServiceIdentity:
    if name.strip() == "":
        raise IdentityError("service name must not be blank")
    if version.strip() == "":
        raise IdentityError("service version must not be blank")
    return ServiceIdentity(name=name, version=version)
