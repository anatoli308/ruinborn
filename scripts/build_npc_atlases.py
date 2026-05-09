"""Convert FLARE-style .txt animation defs to atlas JSON format used by the
Ruinborn frontend, then copy them into `src/assets/npc/atlases/` so they
are picked up by Vite's `import.meta.glob`. Mirrors the player_male format.

Source field order (per `frame=` line):
    direction, frame_index, x, y, w, h, ox, oy

Output JSON shape (per anim):
    {
      "frames_count": N,
      "duration_ms": M,
      "type": "looped" | "back_forth" | "play_once",
      "frames": [ [dir0..dir7], ... ]   # outer index = frame_index
    }
"""
from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path

SRC_DIR = Path(r"c:\Users\anato\Downloads\portraits\npc")
ASSETS_NPC = Path(r"d:\projects\tradewars\src\assets\npc")
OUT_ATLASES = ASSETS_NPC / "atlases"
OUT_ATLASES.mkdir(parents=True, exist_ok=True)

DURATION_RE = re.compile(r"(\d+)\s*ms?")


def parse_duration(value: str) -> int:
    m = DURATION_RE.search(value)
    return int(m.group(1)) if m else 0


def parse_txt(path: Path) -> dict:
    image = ""
    animations: dict[str, dict] = {}
    current: str | None = None
    cur_obj: dict | None = None
    frames_per_anim: dict[str, dict[int, dict[int, dict]]] = {}

    for raw in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            current = line[1:-1].strip()
            cur_obj = {"frames_count": 0, "duration_ms": 0, "type": "looped"}
            animations[current] = cur_obj
            frames_per_anim.setdefault(current, {})
            continue
        if "=" not in line:
            continue
        key, _, value = line.partition("=")
        key = key.strip().lower()
        value = value.strip()

        if current is None:
            if key == "image":
                image = value
            continue

        assert cur_obj is not None
        if key == "frames":
            cur_obj["frames_count"] = int(value)
        elif key == "duration":
            cur_obj["duration_ms"] = parse_duration(value)
        elif key == "type":
            cur_obj["type"] = value
        elif key == "frame":
            parts = [p.strip() for p in value.split(",")]
            if len(parts) != 8:
                continue
            d, fi, x, y, w, h, ox, oy = (int(p) for p in parts)
            frames_per_anim[current].setdefault(fi, {})[d] = {
                "x": x, "y": y, "w": w, "h": h, "ox": ox, "oy": oy,
            }

    # Materialise frames as [frame_index][direction 0..7]
    for name, anim in animations.items():
        count = anim["frames_count"]
        per_frame = frames_per_anim.get(name, {})
        if count <= 0 and per_frame:
            count = max(per_frame.keys()) + 1
            anim["frames_count"] = count
        anim["frames"] = [
            [per_frame.get(i, {}).get(d) for d in range(8)]
            for i in range(count)
        ]
        # Normalise unknown types to looped to be safe.
        if anim["type"] not in {"looped", "back_forth", "play_once"}:
            anim["type"] = "looped"

    return {"image": image, "animations": animations}


def expected_png_for(image_field: str) -> Path:
    """Source files reference e.g. `npc_zombie.png`. They live in ASSETS_NPC."""
    return ASSETS_NPC / image_field


def main() -> int:
    txts = sorted(p for p in SRC_DIR.glob("*.txt"))
    if not txts:
        print("no .txt files found", file=sys.stderr)
        return 1

    written = 0
    skipped: list[str] = []
    missing_png: list[str] = []

    for txt in txts:
        data = parse_txt(txt)
        image = data.get("image") or ""
        if not image:
            skipped.append(f"{txt.name}: no image= line")
            continue
        png = expected_png_for(image)
        if not png.exists():
            # Try common spelling alternatives the user may have on disk.
            alt = None
            stem = png.stem
            for cand in ASSETS_NPC.glob("*.png"):
                if cand.stem.lower() == stem.lower():
                    alt = cand
                    break
            if alt is None and "warebear" in stem.lower():
                cand = ASSETS_NPC / "npc_warebear.png"
                if cand.exists():
                    alt = cand
            if alt is None:
                missing_png.append(f"{txt.name} -> {image}")
                continue
            data["image"] = alt.name

        out_name = txt.stem + ".json"
        out_path = OUT_ATLASES / out_name
        out_path.write_text(json.dumps(data, indent=2), encoding="utf-8")
        written += 1

    print(f"wrote {written} atlases to {OUT_ATLASES}")
    if missing_png:
        print(f"missing PNG for {len(missing_png)} txts:")
        for m in missing_png:
            print("  -", m)
    if skipped:
        print(f"skipped: {skipped}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
