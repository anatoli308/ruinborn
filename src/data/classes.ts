import type { ClassId, ClassInfo, SkillDef } from "../types";

/** Class catalog — mirror of `crates/tradewars-game/src/classes.rs::class_definition`. */
export const CLASS_CATALOG: ClassInfo[] = [
  {
    id: "barbarian",
    name: "Barbar",
    tagline: "Roher Krieger — Stärke & Vitalität",
    icon: "🪓",
    baseStats: { strength: 30, dexterity: 20, vitality: 25, energy: 10 },
    starterSkills: ["bash"],
  },
  {
    id: "sorceress",
    name: "Zauberin",
    tagline: "Elementarmagierin — Energie & Mana",
    icon: "🔮",
    baseStats: { strength: 10, dexterity: 25, vitality: 10, energy: 35 },
    starterSkills: ["fireball"],
  },
  {
    id: "necromancer",
    name: "Totenbeschwörer",
    tagline: "Knochenmagier — Geister & Flüche",
    icon: "💀",
    baseStats: { strength: 15, dexterity: 25, vitality: 15, energy: 25 },
    starterSkills: ["bone_spear"],
  },
];

export function classInfo(id: ClassId): ClassInfo {
  return CLASS_CATALOG.find((c) => c.id === id) ?? CLASS_CATALOG[0];
}

/** Skill catalog — mirror of `crates/tradewars-game/src/skills.rs::skill_catalog`. */
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
    description: "Wuchtiger Hieb gegen ein Ziel.",
  },
  {
    id: "cleave",
    name: "Spaltung",
    classId: "barbarian",
    manaCost: 6,
    cooldownTicks: 14,
    range: 3.0,
    requiresLevel: 6,
    effect: "aoe_around",
    damageType: "physical",
    tags: ["melee"],
    description: "Trifft alle Gegner im Nahbereich.",
  },
  {
    id: "battle_cry",
    name: "Kriegsruf",
    classId: "barbarian",
    manaCost: 8,
    cooldownTicks: 60,
    range: 0,
    requiresLevel: 12,
    effect: "self_buff",
    damageType: null,
    tags: [],
    description: "Brüllen — kurzfristiger Selbst-Buff.",
  },
  // Sorceress
  {
    id: "fireball",
    name: "Feuerball",
    classId: "sorceress",
    manaCost: 5,
    cooldownTicks: 8,
    range: 9.0,
    requiresLevel: 1,
    effect: "direct_damage",
    damageType: "fire",
    tags: ["spell", "ranged"],
    description: "Schleudert eine brennende Kugel.",
  },
  {
    id: "frost_nova",
    name: "Frostnova",
    classId: "sorceress",
    manaCost: 9,
    cooldownTicks: 24,
    range: 4.0,
    requiresLevel: 6,
    effect: "aoe_around",
    damageType: "cold",
    tags: ["spell"],
    description: "Eis-Explosion um die Zauberin.",
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
    description: "Springt zu einer Zielposition.",
  },
  // Necromancer
  {
    id: "bone_spear",
    name: "Knochenspeer",
    classId: "necromancer",
    manaCost: 4,
    cooldownTicks: 8,
    range: 10.0,
    requiresLevel: 1,
    effect: "direct_damage",
    damageType: "magical",
    tags: ["spell", "ranged"],
    description: "Schießt einen magischen Knochenspeer.",
  },
  {
    id: "raise_skeleton",
    name: "Skelett beschwören",
    classId: "necromancer",
    manaCost: 10,
    cooldownTicks: 30,
    range: 0,
    requiresLevel: 6,
    effect: "placeholder",
    damageType: null,
    tags: ["summoning"],
    description: "Beschwört einen Skelettkrieger (kommt in Phase 4).",
  },
  {
    id: "amplify_damage",
    name: "Giftwolke",
    classId: "necromancer",
    manaCost: 6,
    cooldownTicks: 15,
    range: 8.0,
    requiresLevel: 12,
    effect: "damage_over_time",
    damageType: "poison",
    tags: ["spell", "trap"],
    description: "Vergiftet einen Gegner über mehrere Sekunden.",
  },
];

export function skillDef(id: string): SkillDef | undefined {
  return SKILL_CATALOG.find((s) => s.id === id);
}

export function skillsForClass(id: ClassId): SkillDef[] {
  return SKILL_CATALOG.filter((s) => s.classId === id);
}
