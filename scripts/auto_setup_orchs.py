#!/usr/bin/env python3
"""PHOENIX ORCH repo auto-setup.

Clones GitHub repositories into ../orch_repos (parent of Phoenix project root),
auto-detects repo type, runs safe-ish build/install steps in isolation, and
emits a markdown report in the Phoenix project root.

Usage:
  python scripts/auto_setup_orchs.py <url1> <url2> ...
  python scripts/auto_setup_orchs.py --config repos.txt

Notes:
  - Python repos are installed into a per-repo venv under ../orch_repos/.venvs/<repo>/
  - Rust repos attempt `cargo component build --release` first (WASM component model)
    and fall back to `cargo build --release`.
"""

from __future__ import annotations

import argparse
import dataclasses
import datetime as _dt
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Iterable, List, Optional, Sequence, Tuple

import json
import urllib.error
import urllib.parse
import urllib.request


@dataclasses.dataclass
class RepoResult:
    name: str
    url: str
    dest_dir: Path
    detected: str
    build_commands: List[str]
    status: str  # "Success" | "Failure" | "Skipped"
    error: Optional[str]
    entrypoint: Optional[str]
    integration_notes: List[str]


def _parse_github_owner_repo(url: str) -> Optional[Tuple[str, str]]:
    """Extract (owner, repo) from a GitHub HTTPS URL."""
    try:
        u = urllib.parse.urlparse(url.strip())
    except Exception:
        return None
    if u.scheme not in {"http", "https"}:
        return None
    if u.netloc.lower() != "github.com":
        return None
    parts = [p for p in u.path.strip("/").split("/") if p]
    if len(parts) < 2:
        return None
    owner, repo = parts[0], parts[1]
    if repo.endswith(".git"):
        repo = repo[: -len(".git")]
    return owner, repo


def _github_request_json(url: str, *, token: Optional[str], user_agent: str, timeout_sec: int = 15) -> Tuple[int, dict]:
    req = urllib.request.Request(url)
    req.add_header("Accept", "application/vnd.github+json")
    req.add_header("User-Agent", user_agent)
    if token:
        req.add_header("Authorization", f"Bearer {token}")
    try:
        with urllib.request.urlopen(req, timeout=timeout_sec) as resp:
            status = int(getattr(resp, "status", 200))
            data = resp.read().decode("utf-8", errors="replace")
            return status, json.loads(data) if data else {}
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8", errors="replace") if hasattr(e, "read") else ""
        try:
            return int(e.code), json.loads(body) if body else {}
        except Exception:
            return int(e.code), {"error": body or str(e)}
    except Exception as e:
        return 0, {"error": str(e)}


def _github_ci_gate(
    *,
    owner: str,
    repo: str,
    workflow: str,
    branch: str,
    token: Optional[str],
    user_agent: str,
) -> Tuple[bool, str, bool]:
    """Return (passed, note, workflow_exists)."""

    base = f"https://api.github.com/repos/{owner}/{repo}"

    # 1) Does the workflow exist?
    status, meta = _github_request_json(
        f"{base}/actions/workflows/{urllib.parse.quote(workflow)}",
        token=token,
        user_agent=user_agent,
    )
    if status == 404:
        return False, f"CI gate: workflow '{workflow}' not found", False
    if status == 0:
        return False, f"CI gate: GitHub API request failed ({meta.get('error', 'unknown error')})", True
    if status and status >= 400:
        return False, f"CI gate: GitHub API error ({status})", True

    # 2) What is the latest run?
    runs_url = f"{base}/actions/workflows/{urllib.parse.quote(workflow)}/runs?branch={urllib.parse.quote(branch)}&per_page=1"
    status2, runs = _github_request_json(runs_url, token=token, user_agent=user_agent)
    if status2 == 0:
        return False, f"CI gate: GitHub API request failed ({runs.get('error', 'unknown error')})", True
    if status2 and status2 >= 400:
        return False, f"CI gate: cannot read workflow runs ({status2})", True

    wf_runs = runs.get("workflow_runs") or []
    if not wf_runs:
        return False, f"CI gate: no runs found for '{workflow}' on branch '{branch}'", True

    latest = wf_runs[0]
    conclusion = (latest.get("conclusion") or "").lower()
    run_status = (latest.get("status") or "").lower()
    html_url = latest.get("html_url") or ""

    if run_status != "completed":
        return False, f"CI gate: latest run not completed (status={run_status}) {html_url}".strip(), True
    if conclusion != "success":
        return False, f"CI gate: latest run did not succeed (conclusion={conclusion}) {html_url}".strip(), True

    return True, f"CI gate: latest run success {html_url}".strip(), True


def _which(cmd: str) -> Optional[str]:
    return shutil.which(cmd)


def _run(
    argv: Sequence[str],
    *,
    cwd: Path,
    env: Optional[dict] = None,
    timeout_sec: Optional[int] = None,
) -> Tuple[int, str]:
    """Run a command and return (exit_code, combined_output)."""
    proc = subprocess.run(
        list(argv),
        cwd=str(cwd),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=timeout_sec,
    )
    return proc.returncode, proc.stdout


def _pip_safe_env() -> dict:
    """Return an env mapping that avoids common TLS/CA misconfig breaking pip.

    Some Windows installs (notably Postgres) can set SSL_CERT_FILE/REQUESTS_CA_BUNDLE
    to a non-existent path. pip then fails before it can download anything.
    """
    e = dict(os.environ)
    for k in [
        "SSL_CERT_FILE",
        "REQUESTS_CA_BUNDLE",
        "CURL_CA_BUNDLE",
        "PIP_CERT",
    ]:
        if k in e:
            e.pop(k, None)
    e.setdefault("PIP_DISABLE_PIP_VERSION_CHECK", "1")
    e.setdefault("PYTHONUTF8", "1")
    return e


def _find_project_root() -> Path:
    """Find Phoenix project root.

    Prefers script location, falls back to CWD scanning.
    """

    def scan(start: Path) -> Optional[Path]:
        for p in [start, *start.parents]:
            if (p / "Cargo.toml").exists() and (p / "scripts").is_dir():
                # Extra guard: look for a file that strongly implies Phoenix root.
                if (p / "scripts" / "launch_phoenix.sh").exists() or (p / "README.md").exists():
                    return p
        return None

    here = Path(__file__).resolve()
    root = scan(here.parent)
    if root is None:
        root = scan(Path.cwd().resolve())
    if root is None:
        raise RuntimeError("Could not detect Phoenix project root (expected Cargo.toml + scripts/)")
    return root


def _orch_repos_dir(project_root: Path) -> Path:
    return project_root.parent / "orch_repos"


def _parse_repo_name(url: str) -> str:
    # Handles https://github.com/org/name(.git)
    cleaned = url.strip().rstrip("/")
    name = cleaned.split("/")[-1]
    if name.endswith(".git"):
        name = name[: -len(".git")]
    name = re.sub(r"[^A-Za-z0-9._-]+", "_", name)
    return name or "repo"


def _load_urls_from_config(path: Path) -> List[str]:
    urls: List[str] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        urls.append(s)
    return urls


def _git_clone_or_update(url: str, dest_dir: Path, *, shallow: bool) -> Tuple[bool, List[str], Optional[str]]:
    cmds: List[str] = []
    if _which("git") is None:
        return False, cmds, "git not found on PATH"

    if dest_dir.exists() and (dest_dir / ".git").exists():
        cmds.append("git pull --ff-only")
        code, out = _run(["git", "pull", "--ff-only"], cwd=dest_dir)
        if code != 0:
            return False, cmds, out[-4000:]
        return True, cmds, None

    dest_dir.parent.mkdir(parents=True, exist_ok=True)
    argv = ["git", "clone"]
    if shallow:
        argv += ["--depth", "1"]
    argv += [url, str(dest_dir)]
    cmds.append(" ".join(argv))
    code, out = _run(argv, cwd=dest_dir.parent)
    if code != 0:
        return False, cmds, out[-4000:]
    return True, cmds, None


def _detect_repo_type(repo_dir: Path) -> str:
    if (repo_dir / "Cargo.toml").exists():
        return "Rust"
    if (repo_dir / "pyproject.toml").exists() or (repo_dir / "requirements.txt").exists() or (repo_dir / "setup.py").exists():
        return "Python"
    if (repo_dir / "package.json").exists():
        return "Node.js"
    if (repo_dir / "Dockerfile").exists():
        return "Docker"
    # Fallback: examples repos often omit requirements/pyproject at the root.
    if _looks_like_python_repo(repo_dir):
        return "Python"
    return "Unknown"


def _looks_like_python_repo(repo_dir: Path) -> bool:
    skip = {".git", "target", "node_modules", ".venv", ".venvs", "__pycache__"}

    def depth(p: Path) -> int:
        try:
            return len(p.relative_to(repo_dir).parts)
        except Exception:
            return 999

    for root, dirs, files in os.walk(repo_dir):
        rp = Path(root)
        if any(part in skip for part in rp.parts):
            dirs[:] = []
            continue
        if depth(rp) > 3:
            dirs[:] = []
            continue
        for f in files:
            if f.endswith(".py"):
                return True
            if f in {"requirements.txt", "pyproject.toml", "setup.py"}:
                return True
    return False


def _venv_python(venv_dir: Path) -> Path:
    if os.name == "nt":
        return venv_dir / "Scripts" / "python.exe"
    return venv_dir / "bin" / "python"


def _python_version(py_exe: Path, *, cwd: Path) -> Tuple[int, int]:
    code, out = _run([str(py_exe), "-c", "import sys; print(f'{sys.version_info[0]}.{sys.version_info[1]}')"], cwd=cwd)
    if code != 0:
        return (sys.version_info[0], sys.version_info[1])
    m = re.search(r"(\d+)\.(\d+)", out.strip())
    if not m:
        return (sys.version_info[0], sys.version_info[1])
    return (int(m.group(1)), int(m.group(2)))


def _preflight_requirements_compat(repo_dir: Path, py_ver: Tuple[int, int]) -> Optional[str]:
    """Detect common hard-incompatibilities early and return an actionable message."""
    req = repo_dir / "requirements.txt"
    if not req.exists():
        return None
    txt = req.read_text(encoding="utf-8", errors="ignore")
    maj, minor = py_ver

    # numpy==1.23.* has no wheels for Python >=3.12 and tends to fail building from source.
    if (maj, minor) >= (3, 12) and re.search(r"(?im)^\s*numpy\s*==\s*1\.23\.[0-9]+\s*$", txt):
        return (
            "requirements.txt pins numpy==1.23.* which does not support this Python version "
            f"({maj}.{minor}). Install Python 3.10/3.11 (recommended) or update the pin to a Python-{maj}.{minor}-compatible numpy."
        )

    return None


def _ensure_venv(venv_dir: Path) -> Tuple[bool, Optional[str]]:
    py = sys.executable
    venv_dir.parent.mkdir(parents=True, exist_ok=True)
    if _venv_python(venv_dir).exists():
        return True, None
    code, out = _run([py, "-m", "venv", str(venv_dir)], cwd=venv_dir.parent)
    if code != 0:
        return False, out[-4000:]
    return True, None


def _python_build(repo_dir: Path, venvs_dir: Path) -> Tuple[bool, List[str], Optional[str], Optional[str], List[str]]:
    cmds: List[str] = []
    notes: List[str] = [
        "Spawn via tokio::process::Command and communicate over stdin/stdout (or HTTP if the repo exposes it).",
        "Prefer invoking the repo using the per-ORCH venv Python to avoid dependency collisions.",
    ]
    if _which("python") is None and _which("python3") is None and sys.executable is None:
        return False, cmds, "Python not found on PATH", None, notes

    venv_dir = venvs_dir / repo_dir.name
    ok, err = _ensure_venv(venv_dir)
    if not ok:
        return False, cmds, f"venv creation failed: {err}", None, notes

    vpy = _venv_python(venv_dir)

    py_ver = _python_version(vpy, cwd=repo_dir)
    # Always upgrade pip tooling to reduce install friction.
    cmds.append(f"{vpy} -m pip install -U pip setuptools wheel")
    code, out = _run(
        [str(vpy), "-m", "pip", "install", "-U", "pip", "setuptools", "wheel"],
        cwd=repo_dir,
        env=_pip_safe_env(),
    )
    if code != 0:
        return False, cmds, out[-4000:], None, notes

    # Install dependencies
    if (repo_dir / "requirements.txt").exists():
        compat_err = _preflight_requirements_compat(repo_dir, py_ver)
        if compat_err:
            notes.append(f"Compatibility: {compat_err}")
            return False, cmds, compat_err, _guess_python_entrypoint(repo_dir), notes
        cmds.append(f"{vpy} -m pip install -r requirements.txt")
        code, out = _run(
            [str(vpy), "-m", "pip", "install", "-r", "requirements.txt"],
            cwd=repo_dir,
            env=_pip_safe_env(),
        )
        if code != 0:
            return False, cmds, out[-4000:], None, notes
    elif (repo_dir / "poetry.lock").exists() and _which("poetry") is not None:
        cmds.append("poetry install")
        code, out = _run(["poetry", "install"], cwd=repo_dir)
        if code != 0:
            return False, cmds, out[-4000:], None, notes
        notes.append("Poetry-managed environment detected; ensure Phoenix spawns with the correct interpreter (poetry env info -p).")
    elif (repo_dir / "Pipfile").exists() and _which("pipenv") is not None:
        cmds.append("pipenv install")
        code, out = _run(["pipenv", "install"], cwd=repo_dir)
        if code != 0:
            return False, cmds, out[-4000:], None, notes
        notes.append("Pipenv-managed environment detected; ensure Phoenix spawns with `pipenv run python ...`.")
    elif (repo_dir / "pyproject.toml").exists() or (repo_dir / "setup.py").exists():
        # Best-effort editable install. Some repos may require optional extras.
        cmds.append(f"{vpy} -m pip install -e .")
        code, out = _run([str(vpy), "-m", "pip", "install", "-e", "."], cwd=repo_dir, env=_pip_safe_env())
        if code != 0:
            # Fallback to non-editable install
            cmds.append(f"{vpy} -m pip install .")
            code2, out2 = _run([str(vpy), "-m", "pip", "install", "."], cwd=repo_dir, env=_pip_safe_env())
            if code2 != 0:
                return False, cmds, (out2 or out)[-4000:], None, notes
    else:
        return True, cmds, None, _guess_python_entrypoint(repo_dir), notes

    return True, cmds, None, _guess_python_entrypoint(repo_dir), notes


def _guess_python_entrypoint(repo_dir: Path) -> Optional[str]:
    # Conservative heuristics: prefer root scripts.
    for candidate in ["main.py", "app.py", "run.py", "cli.py", "server.py", "bot.py"]:
        p = repo_dir / candidate
        if p.exists():
            return candidate
    # Fall back to README for examples-driven repos.
    if (repo_dir / "README.md").exists():
        return "README.md"
    return None


def _node_build(repo_dir: Path) -> Tuple[bool, List[str], Optional[str], Optional[str], List[str]]:
    cmds: List[str] = []
    notes: List[str] = [
        "Spawn via tokio::process::Command (node) or run as a long-lived service and bridge via HTTP/WebSocket.",
        "Prefer `npm start` / `npm run <script>` based on package.json scripts.",
    ]
    if _which("npm") is None:
        return False, cmds, "npm not found on PATH", None, notes
    cmds.append("npm install")
    code, out = _run(["npm", "install"], cwd=repo_dir)
    if code != 0:
        return False, cmds, out[-4000:], None, notes
    entry = "package.json"
    return True, cmds, None, entry, notes


def _rust_build(repo_dir: Path) -> Tuple[bool, List[str], Optional[str], Optional[str], List[str]]:
    cmds: List[str] = []
    notes: List[str] = [
        "If a .wasm is produced, register it as a WASM ORCH and run via Wasmtime/Wasmer (recommended sandbox).",
        "If only a native binary is produced, spawn it as a subprocess ORCH and bridge over stdio/IPC.",
    ]
    if _which("cargo") is None:
        return False, cmds, "cargo not found on PATH", None, notes

    # Try component build first (WASM component model) then fall back.
    if _which("cargo-component") is not None or True:
        # `cargo component` is a cargo subcommand; presence check is unreliable.
        cmds.append("cargo component build --release")
        code, out = _run(["cargo", "component", "build", "--release"], cwd=repo_dir)
        if code == 0:
            entry = _guess_rust_entrypoint(repo_dir)
            return True, cmds, None, entry, notes
        # fall back
        cmds.append("cargo build --release")
        code2, out2 = _run(["cargo", "build", "--release"], cwd=repo_dir)
        if code2 != 0:
            return False, cmds, (out2 or out)[-4000:], None, notes
        entry = _guess_rust_entrypoint(repo_dir)
        return True, cmds, None, entry, notes


def _guess_rust_entrypoint(repo_dir: Path) -> Optional[str]:
    # Prefer a produced wasm if present.
    target = repo_dir / "target"
    if target.exists():
        wasm_candidates = list(target.glob("wasm32-*/release/*.wasm")) + list(target.glob("**/release/*.wasm"))
        wasm_candidates = [p for p in wasm_candidates if p.is_file()]
        if wasm_candidates:
            # Pick the newest.
            wasm_candidates.sort(key=lambda p: p.stat().st_mtime, reverse=True)
            try:
                return str(wasm_candidates[0].relative_to(repo_dir))
            except Exception:
                return str(wasm_candidates[0])

        # Native binaries are harder to infer; provide a directory hint.
        rel = target / "release"
        if rel.exists():
            return "target/release/<binary>"
    return None


def _docker_build(repo_dir: Path) -> Tuple[bool, List[str], Optional[str], Optional[str], List[str]]:
    cmds: List[str] = []
    notes: List[str] = [
        "Dockerfile detected. Consider containerizing as an external ORCH and bridging via HTTP.",
    ]
    return True, cmds, None, "Dockerfile", notes


def _build_repo(repo_dir: Path, detected: str, *, venvs_dir: Path) -> Tuple[bool, List[str], Optional[str], Optional[str], List[str]]:
    if detected == "Rust":
        return _rust_build(repo_dir)
    if detected == "Python":
        return _python_build(repo_dir, venvs_dir)
    if detected == "Node.js":
        return _node_build(repo_dir)
    if detected == "Docker":
        return _docker_build(repo_dir)
    return True, [], None, None, ["Unknown repo type; no build executed."]


def _format_docs(project_root: Path, orch_dir: Path, results: Sequence[RepoResult], started_utc: _dt.datetime) -> str:
    dt_local = started_utc.astimezone()
    lines: List[str] = []
    lines.append("# PHOENIX ORCH Repository Setup Log")
    lines.append("")
    lines.append(f"**Run Timestamp (local):** {dt_local.strftime('%Y-%m-%d %H:%M:%S %Z')}")
    lines.append(f"**Run Timestamp (UTC):** {started_utc.strftime('%Y-%m-%d %H:%M:%S UTC')}")
    lines.append(f"**Phoenix Root:** `{project_root}`")
    lines.append(f"**ORCH Repos Dir:** `{orch_dir}`")
    lines.append("")
    lines.append("## Cloned & Built Repositories")
    lines.append("")

    for i, r in enumerate(results, start=1):
        lines.append(f"### {i}. {r.name}")
        lines.append(f"- URL: {r.url}")
        lines.append(f"- Path: `{r.dest_dir}`")
        lines.append(f"- Language detected: **{r.detected}**")
        if r.build_commands:
            lines.append("- Build commands executed:")
            for cmd in r.build_commands:
                lines.append(f"  - `{cmd}`")
        else:
            lines.append("- Build commands executed: *(none)*")
        lines.append(f"- Status: **{r.status}**")
        if r.entrypoint:
            lines.append(f"- Suggested entrypoint: `{r.entrypoint}`")
        if r.error:
            lines.append("- Error (tail):")
            lines.append("  ```")
            lines.append(r.error.strip())
            lines.append("  ```")
        if r.integration_notes:
            lines.append("- Integration notes for PHOENIX:")
            for n in r.integration_notes:
                lines.append(f"  - {n}")
        lines.append("")

    lines.append("## Next Steps for Orchestration")
    lines.append("")
    lines.append("- Register each ORCH in Phoenix's ORCH registry (name, type, entrypoint, env/args).")
    lines.append("- For Python/Node ORCHs, spawn with tokio::process::Command and bridge via:")
    lines.append("  - stdin/stdout (JSON-lines recommended), or")
    lines.append("  - a local HTTP server (better for long-running tools)")
    lines.append("- For Rust/WASM ORCHs, load the produced `.wasm` via a WASM runtime and define a stable interface (WIT recommended).")
    lines.append("")
    return "\n".join(lines)


def main(argv: Optional[Sequence[str]] = None) -> int:
    ap = argparse.ArgumentParser(description="Clone + auto-build ORCH repos into ../orch_repos and generate docs.")
    ap.add_argument("urls", nargs="*", help="GitHub repo URLs")
    ap.add_argument("--config", type=str, help="Path to a newline-delimited URL file")
    ap.add_argument("--no-build", action="store_true", help="Only clone/update repos; skip build/install")
    ap.add_argument("--shallow", action="store_true", help="Use shallow clones (depth=1)")
    ap.add_argument(
        "--ci-gate",
        choices=["auto", "require", "off"],
        default="auto",
        help=(
            "GitHub Actions CI gate. "
            "auto=require CI only if workflow exists; require=fail if missing or not passing; off=skip check"
        ),
    )
    ap.add_argument("--ci-workflow", default="ci-tests.yml", help="Workflow file name to check (default: ci-tests.yml)")
    ap.add_argument("--ci-branch", default="main", help="Branch to check CI status for (default: main)")
    ap.add_argument(
        "--skip",
        action="append",
        default=[],
        help="Skip repos whose URL contains this substring (can be passed multiple times)",
    )
    args = ap.parse_args(list(argv) if argv is not None else None)

    project_root = _find_project_root()
    orch_dir = _orch_repos_dir(project_root)
    venvs_dir = orch_dir / ".venvs"
    orch_dir.mkdir(parents=True, exist_ok=True)
    venvs_dir.mkdir(parents=True, exist_ok=True)

    urls: List[str] = []
    if args.config:
        urls.extend(_load_urls_from_config(Path(args.config)))
    urls.extend(args.urls)
    urls = [u for u in urls if u.strip()]

    if not urls:
        print("No URLs provided. Provide URLs or --config.")
        return 2

    started = _dt.datetime.now(tz=_dt.timezone.utc)
    results: List[RepoResult] = []

    gh_token = os.environ.get("GITHUB_PAT") or os.environ.get("GITHUB_TOKEN")
    gh_ua = os.environ.get("GITHUB_USER_AGENT") or "phoenix-2.0-orch-integrator"

    for url in urls:
        if any(s in url for s in args.skip):
            name = _parse_repo_name(url)
            results.append(
                RepoResult(
                    name=name,
                    url=url,
                    dest_dir=orch_dir / name,
                    detected="Skipped",
                    build_commands=[],
                    status="Skipped",
                    error=None,
                    entrypoint=None,
                    integration_notes=["Skipped by --skip filter."],
                )
            )
            continue

        name = _parse_repo_name(url)
        dest = orch_dir / name
        print(f"\n=== [{name}] clone/update ===")

        # CI gate: verify GitHub Actions status before integrating.
        ci_notes: List[str] = []
        if args.ci_gate != "off":
            owner_repo = _parse_github_owner_repo(url)
            if owner_repo is None:
                ci_notes.append("CI gate: skipped (non-GitHub URL)")
            else:
                owner, repo = owner_repo
                passed, note, exists = _github_ci_gate(
                    owner=owner,
                    repo=repo,
                    workflow=args.ci_workflow,
                    branch=args.ci_branch,
                    token=gh_token,
                    user_agent=gh_ua,
                )
                ci_notes.append(note)
                print(note)
                if args.ci_gate == "require" and (not exists or not passed):
                    results.append(
                        RepoResult(
                            name=name,
                            url=url,
                            dest_dir=dest,
                            detected="CI Gate",
                            build_commands=[],
                            status="Failure",
                            error=note,
                            entrypoint=None,
                            integration_notes=ci_notes,
                        )
                    )
                    continue
                if args.ci_gate == "auto" and exists and not passed:
                    results.append(
                        RepoResult(
                            name=name,
                            url=url,
                            dest_dir=dest,
                            detected="CI Gate",
                            build_commands=[],
                            status="Failure",
                            error=note,
                            entrypoint=None,
                            integration_notes=ci_notes,
                        )
                    )
                    continue

        ok, clone_cmds, err = _git_clone_or_update(url, dest, shallow=bool(args.shallow))
        if not ok:
            results.append(
                RepoResult(
                    name=name,
                    url=url,
                    dest_dir=dest,
                    detected="Unknown",
                    build_commands=clone_cmds,
                    status="Failure",
                    error=err,
                    entrypoint=None,
                    integration_notes=ci_notes + ["Clone failed; build not attempted."],
                )
            )
            continue

        detected = _detect_repo_type(dest)
        build_cmds: List[str] = list(clone_cmds)
        entrypoint: Optional[str] = None
        notes: List[str] = []
        build_err: Optional[str] = None
        status: str = "Success"

        if args.no_build:
            status = "Skipped"
            notes = ci_notes + ["Build skipped due to --no-build."]
        else:
            print(f"=== [{name}] build ({detected}) ===")
            ok2, cmds2, err2, entry2, notes2 = _build_repo(dest, detected, venvs_dir=venvs_dir)
            build_cmds.extend(cmds2)
            entrypoint = entry2
            notes = ci_notes + notes2
            if not ok2:
                status = "Failure"
                build_err = err2

        results.append(
            RepoResult(
                name=name,
                url=url,
                dest_dir=dest,
                detected=detected,
                build_commands=build_cmds,
                status=status,
                error=build_err,
                entrypoint=entrypoint,
                integration_notes=notes,
            )
        )

    docs = _format_docs(project_root, orch_dir, results, started)
    out_path = project_root / "orch_repos_docs.md"
    out_path.write_text(docs, encoding="utf-8")
    print(f"\nWrote docs: {out_path}")

    # Non-zero exit if any failures.
    if any(r.status == "Failure" for r in results):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

