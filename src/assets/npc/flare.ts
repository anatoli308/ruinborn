/**
 * NPC FLARE atlas resolver — mirrors `player_male/flare.ts` but loads
 * `src/assets/npc/atlases/*.json` and `src/assets/npc/*.png`.
 *
 * Re-exports the shared types + `pickFrame` from the player_male module so
 * there is a single canonical FLARE format definition in the codebase.
 */
import type { FlareAtlas } from "../player_male/flare";

export type {
  FlareAnimation,
  FlareAnimationType,
  FlareAtlas,
  FlareFrame,
} from "../player_male/flare";
export { pickFrame } from "../player_male/flare";

const atlasModules = import.meta.glob<FlareAtlas>("./atlases/*.json", {
  eager: true,
  import: "default",
});

function keyOf(path: string): string {
  const file = path.split("/").pop() ?? path;
  return file.replace(/\.json$/i, "");
}

/** Atlas table keyed by bare name (e.g. `zombie`, `skeleton_archer`). */
export const NPC_FLARE_ATLASES: Record<string, FlareAtlas> = Object.fromEntries(
  Object.entries(atlasModules).map(([path, atlas]) => [keyOf(path), atlas]),
);

const pngModules = import.meta.glob<string>("./*.png", {
  eager: true,
  import: "default",
  query: "?url",
});

/** PNG filename → bundled URL (e.g. `npc_zombie.png` → `/assets/npc_zombie-abc.png`). */
export const NPC_FLARE_IMAGE_URLS: Record<string, string> = Object.fromEntries(
  Object.entries(pngModules).map(([path, url]) => [
    path.split("/").pop() ?? path,
    url,
  ]),
);

/**
 * Resolve an NPC atlas + its bundled PNG URL.
 * Returns `null` if either the atlas or the PNG is missing.
 */
export function getNpcFlareLayer(name: string): {
  atlas: FlareAtlas;
  imageUrl: string;
} | null {
  const atlas = NPC_FLARE_ATLASES[name];
  if (!atlas) return null;
  const imageUrl = NPC_FLARE_IMAGE_URLS[atlas.image];
  if (!imageUrl) return null;
  return { atlas, imageUrl };
}
