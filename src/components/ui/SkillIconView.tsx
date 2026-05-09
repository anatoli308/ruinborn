/**
 * Renders a skill / item icon. Accepts both:
 *   - URL strings (Vite asset imports — typically end in `.png`/`.jpg` or contain `/`)
 *   - Plain emoji strings (single grapheme like "🔥")
 *
 * The heuristic: if the value looks like a path or data URL, render an `<img>`.
 * Otherwise render the text as-is so the existing emoji fallbacks keep working.
 */
export function isImageIcon(icon: string): boolean {
  if (!icon) return false;
  return (
    icon.startsWith("/") ||
    icon.startsWith("./") ||
    icon.startsWith("data:") ||
    icon.startsWith("blob:") ||
    icon.startsWith("http") ||
    /\.(png|jpe?g|gif|webp|svg)(\?.*)?$/i.test(icon)
  );
}

interface Props {
  icon: string;
  alt?: string;
  /** Optional className passed straight through. */
  className?: string;
  /** Forwarded to the wrapping element. */
  title?: string;
}

export default function SkillIconView({ icon, alt, className, title }: Props) {
  if (isImageIcon(icon)) {
    return (
      <img
        src={icon}
        alt={alt ?? ""}
        className={className ? `${className} skill-icon-img` : "skill-icon-img"}
        title={title}
        draggable={false}
      />
    );
  }
  return (
    <span className={className} title={title} aria-hidden>
      {icon}
    </span>
  );
}
