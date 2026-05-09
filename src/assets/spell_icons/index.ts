// Static imports so Vite hashes and bundles the PNGs.
// Each import resolves to a URL string at build time.
import autoAttack from "./auto_attack.png";
import autoShoot from "./auto_shoot.png";
import bash from "./bash.png";
import boneSpear from "./bone_spear.png";
import cleave from "./cleave.png";
import crusaderStrike from "./crusader_strike.png";
import fireball from "./fireball.png";
import heroicStrike from "./heroic_strike.png";
import shadowbolt from "./shadowbolt.png";
import sinisterStrike from "./sinister_strike.png";

/** Direct file lookup — keep the filename as the key for new assets. */
export const SPELL_ICON_FILES = {
  auto_attack: autoAttack,
  auto_shoot: autoShoot,
  bash,
  bone_spear: boneSpear,
  cleave,
  crusader_strike: crusaderStrike,
  fireball,
  heroic_strike: heroicStrike,
  shadowbolt,
  sinister_strike: sinisterStrike,
} as const;

/**
 * Map game skill ids to their cosmetic icon URL.
 * Skills without a matching asset stay on their emoji fallback in the catalog.
 */
export const SKILL_ID_TO_ICON: Record<string, string> = {
  bash,
  cleave,
  battle_cry: heroicStrike,
  fireball,
  bone_spear: boneSpear,
  raise_skeleton: shadowbolt,
  // frost_nova, teleport, amplify_damage → no fitting asset yet, keep emoji.
};

/** Default basic-attack icon used by the LMB slot when no skill is bound. */
export const DEFAULT_ATTACK_ICON = autoAttack;
