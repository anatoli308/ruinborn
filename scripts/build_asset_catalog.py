"""Build asset catalog JSON from steam-main mapeditor scripts and the local zip distribution.

Inputs:
  - <SRC>/mapeditor_defs/{terrain.txt, foliage.txt, texture_names.txt}
  - <SRC>/mapeditor_textures/*.txt          (one filename per line, blank lines = group breaks)
  - <ZIPS>/src/assets/map/*.zip             (actual texture distribution)
  - <ZIPS>/src/assets/{misc,npc,spell_icons}.zip optional

Output:
  crates/ruinborn-game/data/asset_catalog.json

Schema:
{
  "_doc": "...",
  "aliases":   { "TiledRoom_1.png": "floor1", ... },          # from texture_names.txt
  "groups":    { "ground": [...], "foliage_base": [...] },    # from mapeditor_defs/terrain.txt + foliage.txt
  "categories": {
    "building_house": {
      "kind": "building",                                     # auto-derived from filename prefix
      "textures": ["Building_01.png", ...],
      "subgroups": [["Building_01.png", ...], [ ... ]]        # blank-line separated source clusters
    },
    ...
  },
  "particles": ["green_firefly.psi", ...],
  "zip_index": { "map_textures_1.zip": ["TiledRoom_1.png", ...], ... },
  "missing": ["foo.png", ...],                                # referenced but not in any zip
  "orphans": ["bar.png", ...]                                 # in zip but never referenced
}
"""
from __future__ import annotations

import argparse
import json
import sys
import zipfile
from collections import defaultdict
from pathlib import Path


CATEGORY_KIND_PREFIXES = (
    ("xparticle_system", "particles"),
    ("xmap_terrain_", "xmap"),
    ("tileset_", "tileset"),
    ("building_", "building"),
    ("object_", "object"),
    ("terrain_foilage_", "foliage"),
    ("terrain_tree_", "tree"),
    ("terrain_rock", "rock"),
    ("terrain_wall_", "wall"),
    ("terrain_flooring_", "flooring"),
    ("terrain_bridge", "bridge"),
    ("terrain", "terrain"),
    ("uncategorized_", "uncategorized"),
)


def category_kind(stem: str) -> str:
    for prefix, kind in CATEGORY_KIND_PREFIXES:
        if stem.startswith(prefix):
            return kind
    return "other"


def parse_subgroups(text: str) -> list[list[str]]:
    """Split text on blank lines; each non-empty cluster is a subgroup of filenames."""
    clusters: list[list[str]] = []
    current: list[str] = []
    for raw in text.splitlines():
        line = raw.strip()
        if not line:
            if current:
                clusters.append(current)
                current = []
            continue
        current.append(line)
    if current:
        clusters.append(current)
    return clusters


def parse_aliases(text: str) -> dict[str, str]:
    aliases: dict[str, str] = {}
    for line in text.splitlines():
        line = line.strip()
        if not line or "=" not in line:
            continue
        png, alias = line.split("=", 1)
        aliases[png.strip()] = alias.strip()
    return aliases


def index_zips(asset_root: Path) -> dict[str, list[str]]:
    index: dict[str, list[str]] = {}
    for zip_path in sorted(asset_root.rglob("*.zip")):
        try:
            with zipfile.ZipFile(zip_path) as zf:
                names = [n for n in zf.namelist() if not n.endswith("/")]
            index[zip_path.name] = names
        except zipfile.BadZipFile:
            print(f"WARN: bad zip {zip_path}", file=sys.stderr)
    return index


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--src",
        default=r"C:\Users\anato\Downloads\steam-main\scripts",
        help="Path to steam-main/scripts (contains mapeditor_defs and mapeditor_textures).",
    )
    parser.add_argument(
        "--zips",
        default=str(Path(__file__).resolve().parent.parent / "src" / "assets"),
        help="Root directory holding the .zip asset bundles.",
    )
    parser.add_argument(
        "--out",
        default=str(
            Path(__file__).resolve().parent.parent
            / "crates"
            / "ruinborn-game"
            / "data"
            / "asset_catalog.json"
        ),
        help="Output JSON path.",
    )
    args = parser.parse_args()

    src = Path(args.src)
    defs_dir = src / "mapeditor_defs"
    tex_dir = src / "mapeditor_textures"
    if not tex_dir.is_dir():
        print(f"ERROR: {tex_dir} not found", file=sys.stderr)
        return 1

    aliases = parse_aliases((defs_dir / "texture_names.txt").read_text(encoding="utf-8"))
    groups = {
        "ground": [
            ln.strip()
            for ln in (defs_dir / "terrain.txt").read_text(encoding="utf-8").splitlines()
            if ln.strip()
        ],
        "foliage_base": [
            ln.strip()
            for ln in (defs_dir / "foliage.txt").read_text(encoding="utf-8").splitlines()
            if ln.strip()
        ],
    }

    categories: dict[str, dict] = {}
    particles: list[str] = []

    for txt in sorted(tex_dir.glob("*.txt")):
        stem = txt.stem
        text = txt.read_text(encoding="utf-8")
        subgroups = parse_subgroups(text)
        flat = [name for sub in subgroups for name in sub]
        kind = category_kind(stem)
        if kind == "particles":
            particles.extend(flat)
            continue
        categories[stem] = {
            "kind": kind,
            "textures": flat,
            "subgroups": subgroups if len(subgroups) > 1 else [],
        }

    zip_index = index_zips(Path(args.zips))
    all_in_zips: set[str] = set()
    for names in zip_index.values():
        for n in names:
            all_in_zips.add(Path(n).name)

    referenced: set[str] = set()
    for cat in categories.values():
        referenced.update(cat["textures"])
    referenced.update(particles)
    for grp in groups.values():
        referenced.update(grp)

    missing = sorted(referenced - all_in_zips)
    orphans = sorted(all_in_zips - referenced)

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(
        json.dumps(
            {
                "_doc": (
                    "Generated by scripts/build_asset_catalog.py from "
                    "steam-main/scripts/mapeditor_* and src/assets/*.zip. "
                    "Re-run after editing source .txt files or replacing zips."
                ),
                "aliases": aliases,
                "groups": groups,
                "categories": dict(sorted(categories.items())),
                "particles": particles,
                "zip_index": dict(sorted(zip_index.items())),
                "stats": {
                    "categories": len(categories),
                    "textures_referenced": len(referenced),
                    "textures_in_zips": len(all_in_zips),
                    "missing": len(missing),
                    "orphans": len(orphans),
                },
                "missing": missing,
                "orphans": orphans,
            },
            indent=2,
            ensure_ascii=False,
        ),
        encoding="utf-8",
    )

    print(f"Wrote {out_path}")
    print(f"  categories            : {len(categories)}")
    print(f"  textures referenced   : {len(referenced)}")
    print(f"  textures in zips      : {len(all_in_zips)}")
    print(f"  missing (ref, !zip)   : {len(missing)}")
    print(f"  orphans (zip, !ref)   : {len(orphans)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
