#!/usr/bin/env python3
"""Build the independent XOR-SHA256 digest for a Sift load phase."""

from __future__ import annotations

import argparse
import hashlib


SIGNALS = (("log", 5), ("metric", 3), ("span", 2))


def xor_digest(target: bytearray, value: bytes) -> None:
    for index, byte in enumerate(value):
        target[index] ^= byte


def phase_digest(phase: str, duration: int, batch_items: int) -> str:
    digest = bytearray(32)
    for signal, qps in SIGNALS:
        prefix = f"{phase}-{signal}-".encode()
        for request in range(qps * duration):
            request_prefix = prefix + f"{request:016x}-".encode()
            for item in range(batch_items):
                xor_digest(digest, hashlib.sha256(request_prefix + str(item).encode()).digest())
    return digest.hex()


def parse_digest(value: str) -> bytes:
    try:
        decoded = bytes.fromhex(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("digest must be lowercase hexadecimal") from error
    if len(decoded) != 32 or value != value.lower():
        raise argparse.ArgumentTypeError("digest must be 64 lowercase hexadecimal characters")
    return decoded


def main() -> None:
    parser = argparse.ArgumentParser()
    subcommands = parser.add_subparsers(dest="command", required=True)

    phase = subcommands.add_parser("phase")
    phase.add_argument("--name", choices=("steady", "failover"), required=True)
    phase.add_argument("--duration", type=int, required=True)
    phase.add_argument("--batch-items", type=int, default=1000)

    xor = subcommands.add_parser("xor")
    xor.add_argument("digests", type=parse_digest, nargs="+")

    args = parser.parse_args()
    if args.command == "phase":
        if args.duration <= 0 or args.batch_items <= 0:
            parser.error("duration and batch-items must be positive")
        print(phase_digest(args.name, args.duration, args.batch_items))
        return

    digest = bytearray(32)
    for value in args.digests:
        xor_digest(digest, value)
    print(digest.hex())


if __name__ == "__main__":
    main()
