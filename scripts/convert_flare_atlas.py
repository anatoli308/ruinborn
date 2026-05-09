"""Convert FLARE engine animation .txt definitions to JSON.

FLARE format (per line in an [animation] block):
    frame = frame_index, direction, x, y, w, h, offset_x, offset_y

- 8 directions per frame index (0..7, starting north, going clockwise)
- offset_x/offset_y = pivot offset from sprite top-left to feet anchor

Usage:
    python scripts/convert_flare_atlas.py <src_dir> <dst_dir>

Reads every *.txt in src_dir and writes <name>.json to dst_dir.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

HEADER_RE = re.compile(r"^\[(?P<name>[a-zA-Z0-9_]+)\]\s*$")
KV_RE = re.compile(r"^(?P<key>[a-zA-Z_]+)\s*=\s*(?P<value>.+?)\s*$")


def parse_duration_ms(value: str) -> int:
    value = value.strip().lower()
    if value.endswith("ms"):
        return int(value[:-2])
    if value.endswith("s"):
        return int(float(value[:-1]) * 1000)
    return int(value)


def parse_file(path: Path) -> dict:
    image: str | None = None
    animations: dict[str, dict] = {}
    current: dict | None = None
    current_name: str | None = None

    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue

        header = HEADER_RE.match(line)
        if header:
            current_name = header.group("name")
            current = {
                "frames_count": 0,
                "duration_ms": 0,
                "type": "play_once",
                "frames": [],
            }
            animations[current_name] = current
            continue

        kv = KV_RE.match(line)
        if not kv:
            continue
        key, value = kv.group("key"), kv.group("value")

        if key == "image" and current is None:
            image = value.strip()
            continue

        if current is None:
            continue

        if key == "frames":
            current["frames_count"] = int(value)
            current["frames"] = [[None] * 8 for _ in range(int(value))]
        elif key == "duration":
            current["duration_ms"] = parse_duration_ms(value)
        elif key == "type":
            current["type"] = value.strip()
        elif key == "frame":
            parts = [int(p.strip()) for p in value.split(",")]
            if len(parts) != 8:
                raise ValueError(f"{path.name}: bad frame line: {value}")
            idx, direction, x, y, w, h, ox, oy = parts
            while len(current["frames"]) <= idx:
                current["frames"].append([None] * 8)
            current["frames"][idx][direction] = {
                "x": x, "y": y, "w": w, "h": h, "ox": ox, "oy": oy,
            }

    return {
        "image": image,
        "animations": animations,
    }


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(__doc__)
        return 1
    src = Path(argv[1])
    dst = Path(argv[2])
    dst.mkdir(parents=True, exist_ok=True)

    written = 0
    skipped = 0
    for txt in sorted(src.glob("*.txt")):
        try:
            data = parse_file(txt)
        except Exception as e:  # noqa: BLE001
            print(f"SKIP {txt.name}: {e}")
            skipped += 1
            continue
        if not data.get("animations"):
            print(f"SKIP {txt.name}: no animations")
            skipped += 1
            continue
        out = dst / f"{txt.stem}.json"
        out.write_text(json.dumps(data, indent=2), encoding="utf-8")
        written += 1

    print(f"wrote {written} json files to {dst} (skipped {skipped})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
