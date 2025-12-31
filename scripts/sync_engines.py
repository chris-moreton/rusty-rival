#!/usr/bin/env python3
"""Sync engines.json with v* directories in engines folder.

Automatically adds any new v* engine versions that exist as directories
but are missing from engines.json.
"""

import json
import sys
from pathlib import Path

ENGINES_DIR = Path(__file__).parent.parent / "engines"
ENGINES_JSON = ENGINES_DIR / "engines.json"


def get_existing_versions():
    """Get v* directories that have a rusty-rival binary."""
    versions = []
    for d in ENGINES_DIR.iterdir():
        if d.is_dir() and d.name.startswith("v"):
            binary = d / "rusty-rival"
            if binary.exists():
                versions.append(d.name)
    return sorted(versions)


def load_engines_json():
    """Load current engines.json."""
    with open(ENGINES_JSON) as f:
        return json.load(f)


def save_engines_json(data):
    """Save engines.json with proper formatting."""
    with open(ENGINES_JSON, "w") as f:
        json.dump(data, f, indent=2)
        f.write("\n")


def sync_engines():
    """Add missing v* engines to engines.json."""
    existing_dirs = get_existing_versions()
    engines = load_engines_json()

    added = []
    for version in existing_dirs:
        if version not in engines:
            engines[version] = {
                "binary": f"{version}/rusty-rival"
            }
            added.append(version)

    if added:
        save_engines_json(engines)
        print(f"Added {len(added)} engine(s):")
        for v in added:
            print(f"  + {v}")
    else:
        print("All engines already in engines.json")

    # Show any engines in json that don't have directories
    missing = []
    for name in engines:
        if name.startswith("v") and name not in existing_dirs:
            missing.append(name)

    if missing:
        print(f"\nWarning: {len(missing)} engine(s) in engines.json but no directory:")
        for v in missing:
            print(f"  ? {v}")

    return len(added)


def main():
    if not ENGINES_JSON.exists():
        print(f"Error: {ENGINES_JSON} not found")
        sys.exit(1)

    sync_engines()


if __name__ == "__main__":
    main()
