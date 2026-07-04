#!/bin/bash
# aruaru — PostgreSQL 初期セットアップ
set -euo pipefail

DB_NAME="${ARUARU_DB_NAME:-aruaru}"
DB_USER="${ARUARU_DB_USER:-aruaru}"
DB_PASS="${ARUARU_DB_PASS:-aruaru}"

echo "Creating PostgreSQL role/database: $DB_USER / $DB_NAME"

run_psql() {
  if command -v sudo &>/dev/null && id postgres &>/dev/null 2>&1; then
    sudo -u postgres psql -v ON_ERROR_STOP=1 "$@"
  else
    psql -v ON_ERROR_STOP=1 "$@"
  fi
}

run_psql postgres <<SQL
DO \$\$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '$DB_USER') THEN
    CREATE ROLE $DB_USER LOGIN PASSWORD '$DB_PASS';
  END IF;
END
\$\$;
SELECT 'CREATE DATABASE $DB_NAME OWNER $DB_USER'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$DB_NAME')\gexec
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
SQL

# PostgreSQL 15+ — public スキーマ権限
run_psql -d "$DB_NAME" <<SQL
GRANT ALL ON SCHEMA public TO $DB_USER;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO $DB_USER;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO $DB_USER;
SQL

echo "OK: postgres://${DB_USER}:****@127.0.0.1:5432/${DB_NAME}"
