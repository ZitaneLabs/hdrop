#!/usr/bin/env bash
set -euo pipefail

container=${1:?Usage: configure-postgres-tls.sh CONTAINER CERT_DIRECTORY}
cert_dir=${2:?Usage: configure-postgres-tls.sh CONTAINER CERT_DIRECTORY}
umask 077
mkdir -p "$cert_dir"
cd "$cert_dir"

for name in ca wrong-ca; do
    openssl req -x509 -newkey rsa:2048 -nodes -days 2 \
        -keyout "$name.key" -out "$name.crt" -subj "/CN=hdrop-test-$name" \
        -addext "basicConstraints=critical,CA:TRUE" -addext "keyUsage=critical,keyCertSign,cRLSign"
done
issue_certificate() {
    local name=$1 cn=$2
    shift 2
    printf '%s\n' 'basicConstraints=critical,CA:FALSE' "$@" > "$name.ext"
    openssl req -new -newkey rsa:2048 -nodes -subj "/CN=$cn" -keyout "$name.key" -out "$name.csr"
    openssl x509 -req -days 2 -in "$name.csr" -out "$name.crt" -extfile "$name.ext" \
        -CA ca.crt -CAkey ca.key -CAcreateserial
}
issue_certificate server localhost extendedKeyUsage=serverAuth subjectAltName=DNS:localhost
issue_certificate client hdrop_tls_client extendedKeyUsage=clientAuth

mkdir -p key-permissions home/postgresql
chmod 700 key-permissions
cp client.key key-permissions/client.key
chmod 644 key-permissions/client.key
docker exec "$container" sh -c 'printf "localhost:*:%s:postgres:%s\n" "$POSTGRES_DB" "$POSTGRES_PASSWORD"' > pgpass
# libpq's default filename differs on Unix and Windows. Only localhost has a valid password.
{ cat pgpass; echo '*:*:*:*:wrong-password'; } > home/.pgpass
cp home/.pgpass home/postgresql/pgpass.conf

remote=/var/lib/postgresql/hdrop-tls
docker exec "$container" mkdir -p "$remote"
for file in ca.crt server.crt server.key; do
    docker cp "$file" "$container:$remote/$file"
done
docker exec "$container" chown -R postgres:postgres "$remote"
docker exec "$container" chmod 600 "$remote/server.key"
docker exec "$container" sh -c \
    'sed -i -e "1i\\hostssl hdrop_test hdrop_tls_client all cert" \
        -e "1i\\hostnossl hdrop_test hdrop_tls_client all reject" \
        -e "1i\\hostssl hdrop_test hdrop_plain_client all reject" \
        -e "1i\\hostnossl hdrop_test hdrop_plain_client all trust" "$PGDATA/pg_hba.conf"'
docker exec -i "$container" psql -U postgres -d hdrop_test -v ON_ERROR_STOP=1 <<SQL
CREATE ROLE hdrop_tls_client LOGIN;
CREATE ROLE hdrop_plain_client LOGIN;
ALTER SYSTEM SET ssl_ca_file = '$remote/ca.crt';
ALTER SYSTEM SET ssl_cert_file = '$remote/server.crt';
ALTER SYSTEM SET ssl_key_file = '$remote/server.key';
ALTER SYSTEM SET ssl = on;
SELECT pg_reload_conf();
SQL

for attempt in {1..20}; do
    if docker exec -e PGSSLMODE=require "$container" sh -c \
        'PGPASSWORD="$POSTGRES_PASSWORD" psql -h localhost -U postgres -d "$POSTGRES_DB" -qc "SELECT 1"' \
        >/dev/null 2>&1; then
        exit 0
    fi
    sleep 1
done
echo 'PostgreSQL did not enable TLS after reloading its configuration' >&2
exit 1
