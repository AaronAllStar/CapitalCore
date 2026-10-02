"""
Seed vía HTTP — llama a los endpoints de la API para crear roles y usuarios.
Esto evita problemas de asyncpg en Windows con scripts standalone.

Asegúrate de que la API esté corriendo en http://localhost:8000 antes de ejecutar.
"""
import sys
import os
import urllib.request
import urllib.error
import json

API_BASE = "http://localhost:8000/api/v1"


def post(path: str, data: dict) -> dict | None:
    url = f"{API_BASE}{path}"
    payload = json.dumps(data).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            return json.loads(resp.read())
    except urllib.error.HTTPError as e:
        body = json.loads(e.read()) if e.read else {}
        code = body.get("error", {}).get("code", "")
        if code in ("CONFLICT", ""):
            return None  # Already exists, that's OK
        print(f"  HTTP {e.code} at {path}: {body}")
        return None
    except Exception as ex:
        print(f"  Error at {path}: {ex}")
        return None


def seed():
    print("Seeding roles...")
    post("/admin/seed-roles", {})
    print("  Roles seeded OK")

    print("\nSeeding test users via API...")

    users = [
        ("admin@edgearena.com", "admin", "Admin", "Admin123!"),
        ("trader@edgearena.com", "protrader", "Pro Trader", "Test1234!"),
        ("demo@edgearena.com", "demouser", "Demo User", "Test1234!"),
    ]

    for email, username, display_name, password in users:
        print(f"  Registering {email}...", end=" ")
        result = post("/auth/register", {
            "email": email,
            "username": username,
            "password": password,
            "display_name": display_name,
        })
        if result:
            print(f"OK (id={result['user']['id'][:8]}...)")
        else:
            print("Already exists or skipped")

    print("\nSeed complete!")
    print("\nCredentials:")
    for email, username, _, password in users:
        print(f"  {email} / {password}  ({username})")


if __name__ == "__main__":
    seed()
