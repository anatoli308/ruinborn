import { Suspense, useMemo } from "react";
import { useTexture } from "@react-three/drei";
import * as THREE from "three";
import { useGameStore } from "../../store/gameStore";
import biomesJson from "../../data/biomes.json";
import layoutsJson from "../../data/zone_layouts.json";
import { SPRITE_TILT } from "./cameraConfig";

// ── Vite glob: alle Map-Texturen einmalig als URLs einsammeln ────────────
const TEXTURE_URLS = import.meta.glob<string>("/src/assets/map/*.png", {
  eager: true,
  query: "?url",
  import: "default",
});

/** PNG-Dateiname (ohne .png) → URL, oder undefined wenn nicht vorhanden. */
function texUrl(name: string): string | undefined {
  return TEXTURE_URLS[`/src/assets/map/${name}.png`];
}

// ── Schema-Typen (mirror src/data/biomes.json + zone_layouts.json) ───────

interface BiomePreset {
  ground: string;
  ground_tile_size: number;
  fog_color: string;
  fog_near: number;
  fog_far: number;
  ambient_tint: string;
  directional_tint: string;
}

interface PropDef {
  tex: string;
  x: number;
  z: number;
  scale?: number;
}

interface WallSegment {
  tex: string;
  from: [number, number];
  to: [number, number];
  /** Distance between sprites along the segment. Default 3.0. */
  spacing?: number;
  /** Sprite scale. Default 3.0. */
  scale?: number;
}

interface ZoneLayout {
  biome: string;
  ground?: string;
  props?: PropDef[];
  walls?: WallSegment[];
}

const BIOMES = (biomesJson as { biomes: Record<string, BiomePreset> }).biomes;
const LAYOUTS = (layoutsJson as unknown as { zones: Record<string, ZoneLayout> })
  .zones;

const FALLBACK_BIOME: BiomePreset = BIOMES.town_camp ?? BIOMES.grassland;

const DEFAULT_BOUNDS = { minX: -40, maxX: 40, minZ: -30, maxZ: 30 };
const DEFAULT_ZONE_ID = "rogue_encampment";

/** Boden wird ueber den Zonen-Rand hinaus gerendert (Nebel verschluckt den Rand). */
const GROUND_OVERSCAN = 100;

/**
 * Strata Zone Renderer — Phase A: Town-Focus.
 *
 * • Boden  – stark uebergrosse, gekachelte Plane (verschwindet im Fog → "endlos").
 * • Props  – fixed-iso Sprites (Plane im Kamera-Pitch geneigt, kein Billboard).
 * • Walls  – Segmente, automatisch in einzelne Wand-Sprites expandiert.
 *
 * Faellt auf rogue_encampment-Defaults zurueck, solange der Server-Snapshot
 * fehlt — sonst ist der Login-Bildschirm leer.
 */
export default function Terrain() {
  const zones = useGameStore((s) => s.zones);
  const currentZoneId = useGameStore((s) => s.zone);

  const liveZone = zones.find((z) => z.id === currentZoneId);
  const zoneId = liveZone?.id ?? DEFAULT_ZONE_ID;
  const bounds = liveZone?.bounds ?? DEFAULT_BOUNDS;

  const layout: ZoneLayout = LAYOUTS[zoneId] ?? LAYOUTS[DEFAULT_ZONE_ID] ?? { biome: "town_camp" };
  const biome: BiomePreset = BIOMES[layout.biome] ?? FALLBACK_BIOME;

  const groundTexName = layout.ground ?? biome.ground;
  const groundUrl = texUrl(groundTexName);

  const cx = (bounds.maxX + bounds.minX) / 2;
  const cz = (bounds.maxZ + bounds.minZ) / 2;
  const groundWidth = bounds.maxX - bounds.minX + GROUND_OVERSCAN * 2;
  const groundDepth = bounds.maxZ - bounds.minZ + GROUND_OVERSCAN * 2;

  const propSprites = useMemo(() => expandProps(layout.props), [layout.props]);
  const wallSprites = useMemo(() => expandWalls(layout.walls), [layout.walls]);

  return (
    <Suspense fallback={null}>
      {groundUrl ? (
        <Ground
          url={groundUrl}
          width={groundWidth}
          depth={groundDepth}
          cx={cx}
          cz={cz}
          tileSize={biome.ground_tile_size}
        />
      ) : (
        <SolidGround width={groundWidth} depth={groundDepth} cx={cx} cz={cz} />
      )}

      {[...propSprites, ...wallSprites].map((p, i) => (
        <IsoSprite
          key={`${zoneId}-${i}-${p.tex}`}
          url={p.url}
          x={cx + p.x}
          z={cz + p.z}
          scale={p.scale}
        />
      ))}
    </Suspense>
  );
}

// ── Layout-Expansion ─────────────────────────────────────────────────────

interface ResolvedSprite {
  tex: string;
  url: string;
  x: number;
  z: number;
  scale: number;
}

function expandProps(props: PropDef[] | undefined): ResolvedSprite[] {
  if (!props) return [];
  return props
    .map((p) => {
      const url = texUrl(p.tex);
      if (!url) return null;
      return { tex: p.tex, url, x: p.x, z: p.z, scale: p.scale ?? 2 };
    })
    .filter((p): p is ResolvedSprite => p !== null);
}

function expandWalls(walls: WallSegment[] | undefined): ResolvedSprite[] {
  if (!walls) return [];
  const out: ResolvedSprite[] = [];
  for (const seg of walls) {
    const url = texUrl(seg.tex);
    if (!url) continue;
    const [x1, z1] = seg.from;
    const [x2, z2] = seg.to;
    const dx = x2 - x1;
    const dz = z2 - z1;
    const length = Math.hypot(dx, dz);
    const spacing = seg.spacing ?? 3.0;
    const scale = seg.scale ?? 3.0;
    const count = Math.max(2, Math.floor(length / spacing) + 1);
    for (let i = 0; i < count; i++) {
      const t = count === 1 ? 0 : i / (count - 1);
      out.push({
        tex: seg.tex,
        url,
        x: x1 + dx * t,
        z: z1 + dz * t,
        scale,
      });
    }
  }
  return out;
}

// ── Sub-Komponenten ──────────────────────────────────────────────────────

interface GroundProps {
  url: string;
  width: number;
  depth: number;
  cx: number;
  cz: number;
  tileSize: number;
}

function Ground({ url, width, depth, cx, cz, tileSize }: GroundProps) {
  const tex = useTexture(url);
  tex.wrapS = THREE.RepeatWrapping;
  tex.wrapT = THREE.RepeatWrapping;
  tex.repeat.set(Math.max(1, width / tileSize), Math.max(1, depth / tileSize));
  tex.colorSpace = THREE.SRGBColorSpace;
  tex.anisotropy = 4;
  tex.needsUpdate = true;

  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]} position={[cx, 0, cz]} receiveShadow>
      <planeGeometry args={[width, depth]} />
      <meshStandardMaterial map={tex} roughness={0.95} />
    </mesh>
  );
}

function SolidGround({
  width,
  depth,
  cx,
  cz,
}: {
  width: number;
  depth: number;
  cx: number;
  cz: number;
}) {
  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]} position={[cx, 0, cz]} receiveShadow>
      <planeGeometry args={[width, depth]} />
      <meshStandardMaterial color="#4a3a28" roughness={1} />
    </mesh>
  );
}

interface IsoSpriteProps {
  url: string;
  x: number;
  z: number;
  scale: number;
}

/**
 * Fixed-iso Sprite — eine Plane, die exakt im Kamera-Pitch geneigt ist und
 * sich NIE dreht (anders als ein Billboard). Dadurch wirken die Sprites wie
 * pre-baked D2-Iso-Art und nicht "papp-aufgeklebt".
 *
 * Aufbau:
 *   • Group am Boden bei (x, 0, z), gekippt um SPRITE_TILT um die X-Achse.
 *   • Child-Mesh lokal verschoben um (0, scale/2, 0) → die Plane steht auf
 *     dem Boden mit korrektem "Footprint" bei (x, z).
 */
function IsoSprite({ url, x, z, scale }: IsoSpriteProps) {
  const map = useTexture(url);
  map.colorSpace = THREE.SRGBColorSpace;
  map.magFilter = THREE.NearestFilter;
  map.minFilter = THREE.NearestFilter;
  map.needsUpdate = true;

  return (
    <group position={[x, 0, z]} rotation={[SPRITE_TILT, 0, 0]}>
      <mesh position={[0, scale / 2, 0]}>
        <planeGeometry args={[scale, scale]} />
        <meshBasicMaterial
          map={map}
          transparent
          alphaTest={0.3}
          side={THREE.DoubleSide}
          depthWrite={false}
        />
      </mesh>
    </group>
  );
}
