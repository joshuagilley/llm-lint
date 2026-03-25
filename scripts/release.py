#!/usr/bin/env python3
"""Bump version in Cargo.toml + npm/package.json, commit, push main, tag, and publish a GitHub release.

Usage:
  ./scripts/release.py 0.2.0
  ./scripts/release.py v0.2.0
  ./scripts/release.py 0.2.0 --dry-run
  ./scripts/release.py 0.2.0 --no-pull
  ./scripts/release.py 0.2.0 --notes-file CHANGELOG-snippet.md

Requires: git, gh (authenticated), clean working tree, branch main.

Optional: set LLM_LINT_RELEASE_EXPECT_REPO=owner/repo (or --expect-repo) so the script
refuses to run unless `gh repo view` matches (avoids releasing from a fork).

After the release is published on GitHub, the workflow "Publish to PyPI and npm" uploads wheels
and the npm package (PyPI must succeed first for `uv tool run --from llm-lint` to work).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

_ROOT = Path(__file__).resolve().parents[1]
_CARGO = _ROOT / "Cargo.toml"
_NPM_PKG = _ROOT / "npm" / "package.json"
_VERSION_RE = re.compile(r"^\d+\.\d+\.\d+$")


def _run(cmd: list[str], *, dry_run: bool) -> None:
    print("+", " ".join(cmd))
    if dry_run:
        return
    subprocess.run(cmd, cwd=_ROOT, check=True)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _write_text(path: Path, text: str, *, dry_run: bool) -> None:
    if dry_run:
        print(f"[dry-run] would write {path.relative_to(_ROOT)}")
        return
    path.write_text(text, encoding="utf-8")


def _bump_cargo(semver: str, *, dry_run: bool) -> None:
    text = _read_text(_CARGO)
    new_text, n = re.subn(
        r'^version = "[^"]+"\s*$',
        f'version = "{semver}"',
        text,
        count=1,
        flags=re.MULTILINE,
    )
    if n != 1:
        sys.exit("error: could not find a single version = line in Cargo.toml")
    _write_text(_CARGO, new_text, dry_run=dry_run)


def _bump_npm(semver: str, *, dry_run: bool) -> None:
    if dry_run:
        print(f"[dry-run] would set npm/package.json version to {semver}")
        return
    data = json.loads(_read_text(_NPM_PKG))
    data["version"] = semver
    _NPM_PKG.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def _git_porcelain() -> str:
    r = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=_ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return r.stdout.strip()


def _current_branch() -> str:
    r = subprocess.run(
        ["git", "rev-parse", "--abbrev-ref", "HEAD"],
        cwd=_ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return r.stdout.strip()


def _gh_name_with_owner() -> str:
    r = subprocess.run(
        [
            "gh",
            "repo",
            "view",
            "--json",
            "nameWithOwner",
            "--jq",
            ".nameWithOwner",
        ],
        cwd=_ROOT,
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        err = (r.stderr or r.stdout or "").strip()
        sys.exit(f"error: gh repo view failed: {err}")
    return r.stdout.strip()


def _assert_expected_repo(expected: str | None, *, dry_run: bool) -> None:
    if not expected:
        return
    exp = expected.strip()
    if dry_run:
        print(f"[dry-run] would require gh repo nameWithOwner == {exp!r}")
        return
    actual = _gh_name_with_owner()
    if actual.casefold() != exp.casefold():
        sys.exit(
            f"error: gh reports repo {actual!r}, expected {exp!r} "
            "(set LLM_LINT_RELEASE_EXPECT_REPO or use --expect-repo on the canonical clone)"
        )


def _commit_release(semver: str, *, dry_run: bool) -> None:
    cmd = ["git", "commit", "-m", f"chore: release {semver}"]
    print("+", " ".join(cmd))
    if dry_run:
        return

    first = subprocess.run(cmd, cwd=_ROOT)
    if first.returncode == 0:
        return

    if _git_porcelain():
        print("+ git add -u")
        subprocess.run(["git", "add", "-u"], cwd=_ROOT, check=True)
        print("+", " ".join(cmd))
        subprocess.run(cmd, cwd=_ROOT, check=True)
        return

    raise subprocess.CalledProcessError(first.returncode, cmd)


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description="llm-lint release: version bump through gh release.",
    )
    p.add_argument(
        "version",
        help='Semver, e.g. 0.2.0 or v0.2.0 (Git tag will be "v" + semver).',
    )
    p.add_argument(
        "--dry-run",
        action="store_true",
        help="Print commands only; change nothing.",
    )
    p.add_argument("--no-pull", action="store_true", help="Skip git pull origin main.")
    p.add_argument(
        "--allow-dirty",
        action="store_true",
        help="Allow starting with a dirty working tree (unsafe).",
    )
    p.add_argument(
        "--notes-file",
        metavar="PATH",
        help="Pass to gh release create as --notes-file (default: --generate-notes).",
    )
    p.add_argument(
        "--expect-repo",
        metavar="OWNER/REPO",
        help=(
            "Abort unless `gh repo view` is this repo. "
            "Default: env LLM_LINT_RELEASE_EXPECT_REPO."
        ),
    )
    return p


def _parse_semver(version_arg: str) -> tuple[str, str]:
    raw = version_arg.strip()
    semver = raw.removeprefix("v").removeprefix("V")
    if not _VERSION_RE.match(semver):
        sys.exit(f"error: expected semver like 0.2.0, got {version_arg!r}")
    return semver, f"v{semver}"


def _expect_repo_from_env_and_args(args: argparse.Namespace) -> str | None:
    return (
        args.expect_repo or os.environ.get("LLM_LINT_RELEASE_EXPECT_REPO") or ""
    ).strip() or None


def _preflight(args: argparse.Namespace, expect_repo: str | None, *, dry_run: bool) -> None:
    if dry_run:
        _assert_expected_repo(expect_repo, dry_run=True)
        return
    if _current_branch() != "main":
        sys.exit("error: must be on branch main")
    dirty = _git_porcelain()
    if dirty and not args.allow_dirty:
        sys.exit("error: working tree is not clean (commit/stash first, or --allow-dirty)")
    _assert_expected_repo(expect_repo, dry_run=False)


def _gh_release_command(tag: str, semver: str, notes_file: str | None) -> list[str]:
    cmd = ["gh", "release", "create", tag, "--title", semver]
    if notes_file:
        cmd.extend(["--notes-file", notes_file])
    else:
        cmd.append("--generate-notes")
    return cmd


def _run_publish_pipeline(
    semver: str,
    tag: str,
    *,
    no_pull: bool,
    notes_file: str | None,
    dry_run: bool,
) -> None:
    if not no_pull:
        _run(["git", "pull", "origin", "main"], dry_run=dry_run)
    _bump_cargo(semver, dry_run=dry_run)
    _bump_npm(semver, dry_run=dry_run)
    staged = ["Cargo.toml", "npm/package.json"]
    _run(["git", "add", *staged], dry_run=dry_run)
    _commit_release(semver, dry_run=dry_run)
    _run(["git", "push", "origin", "main"], dry_run=dry_run)
    _run(["git", "tag", tag], dry_run=dry_run)
    _run(["git", "push", "origin", tag], dry_run=dry_run)
    _run(_gh_release_command(tag, semver, notes_file), dry_run=dry_run)


def _print_finish(tag: str, *, dry_run: bool) -> None:
    if dry_run:
        print("\n(dry-run: no files or git remotes were changed)")
    else:
        print(
            f"\nDone. GitHub release {tag} triggers Publish to PyPI and npm.\n"
            "Ensure PyPI publish completes before npm users rely on llm-lint-cli."
        )


def main() -> None:
    args = _build_parser().parse_args()
    semver, tag = _parse_semver(args.version)
    expect_repo = _expect_repo_from_env_and_args(args)
    _preflight(args, expect_repo, dry_run=args.dry_run)
    _run_publish_pipeline(
        semver,
        tag,
        no_pull=args.no_pull,
        notes_file=args.notes_file,
        dry_run=args.dry_run,
    )
    _print_finish(tag, dry_run=args.dry_run)


if __name__ == "__main__":
    main()
