-- DEV ONLY - Brute Force DROP DB (for local dev and unit test)
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE
 usename = 'pg' OR datname = 'app_db';
DROP DATABASE IF EXISTS app_db;
DROP USER IF EXISTS pg;

-- DEV ONLY - Dev only password (for local dev and unit test).
CREATE USER pg PASSWORD 'pg';
CREATE DATABASE app_db owner pg ENCODING = 'UTF-8';