"""Exact in-repository consumers of the renamed dependency boundary."""

from dataclasses import dataclass


@dataclass(frozen=True)
class Consumer:
    check_id: str
    manifest: str
    package: str
    dependency_path: str
    command: tuple[str, ...]


CONSUMERS = (
    Consumer("consumer-lumen", "apps/lumen/Cargo.toml", "lumen", "../../libs/openapi-codegen", ("cargo", "check", "-p", "lumen")),
    Consumer("consumer-tape", "apps/tape/Cargo.toml", "tape", "../../libs/openapi-codegen", ("cargo", "check", "-p", "tape")),
    Consumer("consumer-relay", "apps/relay/Cargo.toml", "relay", "../../libs/openapi-codegen", ("cargo", "check", "-p", "relay")),
    Consumer("consumer-keep", "apps/keep/Cargo.toml", "keep", "../../libs/openapi-codegen", ("cargo", "check", "-p", "keep")),
    Consumer("consumer-defer", "apps/defer/Cargo.toml", "defer", "../../libs/openapi-codegen", ("cargo", "check", "-p", "defer")),
    Consumer("consumer-sift", "projects/sift/Cargo.toml", "sift", "../../libs/openapi-codegen", ("cargo", "check", "-p", "sift")),
    Consumer(
        "consumer-client-transport-policy-example",
        "examples/client-transport-policy/Cargo.toml",
        "axiom-client-transport-policy-example",
        "../../libs/openapi-codegen",
        ("cargo", "test", "-p", "axiom-client-transport-policy-example"),
    ),
)

EXPECTED_CHECK_IDS = tuple(consumer.check_id for consumer in CONSUMERS)
DEPENDENCY_NAME = "openapi-codegen"
DEPENDENCY_VERSION = "0.5"
