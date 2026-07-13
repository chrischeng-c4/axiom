#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:logic:125bf141" tracker="#1597" reason="Run an identical simple-protocol pgbench workload through PgBouncer and pgpool."
set -euo pipefail

readonly PROFILE_SCHEMA="pgpool.pgbouncer-baseline.v1"
readonly BACKEND_CAP=16
readonly CLIENTS=64
readonly JOBS=4
readonly DURATION_SECONDS=30
readonly SCALE=1
readonly DATABASE="pgpool_bench"
readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"

PGPOOL_BIN="${PGPOOL_BIN:-$REPO_ROOT/target/release/pgpool}"
DRY_RUN=false
WORK_DIR=""
POSTGRES_STARTED=false
PGBOUNCER_PID=""
PGPOOL_PID=""
USED_PORTS=()

usage() {
    cat <<'USAGE'
Usage: run.sh [--dry-run] [--pgpool-bin PATH]

Compares PgBouncer and pgpool transaction pooling against one temporary local
PostgreSQL backend. `--dry-run` prints the immutable profile JSON and does not
inspect the machine, create files, bind ports, or start processes.
USAGE
}

emit_dry_run_profile() {
    cat <<JSON
{"schema":"$PROFILE_SCHEMA","profile":{"workload":"pgbench-tpcb","protocol":"simple","pool_mode":"transaction","backend_connection_cap":$BACKEND_CAP,"clients":$CLIENTS,"jobs":$JOBS,"duration_seconds":$DURATION_SECONDS,"scale":$SCALE},"targets":{"pgbouncer":{"pool_mode":"transaction","backend_connection_cap":$BACKEND_CAP},"pgpool":{"pool_mode":"transaction","backend_connection_cap":$BACKEND_CAP}}}
JSON
}

fail() {
    echo "error: $*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || fail "required command '$1' was not found"
}

find_free_port() {
    local port
    for port in $(seq 55432 55531); do
        if [[ " ${USED_PORTS[*]-} " == *" $port "* ]]; then
            continue
        fi
        if ! lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
            USED_PORTS+=("$port")
            printf '%s\n' "$port"
            return 0
        fi
    done
    fail "could not find an unused TCP port in 55432-55531"
}

wait_for_sql() {
    local port="$1"
    local label="$2"
    local _
    for _ in $(seq 1 100); do
        if PGCONNECT_TIMEOUT=1 psql --host 127.0.0.1 --port "$port" --username postgres --dbname "$DATABASE" --no-psqlrc --quiet --command 'SELECT 1' >/dev/null 2>&1; then
            return 0
        fi
        sleep 0.1
    done
    fail "$label did not accept SQL connections; inspect $WORK_DIR"
}

metric() {
    local report="$1"
    local field="$2"
    case "$field" in
        tps) awk '/^tps =/ { print $3; exit }' "$report" ;;
        latency_ms) awk '/^latency average =/ { print $4; exit }' "$report" ;;
        *) fail "unknown metric '$field'" ;;
    esac
}

require_metric() {
    local value="$1"
    local name="$2"
    [[ "$value" =~ ^[0-9]+([.][0-9]+)?$ ]] || fail "could not parse $name from pgbench output"
    awk -v value="$value" 'BEGIN { exit !(value > 0) }' || fail "$name must be positive, got '$value'"
}

cleanup() {
    set +e
    if [[ -n "$PGPOOL_PID" ]]; then
        kill "$PGPOOL_PID" 2>/dev/null
        wait "$PGPOOL_PID" 2>/dev/null
    fi
    if [[ -n "$PGBOUNCER_PID" ]]; then
        kill "$PGBOUNCER_PID" 2>/dev/null
        wait "$PGBOUNCER_PID" 2>/dev/null
    fi
    if [[ "$POSTGRES_STARTED" == true ]]; then
        pg_ctl --pgdata "$WORK_DIR/postgres" --wait --mode fast stop >/dev/null 2>&1
    fi
    [[ -z "$WORK_DIR" ]] || rm -rf "$WORK_DIR"
}

while (($#)); do
    case "$1" in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --pgpool-bin)
            (($# >= 2)) || fail "--pgpool-bin requires a path"
            PGPOOL_BIN="$2"
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            fail "unknown argument '$1'; use --help"
            ;;
    esac
done

if [[ "$DRY_RUN" == true ]]; then
    emit_dry_run_profile
    exit 0
fi

for command in initdb pg_ctl psql pgbench pgbouncer lsof; do
    require_command "$command"
done
[[ -x "$PGPOOL_BIN" ]] || fail "pgpool binary is not executable: $PGPOOL_BIN (build with: cargo build --release -p pgpool)"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/pgpool-pgbouncer-benchmark.XXXXXX")"
trap cleanup EXIT INT TERM

POSTGRES_PORT="$(find_free_port)"
PGBOUNCER_PORT="$(find_free_port)"
PGPOOL_PORT="$(find_free_port)"
ADMIN_PORT="$(find_free_port)"

initdb --pgdata "$WORK_DIR/postgres" --auth trust --username postgres --no-locale >"$WORK_DIR/initdb.log" 2>&1
pg_ctl --pgdata "$WORK_DIR/postgres" --options "-h 127.0.0.1 -p $POSTGRES_PORT" --wait start >"$WORK_DIR/postgres.log" 2>&1
POSTGRES_STARTED=true

psql --host 127.0.0.1 --port "$POSTGRES_PORT" --username postgres --dbname postgres --no-psqlrc --quiet --command "CREATE DATABASE $DATABASE" >"$WORK_DIR/create-db.log"
pgbench --initialize --scale "$SCALE" --host 127.0.0.1 --port "$POSTGRES_PORT" --username postgres "$DATABASE" >"$WORK_DIR/pgbench-init.log" 2>&1

# Warm the identical seeded backend before either target is measured.
pgbench --no-vacuum --protocol simple --client 8 --jobs 2 --time 3 --host 127.0.0.1 --port "$POSTGRES_PORT" --username postgres "$DATABASE" >"$WORK_DIR/warmup.log" 2>&1

cat >"$WORK_DIR/pgbouncer.ini" <<CONFIG
[databases]
$DATABASE = host=127.0.0.1 port=$POSTGRES_PORT dbname=$DATABASE

[pgbouncer]
listen_addr = 127.0.0.1
listen_port = $PGBOUNCER_PORT
auth_type = trust
pool_mode = transaction
max_client_conn = 1000
default_pool_size = $BACKEND_CAP
server_reset_query = DISCARD ALL
ignore_startup_parameters = extra_float_digits
admin_users = postgres
pidfile = $WORK_DIR/pgbouncer.pid
logfile = $WORK_DIR/pgbouncer.log
CONFIG

pgbouncer "$WORK_DIR/pgbouncer.ini" >"$WORK_DIR/pgbouncer.stdout" 2>&1 &
PGBOUNCER_PID=$!
wait_for_sql "$PGBOUNCER_PORT" "PgBouncer"

"$PGPOOL_BIN" serve \
    --backend-host 127.0.0.1 \
    --backend-port "$POSTGRES_PORT" \
    --bind "127.0.0.1:$PGPOOL_PORT" \
    --admin-bind "127.0.0.1:$ADMIN_PORT" \
    --max-backend-connections "$BACKEND_CAP" \
    >"$WORK_DIR/pgpool.log" 2>&1 &
PGPOOL_PID=$!
wait_for_sql "$PGPOOL_PORT" "pgpool"

pgbench --no-vacuum --protocol simple --client "$CLIENTS" --jobs "$JOBS" --time "$DURATION_SECONDS" --host 127.0.0.1 --port "$PGBOUNCER_PORT" --username postgres "$DATABASE" >"$WORK_DIR/pgbouncer-pgbench.log" 2>&1
pgbench --no-vacuum --protocol simple --client "$CLIENTS" --jobs "$JOBS" --time "$DURATION_SECONDS" --host 127.0.0.1 --port "$PGPOOL_PORT" --username postgres "$DATABASE" >"$WORK_DIR/pgpool-pgbench.log" 2>&1

PGBOUNCER_TPS="$(metric "$WORK_DIR/pgbouncer-pgbench.log" tps)"
PGBOUNCER_LATENCY_MS="$(metric "$WORK_DIR/pgbouncer-pgbench.log" latency_ms)"
PGPOOL_TPS="$(metric "$WORK_DIR/pgpool-pgbench.log" tps)"
PGPOOL_LATENCY_MS="$(metric "$WORK_DIR/pgpool-pgbench.log" latency_ms)"
require_metric "$PGBOUNCER_TPS" "PgBouncer TPS"
require_metric "$PGBOUNCER_LATENCY_MS" "PgBouncer latency"
require_metric "$PGPOOL_TPS" "pgpool TPS"
require_metric "$PGPOOL_LATENCY_MS" "pgpool latency"

TPS_RATIO="$(awk -v pgpool="$PGPOOL_TPS" -v pgbouncer="$PGBOUNCER_TPS" 'BEGIN { printf "%.6f", pgpool / pgbouncer }')"
WINNER="$(awk -v pgpool="$PGPOOL_TPS" -v pgbouncer="$PGBOUNCER_TPS" 'BEGIN { print (pgpool > pgbouncer ? "pgpool" : (pgpool < pgbouncer ? "pgbouncer" : "tie")) }')"

printf '{"schema":"%s","profile":{"workload":"pgbench-tpcb","protocol":"simple","pool_mode":"transaction","backend_connection_cap":%s,"clients":%s,"jobs":%s,"duration_seconds":%s,"scale":%s},"targets":{"pgbouncer":{"tps":%s,"latency_average_ms":%s},"pgpool":{"tps":%s,"latency_average_ms":%s}},"ratios":{"pgpool_over_pgbouncer_tps":%s},"winner_by_tps":"%s"}\n' \
    "$PROFILE_SCHEMA" "$BACKEND_CAP" "$CLIENTS" "$JOBS" "$DURATION_SECONDS" "$SCALE" \
    "$PGBOUNCER_TPS" "$PGBOUNCER_LATENCY_MS" "$PGPOOL_TPS" "$PGPOOL_LATENCY_MS" "$TPS_RATIO" "$WINNER"
# HANDWRITE-END
