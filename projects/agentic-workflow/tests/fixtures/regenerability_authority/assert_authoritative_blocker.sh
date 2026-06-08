#!/usr/bin/env bash
set -euo pipefail

cargo test -p agentic-workflow authoritative_regenerability_gaps_block_project_health -- --nocapture
