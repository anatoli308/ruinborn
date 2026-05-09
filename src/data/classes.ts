import type { ClassId, ClassInfo, SkillDef } from "../types";
import { SKILL_ID_TO_ICON } from "../assets/spell_icons";

/** Class catalog — mirror of `crates/ruinborn-game/src/classes.rs::class_definition`. */
export const CLASS_CATALOG: ClassInfo[] = [
  {
    id: "barbarian",
    name: "Barbarian",
    tagline: "Brutal warrior — Strength & Vitality",
    icon: "🪓",
    baseStats: { strength: 30, dexterity: 20, vitality: 25, energy: 10 },
    starterSkills: ["bash"],
  },
  {
    id: "sorceress",
    name: "Sorceress",
    tagline: "Elemental mage — Energy & Mana",
    icon: "🔮",
    baseStats: { strength: 10, dexterity: 25, vitality: 10, energy: 35 },
    starterSkills: ["fireball"],
  },
  {
    id: "necromancer",
    name: "Necromancer",
    tagline: "Bone mage — spirits & curses",
    icon: "💀",
    baseStats: { strength: 15, dexterity: 25, vitality: 15, energy: 25 },
    starterSkills: ["bone_spear"],
  },
];

export function classInfo(id: ClassId): ClassInfo {
  return CLASS_CATALOG.find((c) => c.id === id) ?? CLASS_CATALOG[0];
}

/** Skill catalog — mirror of `crates/ruinborn-game/src/skills.rs::skill_catalog`. */
export const SKILL_CATALOG: SkillDef[] = [
  // Barbarian
  {
    id: "bash",
    name: "Bash",
    classId: "barbarian",
    manaCost: 2,
    cooldownTicks: 6,
    range: 2.5,
    requiresLevel: 1,
    effect: "direct_damage",
    damageType: "physical",
    tags: ["melee"],
    icon: SKILL_ID_TO_ICON.bash,
    description: "Heavy strike against a single target.",
  },
  {
    id: "cleave",
    name: "Cleave",
    classId: "barbarian",
    manaCost: 6,
    cooldownTicks: 14,
    range: 3.0,
    requiresLevel: 6,
    effect: "aoe_around",
    damageType: "physical",
    tags: ["melee"],
    icon: SKILL_ID_TO_ICON.cleave,
    description: "Hits all enemies in melee range.",
  },
  {
    id: "battle_cry",
    name: "Battle Cry",
    classId: "barbarian",
    manaCost: 8,
    cooldownTicks: 60,
    range: 0,
    requiresLevel: 12,
    effect: "self_buff",
    damageType: null,
    tags: [],
    icon: SKILL_ID_TO_ICON.battle_cry,
    description: "A roar — short self-buff.",
  },
  // Sorceress
  {
    id: "fireball",
    name: "Fireball",
    classId: "sorceress",
    manaCost: 5,
    cooldownTicks: 8,
    range: 9.0,
    requiresLevel: 1,
    effect: "direct_damage",
    damageType: "fire",
    tags: ["spell", "ranged"],
    icon: SKILL_ID_TO_ICON.fireball,
    description: "Hurls a burning orb.",
  },
  {
    id: "frost_nova",
    name: "Frost Nova",
    classId: "sorceress",
    manaCost: 9,
    cooldownTicks: 24,
    range: 4.0,
    requiresLevel: 6,
    effect: "aoe_around",
    damageType: "cold",
    tags: ["spell"],
    icon: "❄️",
    description: "Ice explosion around the sorceress.",
  },
  {
    id: "teleport",
    name: "Teleport",
    classId: "sorceress",
    manaCost: 12,
    cooldownTicks: 18,
    range: 12.0,
    requiresLevel: 12,
    effect: "teleport",
    damageType: null,
    tags: ["spell"],
    icon: "✨",
    description: "Jumps to a target location.",
  },
  // Necromancer
  {
    id: "bone_spear",
    name: "Bone Spear",
    classId: "necromancer",
    manaCost: 4,
    cooldownTicks: 8,
    range: 10.0,
    requiresLevel: 1,
    effect: "direct_damage",
    damageType: "magical",
    tags: ["spell", "ranged"],
    icon: SKILL_ID_TO_ICON.bone_spear,
    description: "Fires a magical bone spear.",
  },
  {
    id: "raise_skeleton",
    name: "Raise Skeleton",
    classId: "necromancer",
    manaCost: 10,
    cooldownTicks: 30,
    range: 0,
    requiresLevel: 6,
    effect: "placeholder",
    damageType: null,
    tags: ["summoning"],
    icon: SKILL_ID_TO_ICON.raise_skeleton,
    description: "Summons a skeleton warrior (coming in Phase 4).",
  },
  {
    id: "amplify_damage",
    name: "Poison Cloud",
    classId: "necromancer",
    manaCost: 6,
    cooldownTicks: 15,
    range: 8.0,
    requiresLevel: 12,
    effect: "damage_over_time",
    damageType: "poison",
    tags: ["spell", "trap"],
    icon: "☠️",
    description: "Poisons an enemy over several seconds.",
  },
];

export function skillDef(id: string): SkillDef | undefined {
  return SKILL_CATALOG.find((s) => s.id === id);
}

export function skillsForClass(id: ClassId): SkillDef[] {
  return SKILL_CATALOG.filter((s) => s.classId === id);
}

// ─── Targeting kind ──────────────────────────────────────────────────────

/**
 * How a skill should be visualized in the world while the player is
 * preparing to cast it. Drives the on-ground range indicator.
 */
export type SkillTargetingKind =
  /** No indicator (self-only buff or placeholder). */
  | "self"
  /** Ring around the player at radius=range (melee-targeted). */
  | "circle"
  /** Filled disc around the player (AoE around self). */
  | "aoe_around_self"
  /** Directional ray from player toward cursor, capped at range. */
  | "skillshot";

/**
 * Single source of truth for derived targeting kind. Any UI that needs to
 * know "ring vs line vs disc vs nothing" must call this — never duplicate
 * the rules.
 */
export function targetingKind(skill: SkillDef): SkillTargetingKind {
  if (skill.range <= 0) return "self";
  if (skill.effect === "self_buff" || skill.effect === "placeholder") return "self";
  if (skill.effect === "aoe_around") return "aoe_around_self";
  if (skill.effect === "teleport") return "skillshot";
  if (skill.effect === "direct_damage" || skill.effect === "damage_over_time") {
    const isRanged =
      skill.tags.includes("ranged") ||
      (skill.tags.includes("spell") && !skill.tags.includes("melee"));
    return isRanged ? "skillshot" : "circle";
  }
  return "self";
}
