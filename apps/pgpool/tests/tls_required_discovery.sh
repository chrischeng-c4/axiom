#!/usr/bin/env sh
# SPEC-MANAGED: apps/pgpool/tech-design/logic/prove-managed-discovery-against-tls-required-postgresql.md#unit-test
# HANDWRITE-BEGIN gap="missing-generator:unit-test" tracker="#1924" reason="Docker-backed PostgreSQL TLS fixture setup and cleanup require host shell orchestration."
set -eu

root="$(git rev-parse --show-toplevel)"
port="${PGPOOL_TLS_DISCOVERY_PORT:-55432}"
container="pgpool-tls-discovery-$$"
scratch="$(mktemp -d)"

cleanup() {
    docker rm --force "$container" >/dev/null 2>&1 || true
    rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

docker run --detach --name "$container" \
    --env POSTGRES_HOST_AUTH_METHOD=trust \
    --publish "127.0.0.1:${port}:5432" \
    --entrypoint sh \
    postgres:15 \
    -c '
set -eu
openssl req -new -x509 -nodes -newkey rsa:2048 -days 1 \
    -subj /CN=pgpool-test-ca \
    -addext basicConstraints=critical,CA:TRUE \
    -addext keyUsage=critical,keyCertSign,cRLSign \
    -keyout /tmp/pgpool-tls-ca.key -out /tmp/pgpool-tls-ca.crt
openssl req -new -nodes -newkey rsa:2048 \
    -subj /CN=localhost \
    -addext subjectAltName=DNS:localhost \
    -addext basicConstraints=critical,CA:FALSE \
    -addext keyUsage=digitalSignature,keyEncipherment \
    -addext extendedKeyUsage=serverAuth \
    -keyout /tmp/pgpool-tls.key -out /tmp/pgpool-tls.csr
openssl x509 -req -in /tmp/pgpool-tls.csr \
    -CA /tmp/pgpool-tls-ca.crt -CAkey /tmp/pgpool-tls-ca.key -CAcreateserial \
    -days 1 -copy_extensions copy -out /tmp/pgpool-tls.crt
printf "local all all trust\\nhostssl all all all trust\\n" > /tmp/pgpool-tls-hba.conf
chown postgres:postgres /tmp/pgpool-tls.key /tmp/pgpool-tls.crt /tmp/pgpool-tls-hba.conf
chmod 600 /tmp/pgpool-tls.key
exec /usr/local/bin/docker-entrypoint.sh postgres \
    -c ssl=on \
    -c ssl_cert_file=/tmp/pgpool-tls.crt \
    -c ssl_key_file=/tmp/pgpool-tls.key \
    -c hba_file=/tmp/pgpool-tls-hba.conf
'

attempt=0
while [ "$attempt" -lt 60 ]; do
    if docker logs "$container" 2>&1 | grep -q 'PostgreSQL init process complete; ready for start up.'; then
        break
    fi
    sleep 1
    attempt=$((attempt + 1))
done
if [ "$attempt" -eq 60 ]; then
    docker logs "$container"
    echo "TLS-required PostgreSQL did not finish initialization" >&2
    exit 1
fi
sleep 2

docker cp "$container:/tmp/pgpool-tls-ca.crt" "$scratch/ca.pem"

if PGSSLMODE=disable psql \
    "host=127.0.0.1 port=${port} user=postgres dbname=postgres connect_timeout=3" \
    --command 'SELECT 1' >/dev/null 2>&1; then
    echo "TLS-required PostgreSQL accepted a plaintext client" >&2
    exit 1
fi

cd "$root"
PGPOOL_TLS_DISCOVERY_PORT="$port" \
PGPOOL_TLS_DISCOVERY_CA="$scratch/ca.pem" \
cargo test -p pgpool --test connection_discovery \
    cloudsql_discovery_succeeds_against_tls_required_postgres -- --exact
# HANDWRITE-END
