#!/usr/bin/env bash
set -euo pipefail

cargo test -p agentic-workflow regenerability_gaps_are_advisory_when_production_gates_clean -- --nocapture
