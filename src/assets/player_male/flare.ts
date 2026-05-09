/**
 * FLARE animation atlas types and loaders.
 *
 * A FLARE spritesheet is organised as:
 *   animations[name].frames[frameIndex][direction] = Rect
 *
 * - 8 directions per frame (0 = North, then clockwise).
 * - `ox` / `oy` are the pivot offsets from the sprite's top-left
 *   to the character's feet anchor. Subtract them when placing
 *   the sprite so the feet sit on the world position.
 */

export interface FlareFrame {
  x: number;
  y: number;
  w: number;
  h: number;
  ox: number;
  oy: number;
}

export type FlareAnimationType = "play_once" | "looped" | "back_forth";

export interface FlareAnimation {
  frames_count: number;
  duration_ms: number;
  type: FlareAnimationType;
  /** [frameIndex][direction 0..7] — slot may be null if the source omitted it. */
  frames: (FlareFrame | null)[][];
}

export interface FlareAtlas {
  /** Filename of the source PNG (e.g. `player_male_default_chest.png`). */
  image: string;
  animations: Record<string, FlareAnimation>;
}

// Eager-load every atlas JSON next to this file.
// Vite's import.meta.glob returns a record keyed by file path.
const atlasModules = import.meta.glob<FlareAtlas>("./atlases/*.json", {
  eager: true,
  import: "default",
});

/** Strip path + extension to get the bare atlas key (e.g. `default_chest`). */
function keyOf(path: string): string {
  const file = path.split("/").pop() ?? path;
  return file.replace(/\.json$/i, "");
}

/** Eagerly-loaded atlas table keyed by name (e.g. `default_chest`, `head_short`). */
export const FLARE_ATLASES: Record<string, FlareAtlas> = Object.fromEntries(
  Object.entries(atlasModules).map(([path, atlas]) => [keyOf(path), atlas]),
);

// Eager-load every PNG so atlas.image -> URL is resolvable at runtime.
const pngModules = import.meta.glob<string>("./*.png", {
  eager: true,
  import: "default",
  query: "?url",
});

/** Lookup table: PNG filename -> bundled URL. */
export const FLARE_IMAGE_URLS: Record<string, string> = Object.fromEntries(
  Object.entries(pngModules).map(([path, url]) => [
    path.split("/").pop() ?? path,
    url,
  ]),
);

/**
 * Resolve an atlas + its bundled PNG URL.
 * Returns `null` if either piece is missing — callers can fall back gracefully.
 */
export function getFlareLayer(name: string): {
  atlas: FlareAtlas;
  imageUrl: string;
} | null {
  const atlas = FLARE_ATLASES[name];
  if (!atlas) return null;
  const imageUrl = FLARE_IMAGE_URLS[atlas.image];
  if (!imageUrl) return null;
  return { atlas, imageUrl };
}

/**
 * Pick the right frame for an animation at a given elapsed time + 8-way direction.
 * Honors `play_once`, `looped`, and `back_forth` timing types.
 */
export function pickFrame(
  anim: FlareAnimation,
  elapsedMs: number,
  direction: number,
): FlareFrame | null {
  const dir = ((direction % 8) + 8) % 8;
  const count = anim.frames_count;
  if (count <= 0) return null;
  const perFrame = anim.duration_ms / count;
  if (perFrame <= 0) {
    return anim.frames[0]?.[dir] ?? null;
  }

  let idx: number;
  switch (anim.type) {
    case "looped": {
      idx = Math.floor((elapsedMs / perFrame) % count);
      break;
    }
    case "back_forth": {
      const cycle = Math.max(1, count * 2 - 2);
      const t = Math.floor(elapsedMs / perFrame) % cycle;
      idx = t < count ? t : cycle - t;
      break;
    }
    case "play_once":
    default: {
      idx = Math.min(count - 1, Math.floor(elapsedMs / perFrame));
      break;
    }
  }
  return anim.frames[idx]?.[dir] ?? null;
}
