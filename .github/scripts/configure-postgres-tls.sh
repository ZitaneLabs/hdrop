#!/usr/bin/env bash
set -euo pipefail

container=${1:?Usage: configure-postgres-tls.sh CONTAINER CERT_DIRECTORY}
cert_dir=${2:?Usage: configure-postgres-tls.sh CONTAINER CERT_DIRECTORY}
umask 077
mkdir -p "$cert_dir"

for name in ca wrong-ca; do
    openssl req -x509 -newkey rsa:2048 -nodes -days 2 \
        -keyout "$cert_dir/$name.key" -out "$cert_dir/$name.crt" \
        -subj "/CN=hdrop-test-$name" \
        -addext "basicConstraints=critical,CA:TRUE" \
        -addext "keyUsage=critical,keyCertSign,cRLSign"
done
openssl req -new -newkey rsa:2048 -nodes \
    -keyout "$cert_dir/server.key" -out "$cert_dir/server.csr" \
    -subj "/CN=localhost"
printf '%s\n' \
    'subjectAltName=DNS:localhost' \
    'extendedKeyUsage=serverAuth' \
    'basicConstraints=critical,CA:FALSE' > "$cert_dir/server.ext"
openssl x509 -req -days 2 -in "$cert_dir/server.csr" \
    -CA "$cert_dir/ca.crt" -CAkey "$cert_dir/ca.key" -CAcreateserial \
    -out "$cert_dir/server.crt" -extfile "$cert_dir/server.ext"
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 \
    -out "$cert_dir/client.key"
openssl req -new -key "$cert_dir/client.key" -out "$cert_dir/client.csr" \
    -subj "/CN=hdrop_tls_client"
printf '%s\n' \
    'extendedKeyUsage=clientAuth' \
    'basicConstraints=critical,CA:FALSE' > "$cert_dir/client.ext"
openssl x509 -req -days 2 -in "$cert_dir/client.csr" \
    -CA "$cert_dir/ca.crt" -CAkey "$cert_dir/ca.key" -CAcreateserial \
    -out "$cert_dir/client.crt" -extfile "$cert_dir/client.ext"

docker exec "$container" mkdir -p /var/lib/postgresql/hdrop-tls
docker cp "$cert_dir/ca.crt" "$container:/var/lib/postgresql/hdrop-tls/ca.crt"
docker cp "$cert_dir/server.crt" "$container:/var/lib/postgresql/hdrop-tls/server.crt"
docker cp "$cert_dir/server.key" "$container:/var/lib/postgresql/hdrop-tls/server.key"
docker exec "$container" chown postgres:postgres \
    /var/lib/postgresql/hdrop-tls \
    /var/lib/postgresql/hdrop-tls/ca.crt \
    /var/lib/postgresql/hdrop-tls/server.crt \
    /var/lib/postgresql/hdrop-tls/server.key
docker exec "$container" chmod 600 /var/lib/postgresql/hdrop-tls/server.key
docker exec "$container" sh -c \
    'sed -i "1i\\hostssl hdrop_test hdrop_tls_client all cert" "$PGDATA/pg_hba.conf"'
docker exec "$container" psql -U postgres -d hdrop_test -v ON_ERROR_STOP=1 \
    -c 'CREATE ROLE hdrop_tls_client LOGIN' \
    -c 'GRANT CONNECT ON DATABASE hdrop_test TO hdrop_tls_client' \
    -c "ALTER SYSTEM SET ssl_ca_file = '/var/lib/postgresql/hdrop-tls/ca.crt'" \
    -c "ALTER SYSTEM SET ssl_cert_file = '/var/lib/postgresql/hdrop-tls/server.crt'" \
    -c "ALTER SYSTEM SET ssl_key_file = '/var/lib/postgresql/hdrop-tls/server.key'" \
    -c "ALTER SYSTEM SET ssl = on" \
    -c 'SELECT pg_reload_conf()'

for attempt in {1..20}; do
    if [[ $(docker exec -e PGSSLMODE=require "$container" sh -c \
        'PGPASSWORD="$POSTGRES_PASSWORD" psql -h localhost -U postgres -d "$POSTGRES_DB" -Atqc "SELECT ssl FROM pg_stat_ssl WHERE pid = pg_backend_pid()"' \
        2>/dev/null) == t ]]; then
        exit 0
    fi
    sleep 1
done
echo 'PostgreSQL did not enable TLS after reloading its configuration' >&2
exit 1
