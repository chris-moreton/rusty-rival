#!/usr/bin/env python3
"""
Release script for Rusty Rival.

Updates version in Cargo.toml, commits, and creates a git tag.

Usage:
    python scripts/release.py 1.0.13
    python scripts/release.py 1.0.13 --dry-run
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
PROJECT_DIR = SCRIPT_DIR.parent
CARGO_TOML = PROJECT_DIR / "Cargo.toml"


def run(cmd: str, check: bool = True, capture: bool = True) -> subprocess.CompletedProcess:
    """Run a shell command."""
    print(f"  $ {cmd}")
    result = subprocess.run(cmd, shell=True, capture_output=capture, text=True, cwd=PROJECT_DIR)
    if check and result.returncode != 0:
        print(f"  ERROR: {result.stderr}")
        sys.exit(1)
    return result


def validate_version(version: str) -> bool:
    """Validate version format (e.g., 1.0.13)."""
    pattern = r'^\d+\.\d+\.\d+$'
    return bool(re.match(pattern, version))


def get_current_version() -> str:
    """Get current version from Cargo.toml."""
    content = CARGO_TOML.read_text()
    match = re.search(r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
    if match:
        return match.group(1)
    return "unknown"


def update_cargo_toml(version: str) -> None:
    """Update version in Cargo.toml."""
    content = CARGO_TOML.read_text()
    new_content = re.sub(
        r'^(version\s*=\s*)"[^"]+"',
        f'\\1"{version}"',
        content,
        count=1,
        flags=re.MULTILINE
    )
    CARGO_TOML.write_text(new_content)


def check_working_tree_clean() -> bool:
    """Check if git working tree is clean."""
    result = run("git status --porcelain", check=False)
    return len(result.stdout.strip()) == 0


def tag_exists(tag: str) -> bool:
    """Check if a git tag already exists."""
    result = run(f"git tag -l {tag}", check=False)
    return tag in result.stdout


def main():
    parser = argparse.ArgumentParser(description="Release a new version of Rusty Rival")
    parser.add_argument("version", help="Version number (e.g., 1.0.13)")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be done without making changes")
    args = parser.parse_args()

    version = args.version
    tag = f"v{version}"

    # Validate version format
    if not validate_version(version):
        print(f"ERROR: Invalid version format '{version}'. Expected format: X.Y.Z (e.g., 1.0.13)")
        sys.exit(1)

    current_version = get_current_version()
    print(f"Current version: {current_version}")
    print(f"New version:     {version}")
    print(f"Tag:             {tag}")
    print()

    # Check if tag already exists
    if tag_exists(tag):
        print(f"ERROR: Tag '{tag}' already exists")
        sys.exit(1)

    # Check for uncommitted changes (excluding Cargo.toml which we'll modify)
    result = run("git status --porcelain", check=False)
    uncommitted = [line for line in result.stdout.strip().split('\n') if line and 'Cargo.toml' not in line]
    if uncommitted:
        print("ERROR: You have uncommitted changes:")
        for line in uncommitted:
            print(f"  {line}")
        print("\nPlease commit or stash them before releasing.")
        sys.exit(1)

    if args.dry_run:
        print("DRY RUN - would perform the following actions:")
        print(f"  1. Update Cargo.toml version to {version}")
        print(f"  2. Commit with message 'Release v{version}'")
        print(f"  3. Create tag '{tag}'")
        print("\nRun without --dry-run to execute.")
        return

    # Update Cargo.toml
    print("Updating Cargo.toml...")
    update_cargo_toml(version)

    # Commit
    print("Committing...")
    run("git add Cargo.toml")
    run(f'git commit -m "Release v{version}"')

    # Tag
    print("Creating tag...")
    run(f'git tag {tag}')

    print()
    print("=" * 50)
    print(f"Released v{version}")
    print("=" * 50)
    print()
    print("Next steps:")
    print("  git push && git push --tags")
    print()


if __name__ == "__main__":
    main()
