#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

python3 - "$ROOT" <<'PY'
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
errors = []
checked = 0

root_docs = [
    "README.md",
    "CONTRIBUTING.md",
    "CODE_OF_CONDUCT.md",
    "SECURITY.md",
    "SUPPORT.md",
    "GOVERNANCE.md",
    "MAINTAINERS.md",
    "CHANGELOG.md",
    "LICENSE",
]

docs_dir = root / "docs"
doc_files = sorted(p.name for p in docs_dir.glob("*.md")) if docs_dir.is_dir() else []
tracked = [root / name for name in root_docs] + [docs_dir / name for name in doc_files]

# Root .md documents must have a .zh.md counterpart; LICENSE has a special pair.
for name in root_docs:
    if name == "LICENSE":
        if not (root / "LICENSE.zh.md").is_file():
            errors.append("missing LICENSE.zh.md")
        continue
    if not (root / name).is_file():
        errors.append(f"missing root document: {name}")
        continue
    zh = name[:-3] + ".zh.md"
    if not (root / zh).is_file():
        errors.append(f"missing Chinese counterpart: {zh}")

# docs/*.md must be paired.
for name in doc_files:
    if name.endswith(".zh.md"):
        base = name[:-6] + ".md"  # strip .zh.md
        if not (docs_dir / base).is_file():
            errors.append(f"Chinese document has no English counterpart: docs/{name}")
        continue
    zh = name[:-3] + ".zh.md"
    if not (docs_dir / zh).is_file():
        errors.append(f"missing Chinese counterpart: docs/{zh}")

# Language switcher and local markdown links.
link_re = re.compile(r"\]\(([^)]+)\)")
for path in tracked:
    if not path.is_file():
        continue
    checked += 1
    text = path.read_text(encoding="utf-8")
    head = "\n".join(text.splitlines()[:8])
    if path.name != "LICENSE" and ("简体中文" not in head or "English" not in head):
        errors.append(f"missing language switcher near the top: {path.relative_to(root)}")

    # Ignore fenced code blocks so documentation templates are not treated as links.
    prose = []
    in_fence = False
    for line in text.splitlines():
        if line.lstrip().startswith("```"):
            in_fence = not in_fence
            continue
        if not in_fence:
            prose.append(line)
    body = re.sub(r"`[^`]*`", "", "\n".join(prose))

    for raw in link_re.findall(body):
        target = raw.split("#", 1)[0].strip()
        if not target or target.startswith(("http://", "https://", "mailto:", "#")):
            continue
        if target.startswith("<") and target.endswith(">"):
            target = target[1:-1]
        if not target.endswith((".md", ".zh.md")):
            continue
        resolved = (path.parent / target).resolve()
        if not resolved.is_file():
            errors.append(
                f"broken link in {path.relative_to(root)}: {raw} -> {resolved}"
            )

if errors:
    print("Documentation check failed:")
    for error in errors:
        print(f"  - {error}")
    sys.exit(1)

print(f"Documentation check passed ({checked} files).")
PY
