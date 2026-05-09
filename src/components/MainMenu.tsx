import { useState, useCallback } from "react";

interface MainMenuProps {
  onJoin: (playerName: string) => void;
}

export default function MainMenu({ onJoin }: MainMenuProps) {
  const [name, setName] = useState("");
  const [error, setError] = useState("");

  const handleJoin = useCallback(() => {
    const trimmed = name.trim();
    if (trimmed.length < 2) {
      setError("Name must be at least 2 characters.");
      return;
    }
    if (trimmed.length > 20) {
      setError("Name must be 20 characters or less.");
      return;
    }
    setError("");
    onJoin(trimmed);
  }, [name, onJoin]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") handleJoin();
    },
    [handleJoin]
  );

  return (
    <div className="menu-backdrop">
      <div className="menu-card">
        <h1 className="menu-title">Ruinborn</h1>
        <p className="menu-subtitle">Economy &middot; Strategy &middot; Trade</p>

        <div className="menu-field">
          <label className="menu-label" htmlFor="player-name">
            Player Name
          </label>
          <input
            id="player-name"
            className="menu-input"
            type="text"
            maxLength={20}
            placeholder="Enter your name..."
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={handleKeyDown}
          />
          {error && <span className="menu-error">{error}</span>}
        </div>

        <button
          className="menu-btn"
          disabled={name.trim().length < 2}
          onClick={handleJoin}
        >
          Join Server
        </button>

        <p className="menu-footer">ws://localhost:9000</p>
      </div>
    </div>
  );
}
