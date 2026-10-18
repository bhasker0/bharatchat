#!/usr/bin/env python3
"""
BharatChat Sovereign Platform - Local Database Initializer
Executes PostgreSQL DDL migrations against the local bharatchat-postgres container.
"""

import subprocess
import os
import sys
import time

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SQL_FILE = os.path.join(REPO_ROOT, "scripts", "init_local_db.sql")

def init_postgres():
    print("======================================================================")
    print("   BHARATCHAT LOCAL DATABASE SCHEMA PROVISIONER")
    print("   Target: Local PostgreSQL 16 (bharatchat-postgres on 127.0.0.1:5432)")
    print("======================================================================")

    # 1. Check if container is running
    res = subprocess.run(["podman", "ps", "--filter", "name=bharatchat-postgres", "--format", "{{.Names}}"], capture_output=True, text=True)
    if "bharatchat-postgres" not in res.stdout:
        print("[ERROR] Container 'bharatchat-postgres' is not running.")
        print("Run './scripts/server_ctl.sh start' first to start database services.")
        sys.exit(1)

    # 2. Wait until PostgreSQL is ready to accept connections
    print("[1/3] Waiting for PostgreSQL readiness...")
    ready = False
    for i in range(15):
        chk = subprocess.run(["podman", "exec", "bharatchat-postgres", "pg_isready", "-U", "bharat_admin", "-d", "bharatchat"], capture_output=True, text=True)
        if chk.returncode == 0:
            ready = True
            break
        time.sleep(1)

    if not ready:
        print("[ERROR] PostgreSQL is not accepting connections after 15 seconds.")
        sys.exit(1)
    print("      [OK] PostgreSQL is healthy and accepting connections.")

    # 3. Execute DDL script
    print("[2/3] Executing schema DDL migrations from scripts/init_local_db.sql...")
    with open(SQL_FILE, "r", encoding="utf-8") as f:
        sql_content = f.read()

    proc = subprocess.run(
        ["podman", "exec", "-i", "bharatchat-postgres", "psql", "-U", "bharat_admin", "-d", "bharatchat"],
        input=sql_content,
        capture_output=True,
        text=True
    )
    if proc.returncode != 0:
        print(f"[ERROR] Migration failed:\n{proc.stderr}")
        sys.exit(1)
    print("      [OK] Schema successfully applied.")

    # 4. Verify table list
    print("[3/3] Inspecting created relational tables in 'bharatchat' database...")
    verify_cmd = "SELECT table_name FROM information_schema.tables WHERE table_schema='public' ORDER BY table_name;"
    v_proc = subprocess.run(
        ["podman", "exec", "bharatchat-postgres", "psql", "-U", "bharat_admin", "-d", "bharatchat", "-t", "-c", verify_cmd],
        capture_output=True,
        text=True
    )
    tables = [t.strip() for t in v_proc.stdout.splitlines() if t.strip()]
    print(f"      Discovered {len(tables)} active tables: {tables}")
    expected_tables = [
        "abuse_grievance_tickets",
        "broadcast_channels",
        "call_logs",
        "channel_posts",
        "chat_messages",
        "commerce_orders",
        "consent_records",
        "forwarding_ledger",
        "identities",
        "media_chunks"
    ]
    for exp in expected_tables:
        assert exp in tables, f"Expected table '{exp}' not found in database!"
        print(f"      [OK] Table '{exp}' verified.")

    print("======================================================================")
    print(" [SUCCESS] Local BharatChat Database successfully provisioned and ready!")
    print("======================================================================")

if __name__ == "__main__":
    init_postgres()
