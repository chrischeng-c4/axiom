#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:logic:125bf141" tracker="#1597" reason="Run an identical simple-protocol pgbench workload through PgBouncer and pgpool."
set -euo pipefail

readonly PROFILE_SCHEMA="pgpool.pgbouncer-baseline.v1"
readonly BACKEND_CAP=16
readonly CLIENTS=64
readonly JOBS=4
readonly DURATION_SECONDS=30
readonly METER_DURATION_CAP_SECONDS=$((DURATION_SECONDS + 30))
readonly SCALE=1
readonly POOL_ACQUIRE_TIMEOUT_MS=60000
readonly DATABASE="pgpool_bench"
readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"

PGPOOL_BIN="${PGPOOL_BIN:-$REPO_ROOT/target/release/pgpool}"
METER_BIN=""
DRY_RUN=false
WORK_DIR=""
POSTGRES_STARTED=false
PGBOUNCER_PID=""
PGPOOL_PID=""
USED_PORTS=()
NEXT_FREE_PORT=""
KEEP_WORK_DIR="${PGPOOL_BENCH_KEEP_WORK_DIR:-false}"

usage() {
    cat <<'USAGE'
Usage: run.sh [--dry-run] [--pgpool-bin PATH] [--meter-bin PATH]

Compares PgBouncer and pgpool transaction pooling against one temporary local
PostgreSQL backend. `--dry-run` prints the immutable profile JSON and does not
inspect the machine, create files, bind ports, or start processes.

`--meter-bin` is an opt-in pgpool-only diagnostic: meter launches and samples
the pgpool process while its opaque driver runs the same pgbench leg. It retains
the temporary work directory with the meter JSON and folded stacks, and labels
the resulting comparison as diagnostic-only rather than win evidence.
USAGE
}

emit_dry_run_profile() {
    cat <<JSON
{"schema":"$PROFILE_SCHEMA","profile":{"workload":"pgbench-tpcb","protocol":"simple","pool_mode":"transaction","backend_connection_cap":$BACKEND_CAP,"clients":$CLIENTS,"jobs":$JOBS,"duration_seconds":$DURATION_SECONDS,"scale":$SCALE,"pool_acquire_timeout_ms":$POOL_ACQUIRE_TIMEOUT_MS},"targets":{"pgbouncer":{"pool_mode":"transaction","backend_connection_cap":$BACKEND_CAP},"pgpool":{"pool_mode":"transaction","backend_connection_cap":$BACKEND_CAP,"pool_acquire_timeout_ms":$POOL_ACQUIRE_TIMEOUT_MS}}}
JSON
}

fail() {
    echo "error: $*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || fail "required command '$1' was not found"
}

absolute_executable_path() {
    local path="$1"
    local directory
    directory="$(cd "$(dirname "$path")" && pwd)"
    printf '%s/%s\n' "$directory" "$(basename "$path")"
}

find_free_port() {
    local port
    for port in $(seq 55432 55531); do
        if [[ " ${USED_PORTS[*]-} " == *" $port "* ]]; then
            continue
        fi
        if ! lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
            USED_PORTS+=("$port")
            NEXT_FREE_PORT="$port"
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

reported_clients() {
    awk -F: '/^number of clients:/ { gsub(/^[[:space:]]+|[[:space:]]+$/, "", $2); print $2; exit }' "$1"
}

require_metric() {
    local value="$1"
    local name="$2"
    [[ "$value" =~ ^[0-9]+([.][0-9]+)?$ ]] || fail "could not parse $name from pgbench output"
    awk -v value="$value" 'BEGIN { exit !(value > 0) }' || fail "$name must be positive, got '$value'"
}

require_client_count() {
    local value="$1"
    local target="$2"
    [[ "$value" == "$CLIENTS" ]] || fail "$target benchmark did not establish all $CLIENTS clients (reported '$value')"
}

require_clean_pgbench() {
    local report="$1"
    local target="$2"
    if grep -q '^pgbench: error:' "$report"; then
        fail "$target benchmark reported client errors; refusing a partial-workload comparison"
    fi
}

write_meter_driver() {
    cat >"$WORK_DIR/pgpool-meter-drive.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

for _ in $(seq 1 100); do
    if PGCONNECT_TIMEOUT=1 psql --host "$PGPOOL_BENCH_PGPOOL_HOST" --port "$PGPOOL_BENCH_PGPOOL_PORT" --username postgres --dbname "$PGPOOL_BENCH_DATABASE" --no-psqlrc --quiet --command 'SELECT 1' >/dev/null 2>&1; then
        exec pgbench --no-vacuum --protocol simple --client "$PGPOOL_BENCH_CLIENTS" --jobs "$PGPOOL_BENCH_JOBS" --time "$PGPOOL_BENCH_DURATION_SECONDS" --host "$PGPOOL_BENCH_PGPOOL_HOST" --port "$PGPOOL_BENCH_PGPOOL_PORT" --username postgres "$PGPOOL_BENCH_DATABASE" >"$PGPOOL_BENCH_PGPOOL_PGBENCH_LOG" 2>&1
    fi
    sleep 0.1
done

echo "pgpool did not accept SQL connections for meter diagnostic" >&2
exit 1
SCRIPT
    chmod +x "$WORK_DIR/pgpool-meter-drive.sh"
}

require_meter_artifacts() {
    local collapsed=()
    [[ -s "$WORK_DIR/meter-report.json" ]] || fail "meter did not produce its JSON report"
    [[ -s "$WORK_DIR/.meter/last-report.json" ]] || fail "meter did not persist .meter/last-report.json"
    grep -q '"schema_version": "meter.report/1"' "$WORK_DIR/meter-report.json" || fail "meter report did not use meter.report/1"
    shopt -s nullglob
    collapsed=("$WORK_DIR/.meter/"*.collapsed)
    shopt -u nullglob
    ((${#collapsed[@]} > 0)) || fail "meter did not produce collapsed stack evidence"
}

run_pgpool_benchmark() {
    if [[ -z "$METER_BIN" ]]; then
        "$PGPOOL_BIN" serve \
            --backend-host 127.0.0.1 \
            --backend-port "$POSTGRES_PORT" \
            --bind "127.0.0.1:$PGPOOL_PORT" \
            --admin-bind "127.0.0.1:$ADMIN_PORT" \
            --max-backend-connections "$BACKEND_CAP" \
            --pool-acquire-timeout-ms "$POOL_ACQUIRE_TIMEOUT_MS" \
            >"$WORK_DIR/pgpool.log" 2>&1 &
        PGPOOL_PID=$!
        wait_for_sql "$PGPOOL_PORT" "pgpool"
        pgbench --no-vacuum --protocol simple --client "$CLIENTS" --jobs "$JOBS" --time "$DURATION_SECONDS" --host 127.0.0.1 --port "$PGPOOL_PORT" --username postgres "$DATABASE" >"$WORK_DIR/pgpool-pgbench.log" 2>&1
        return
    fi

    write_meter_driver
    KEEP_WORK_DIR=true
    (
        cd "$WORK_DIR"
        PGPOOL_BENCH_PGPOOL_HOST=127.0.0.1 \
        PGPOOL_BENCH_PGPOOL_PORT="$PGPOOL_PORT" \
        PGPOOL_BENCH_DATABASE="$DATABASE" \
        PGPOOL_BENCH_CLIENTS="$CLIENTS" \
        PGPOOL_BENCH_JOBS="$JOBS" \
        PGPOOL_BENCH_DURATION_SECONDS="$DURATION_SECONDS" \
        PGPOOL_BENCH_PGPOOL_PGBENCH_LOG="$WORK_DIR/pgpool-pgbench.log" \
        "$METER_BIN" measure "$PGPOOL_BIN" --level sample --duration-cap "$METER_DURATION_CAP_SECONDS" --drive ./pgpool-meter-drive.sh -- serve \
            --backend-host 127.0.0.1 \
            --backend-port "$POSTGRES_PORT" \
            --bind "127.0.0.1:$PGPOOL_PORT" \
            --admin-bind "127.0.0.1:$ADMIN_PORT" \
            --max-backend-connections "$BACKEND_CAP" \
            --pool-acquire-timeout-ms "$POOL_ACQUIRE_TIMEOUT_MS" \
            >"$WORK_DIR/meter-report.json" 2>"$WORK_DIR/meter.log"
    )
    require_meter_artifacts
    echo "meter sampled pgpool only; its comparison result is diagnostic-only, not win evidence" >&2
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
    if [[ -n "$WORK_DIR" && "$KEEP_WORK_DIR" == true ]]; then
        echo "benchmark artifacts retained at $WORK_DIR" >&2
    elif [[ -n "$WORK_DIR" ]]; then
        rm -rf "$WORK_DIR"
    fi
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
        --meter-bin)
            (($# >= 2)) || fail "--meter-bin requires a path"
            METER_BIN="$2"
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
PGPOOL_BIN="$(absolute_executable_path "$PGPOOL_BIN")"
if [[ -n "$METER_BIN" ]]; then
    [[ -x "$METER_BIN" ]] || fail "meter binary is not executable: $METER_BIN (build with: cargo build -p meter-cli --bin meter)"
    METER_BIN="$(absolute_executable_path "$METER_BIN")"
fi

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/pgpool-pgbouncer-benchmark.XXXXXX")"
trap cleanup EXIT INT TERM

find_free_port
POSTGRES_PORT="$NEXT_FREE_PORT"
find_free_port
PGBOUNCER_PORT="$NEXT_FREE_PORT"
find_free_port
PGPOOL_PORT="$NEXT_FREE_PORT"
find_free_port
ADMIN_PORT="$NEXT_FREE_PORT"

initdb --pgdata "$WORK_DIR/postgres" --auth trust --username postgres --no-locale >"$WORK_DIR/initdb.log" 2>&1
pg_ctl --pgdata "$WORK_DIR/postgres" --options "-h 127.0.0.1 -p $POSTGRES_PORT" --wait start >"$WORK_DIR/postgres.log" 2>&1
POSTGRES_STARTED=true

psql --host 127.0.0.1 --port "$POSTGRES_PORT" --username postgres --dbname postgres --no-psqlrc --quiet --command "CREATE DATABASE $DATABASE" >"$WORK_DIR/create-db.log"
pgbench --initialize --scale "$SCALE" --host 127.0.0.1 --port "$POSTGRES_PORT" --username postgres "$DATABASE" >"$WORK_DIR/pgbench-init.log" 2>&1

# PgBouncer 1.25 still requires a recognized login role when auth_type=trust.
# The ephemeral PostgreSQL cluster itself is trust-authenticated, so the
# temporary userlist deliberately contains no reusable credential material.
printf '"postgres" ""\n' >"$WORK_DIR/userlist.txt"

# Warm the identical seeded backend before either target is measured.
pgbench --no-vacuum --protocol simple --client 8 --jobs 2 --time 3 --host 127.0.0.1 --port "$POSTGRES_PORT" --username postgres "$DATABASE" >"$WORK_DIR/warmup.log" 2>&1

cat >"$WORK_DIR/pgbouncer.ini" <<CONFIG
[databases]
$DATABASE = host=127.0.0.1 port=$POSTGRES_PORT dbname=$DATABASE

[pgbouncer]
listen_addr = 127.0.0.1
listen_port = $PGBOUNCER_PORT
auth_type = trust
auth_file = $WORK_DIR/userlist.txt
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

pgbench --no-vacuum --protocol simple --client "$CLIENTS" --jobs "$JOBS" --time "$DURATION_SECONDS" --host 127.0.0.1 --port "$PGBOUNCER_PORT" --username postgres "$DATABASE" >"$WORK_DIR/pgbouncer-pgbench.log" 2>&1
run_pgpool_benchmark

PGBOUNCER_TPS="$(metric "$WORK_DIR/pgbouncer-pgbench.log" tps)"
PGBOUNCER_LATENCY_MS="$(metric "$WORK_DIR/pgbouncer-pgbench.log" latency_ms)"
PGPOOL_TPS="$(metric "$WORK_DIR/pgpool-pgbench.log" tps)"
PGPOOL_LATENCY_MS="$(metric "$WORK_DIR/pgpool-pgbench.log" latency_ms)"
PGBOUNCER_CLIENTS="$(reported_clients "$WORK_DIR/pgbouncer-pgbench.log")"
PGPOOL_CLIENTS="$(reported_clients "$WORK_DIR/pgpool-pgbench.log")"
require_metric "$PGBOUNCER_TPS" "PgBouncer TPS"
require_metric "$PGBOUNCER_LATENCY_MS" "PgBouncer latency"
require_metric "$PGPOOL_TPS" "pgpool TPS"
require_metric "$PGPOOL_LATENCY_MS" "pgpool latency"
require_client_count "$PGBOUNCER_CLIENTS" "PgBouncer"
require_client_count "$PGPOOL_CLIENTS" "pgpool"
require_clean_pgbench "$WORK_DIR/pgbouncer-pgbench.log" "PgBouncer"
require_clean_pgbench "$WORK_DIR/pgpool-pgbench.log" "pgpool"

TPS_RATIO="$(awk -v pgpool="$PGPOOL_TPS" -v pgbouncer="$PGBOUNCER_TPS" 'BEGIN { printf "%.6f", pgpool / pgbouncer }')"
WINNER="$(awk -v pgpool="$PGPOOL_TPS" -v pgbouncer="$PGBOUNCER_TPS" 'BEGIN { print (pgpool > pgbouncer ? "pgpool" : (pgpool < pgbouncer ? "pgbouncer" : "tie")) }')"
DIAGNOSTICS_JSON=""
if [[ -n "$METER_BIN" ]]; then
    DIAGNOSTICS_JSON=',"diagnostics":{"meter_sampled_pgpool":true,"comparison_valid":false}'
fi

printf '{"schema":"%s","profile":{"workload":"pgbench-tpcb","protocol":"simple","pool_mode":"transaction","backend_connection_cap":%s,"clients":%s,"jobs":%s,"duration_seconds":%s,"scale":%s},"targets":{"pgbouncer":{"tps":%s,"latency_average_ms":%s},"pgpool":{"tps":%s,"latency_average_ms":%s}},"ratios":{"pgpool_over_pgbouncer_tps":%s},"winner_by_tps":"%s"%s}\n' \
    "$PROFILE_SCHEMA" "$BACKEND_CAP" "$CLIENTS" "$JOBS" "$DURATION_SECONDS" "$SCALE" \
    "$PGBOUNCER_TPS" "$PGBOUNCER_LATENCY_MS" "$PGPOOL_TPS" "$PGPOOL_LATENCY_MS" "$TPS_RATIO" "$WINNER" "$DIAGNOSTICS_JSON"
# HANDWRITE-END
