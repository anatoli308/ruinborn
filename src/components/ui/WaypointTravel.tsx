import { useGameStore } from "../../store/gameStore";
import type { ZoneId } from "../../types";

/** Waypoint travel panel — opened via the minimap (D2-style). */
export default function WaypointTravel() {
  const open = useGameStore((s) => s.waypointMenuOpen);
  const setOpen = useGameStore((s) => s.setWaypointMenuOpen);
  const zones = useGameStore((s) => s.zones);
  const unlockedWaypoints = useGameStore((s) => s.unlockedWaypoints);
  const currentZone = useGameStore((s) => s.zone);
  const sendTravelWaypoint = useGameStore((s) => s.sendTravelWaypoint);

  if (!open) return null;

  const unlocked = new Set<ZoneId>(unlockedWaypoints);
  const travelable = zones.filter((z) => unlocked.has(z.id));

  return (
    <div className="waypoint-panel">
      <div className="waypoint-list">
        <div className="waypoint-header">
          <span>Wegpunkte</span>
          <button
            type="button"
            className="waypoint-close"
            onClick={() => setOpen(false)}
            title="Schließen"
          >
            ✕
          </button>
        </div>
        {travelable.length === 0 && (
          <div className="waypoint-empty">Keine Wegpunkte freigeschaltet</div>
        )}
        {travelable.map((z) => {
          const isCurrent = z.id === currentZone;
          return (
            <button
              key={z.id}
              type="button"
              className="waypoint-item"
              disabled={isCurrent}
              onClick={() => {
                void sendTravelWaypoint(z.id);
                setOpen(false);
              }}
            >
              <span className="waypoint-name">{z.name}</span>
              <span className="waypoint-kind">{z.kind}</span>
              {isCurrent && <span className="waypoint-current"> · hier</span>}
            </button>
          );
        })}
      </div>
    </div>
  );
}
