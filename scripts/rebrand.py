"""One-off rebrand helper: replace TradeWars/tradewars references with Ruinborn/ruinborn.

Targeted: only specified files, skips Cargo.lock/target/node_modules.
Replacements (longest first to avoid partial overlaps):
  TradeWars        -> Ruinborn
  TRADEWARS        -> RUINBORN
  tradewars-game   -> ruinborn-game
  tradewars-protocol -> ruinborn-protocol
  tradewars-server -> ruinborn-server
  tradewars_game   -> ruinborn_game
  tradewars_protocol -> ruinborn_protocol
  tradewars_server -> ruinborn_server
  tradewars        -> ruinborn   (lowercase plain — for db name, etc.)

NOTE: We do NOT touch the workspace folder name `d:\\projects\\tradewars\\`.
That stays — only the `crates/tradewars-*` segment inside paths is renamed
(handled by the longer patterns above which match first).
"""
from __future__ import annotations
import pathlib

ROOT = pathlib.Path(r"d:\projects\tradewars")

# Files to rewrite (relative to ROOT).
TARGETS = [
    "crates/ruinborn-server/src/db.rs",
    "crates/ruinborn-server/src/db_sea.rs",
    "crates/ruinborn-protocol/src/lib.rs",
    "package.json",
    "package-lock.json",
    "index.html",
    "README.md",
    ".env.example",
    ".github/copilot-instructions.md",
    "docs/OVERVIEW.md",
    "docs/ARCHITECTURE.md",
    "docs/NETWORKING.md",
    "docs/DAMAGE_MODEL.md",
    "docs/IDEEN.md",
    "src/components/MainMenu.tsx",
    "src/components/ui/ActionBar.tsx",
    "src/components/ui/SkillTreePanel.tsx",
    "src/components/world/Waypoints.tsx",
    "src/components/world/Terrain.tsx",
    "src/data/classes.ts",
    "scripts/build_npc_atlases.py",
    "scripts/write_market.py",
    "scripts/write_store.py",
]

# Order matters: longer patterns first so we don't half-replace.
REPLACEMENTS: list[tuple[str, str]] = [
    ("TradeWars", "Ruinborn"),
    ("TRADEWARS", "RUINBORN"),
    ("Tradewars", "Ruinborn"),
    ("tradewars-game", "ruinborn-game"),
    ("tradewars-protocol", "ruinborn-protocol"),
    ("tradewars-server", "ruinborn-server"),
    ("tradewars_game", "ruinborn_game"),
    ("tradewars_protocol", "ruinborn_protocol"),
    ("tradewars_server", "ruinborn_server"),
    ("tradewars_lib", "ruinborn_lib"),
    ("x-tradewars-skill", "x-ruinborn-skill"),
    # Lowercase plain "tradewars" — careful: must NOT touch the workspace
    # folder path `d:\projects\tradewars\`. We handle that by replacing
    # only the standalone token.
    # Special: db name in `localhost/tradewars` and similar strings.
    ("/tradewars", "/ruinborn"),
    # package.json `"name": "tradewars"`
    ('"name": "tradewars"', '"name": "ruinborn"'),
]


def rewrite(path: pathlib.Path) -> bool:
    text = path.read_text(encoding="utf-8")
    new = text
    for old, new_s in REPLACEMENTS:
        new = new.replace(old, new_s)
    if new != text:
        path.write_text(new, encoding="utf-8")
        return True
    return False


def main() -> None:
    changed = []
    skipped = []
    for rel in TARGETS:
        p = ROOT / rel
        if not p.exists():
            skipped.append(rel)
            continue
        if rewrite(p):
            changed.append(rel)
    print(f"changed {len(changed)} files:")
    for c in changed:
        print(f"  - {c}")
    if skipped:
        print(f"missing {len(skipped)} files (skipped):")
        for s in skipped:
            print(f"  - {s}")


if __name__ == "__main__":
    main()
