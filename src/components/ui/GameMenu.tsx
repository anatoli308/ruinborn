import { useEffect, useRef, useState } from "react";

/**
 * Compact menu launcher to the left of the action bar.
 * Currently exposes Settings (placeholder) and Logout. Click outside or press
 * Escape to close.
 */
export default function GameMenu() {
  const [open, setOpen] = useState(false);
  const wrapperRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onClick = (e: MouseEvent) => {
      if (!wrapperRef.current) return;
      if (!wrapperRef.current.contains(e.target as Node)) setOpen(false);
    };
    window.addEventListener("mousedown", onClick);
    return () => window.removeEventListener("mousedown", onClick);
  }, [open]);

  const handleSettings = () => {
    setOpen(false);
    window.alert("Einstellungen folgen bald.");
  };

  const handleLogout = () => {
    // Hard-reset all transport + module state by reloading the page.
    window.location.reload();
  };

  return (
    <div className="game-menu" ref={wrapperRef}>
      <button
        type="button"
        className="game-menu__toggle"
        title="Menu"
        onClick={() => setOpen((o) => !o)}
      >
        ☰
      </button>
      {open && (
        <div className="game-menu__popup" role="menu">
          <button
            type="button"
            className="game-menu__item"
            onClick={handleSettings}
            role="menuitem"
          >
            ⚙ Einstellungen
          </button>
          <button
            type="button"
            className="game-menu__item game-menu__item--danger"
            onClick={handleLogout}
            role="menuitem"
          >
            🚪 Logout
          </button>
        </div>
      )}
    </div>
  );
}
