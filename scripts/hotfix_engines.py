#!/usr/bin/env python3
"""
Apply hotfix to all engine versions for the multi_pv index out of bounds bug.
"""

import subprocess
import re
from pathlib import Path

ENGINE_DIR = Path(__file__).parent.parent / "engines"
SCRIPT_DIR = Path(__file__).parent.parent

# The fix line to add after "let multi_pv = ..."
FIX_LINE = "    // Limit multi_pv to actual number of root moves to avoid index out of bounds\n    let multi_pv = multi_pv.min(search_state.root_moves.len() as u8);\n"

def run(cmd, check=True, capture=True):
    """Run a shell command."""
    print(f"  $ {cmd}")
    result = subprocess.run(cmd, shell=True, capture_output=capture, text=True, cwd=SCRIPT_DIR)
    if check and result.returncode != 0:
        print(f"  ERROR: {result.stderr}")
        raise Exception(f"Command failed: {cmd}")
    return result

def get_engine_versions():
    """Get list of engine version directories."""
    versions = []
    for d in sorted(ENGINE_DIR.iterdir()):
        if d.is_dir() and (d / "rusty-rival").exists():
            versions.append(d.name)
    return versions

def tag_exists(tag):
    """Check if a git tag exists."""
    result = run(f"git tag -l '{tag}'", check=False)
    return tag in result.stdout

def apply_fix(version):
    """Apply the hotfix to src/utils.rs for the current branch."""
    utils_path = SCRIPT_DIR / "src" / "utils.rs"
    content = utils_path.read_text()

    # Check if fix is already applied
    if "multi_pv.min(search_state.root_moves.len()" in content:
        print(f"  Fix already applied in {version}")
        return True

    # Find the line to fix
    # Pattern: "let multi_pv = if show_multi_pv { search_state.multi_pv } else { 1 };"
    pattern = r'(    let multi_pv = if show_multi_pv \{ search_state\.multi_pv \} else \{ 1 \};)\n(    if search_state\.start_time)'

    if not re.search(pattern, content):
        print(f"  WARNING: Could not find pattern to fix in {version}")
        return False

    # Insert the fix line
    replacement = r'\1\n' + FIX_LINE + r'\2'
    new_content = re.sub(pattern, replacement, content)

    if new_content == content:
        print(f"  WARNING: Fix did not change content in {version}")
        return False

    utils_path.write_text(new_content)
    print(f"  Applied fix to utils.rs")
    return True

def main():
    versions = get_engine_versions()
    print(f"Found {len(versions)} engine versions: {versions}\n")

    # Store current branch
    result = run("git branch --show-current")
    original_branch = result.stdout.strip()
    print(f"Current branch: {original_branch}\n")

    failed = []
    succeeded = []

    for version in versions:
        print(f"\n{'='*60}")
        print(f"Processing {version}")
        print('='*60)

        tag = version
        if not tag_exists(tag):
            print(f"  Tag '{tag}' not found, skipping")
            failed.append((version, "tag not found"))
            continue

        hotfix_branch = f"{version}-hotfix"

        # Check if hotfix branch already exists
        result = run(f"git branch -l '{hotfix_branch}'", check=False)
        branch_exists = hotfix_branch in result.stdout

        try:
            if branch_exists:
                print(f"  Hotfix branch already exists, checking out")
                run(f"git checkout {hotfix_branch}")
            else:
                print(f"  Creating hotfix branch from tag {tag}")
                run(f"git checkout -b {hotfix_branch} {tag}")

            # Apply fix
            if not apply_fix(version):
                failed.append((version, "fix failed"))
                run(f"git checkout {original_branch}")
                continue

            # Check if there are changes to commit
            result = run("git status --porcelain", check=False)
            if result.stdout.strip():
                # Commit the fix
                run("git add src/utils.rs")
                run('git commit -m "Fix multi_pv index out of bounds bug"')
                print("  Committed fix")
            else:
                print("  No changes to commit (fix already applied)")

            # Build
            print("  Building...")
            result = run("cargo build --release", check=False)
            if result.returncode != 0:
                print(f"  Build FAILED")
                failed.append((version, "build failed"))
                run(f"git checkout {original_branch}")
                continue

            # Archive old and copy new
            engine_path = ENGINE_DIR / version / "rusty-rival"
            engine_original = ENGINE_DIR / version / "rusty-rival_original"
            new_binary = SCRIPT_DIR / "target" / "release" / "rusty-rival"

            if engine_path.exists() and not engine_original.exists():
                print(f"  Renaming old executable to rusty-rival_original")
                engine_path.rename(engine_original)

            print(f"  Copying new executable")
            run(f"cp {new_binary} {engine_path}")

            succeeded.append(version)
            print(f"  SUCCESS: {version} updated")

        except Exception as e:
            print(f"  ERROR: {e}")
            failed.append((version, str(e)))

        # Return to original branch
        run(f"git checkout {original_branch}", check=False)

    # Summary
    print(f"\n{'='*60}")
    print("SUMMARY")
    print('='*60)
    print(f"Succeeded: {len(succeeded)}")
    for v in succeeded:
        print(f"  - {v}")
    print(f"\nFailed: {len(failed)}")
    for v, reason in failed:
        print(f"  - {v}: {reason}")

if __name__ == "__main__":
    main()
