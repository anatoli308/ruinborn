import { useCallback, useEffect, useRef } from "react";
import { useGameStore } from "./store/gameStore";
import MainMenu from "./components/MainMenu";
import GameWorld from "./components/GameWorld";
import GameTicker from "./components/GameTicker";
import HUD from "./components/ui/HUD";
import TradePanel from "./components/ui/TradePanel";
import Inventory from "./components/ui/Inventory";
import Minimap from "./components/ui/Minimap";
import ActionBar from "./components/ui/ActionBar";
import BagBar from "./components/ui/BagBar";
import GameMenu from "./components/ui/GameMenu";
import CharacterView from "./components/ui/CharacterView";
import InventoryWindow from "./components/ui/InventoryWindow";
import WaypointTravel from "./components/ui/WaypointTravel";
import PortraitBar from "./components/ui/PortraitBar";
import ClassSelectModal from "./components/ui/ClassSelectModal";
import SkillTreePanel from "./components/ui/SkillTreePanel";
import Toast from "./components/ui/Toast";

export default function App() {
  const connected = useGameStore((s) => s.connected);
  const joining = useGameStore((s) => s.joining);
  const initConnection = useGameStore((s) => s.initConnection);
  const fpsAnchorRef = useRef<HTMLDivElement>(null);

  const handleJoin = useCallback(
    (playerName: string) => {
      initConnection(playerName);
    },
    [initConnection]
  );

  // Global ESC handler: close any open overlay so every menu is dismissable.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.code !== "Escape") return;
      const target = e.target as HTMLElement | null;
      if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)) {
        return;
      }
      const s = useGameStore.getState();
      // Priority order: only close the topmost overlay.
      if (s.waypointMenuOpen) {
        s.setWaypointMenuOpen(false);
        e.preventDefault();
        return;
      }
      if (s.skillTreeOpen) {
        s.setSkillTreeOpen(false);
        e.preventDefault();
        return;
      }
      if (s.characterOpen) {
        s.setCharacterOpen(false);
        e.preventDefault();
        return;
      }
      if (s.inventoryOpen) {
        s.setInventoryOpen(false);
        e.preventDefault();
        return;
      }
      if (s.showTradePanel) {
        s.sendToggleTradePanel();
        e.preventDefault();
        return;
      }
      // Fall through: clear current target enemy.
      if (s.targetEnemyId) {
        s.setTargetEnemy(null);
        e.preventDefault();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Show menu until connected
  if (!connected && !joining) {
    return <MainMenu onJoin={handleJoin} />;
  }

  return (
    <div className="game-container">
      {/* Server state listener */}
      <GameTicker />

      {/* 3D World (full screen) */}
      <GameWorld fpsAnchorRef={fpsAnchorRef} />

      {/* HTML Overlay UI */}
      <div className="hud-layer">
        <HUD />
        <PortraitBar />
        <Inventory />
        <Minimap />
        <TradePanel />
        <ActionBar />
        <BagBar />
        <GameMenu />
        <WaypointTravel />
        <InventoryWindow />
        <CharacterView />
        <SkillTreePanel />
        <ClassSelectModal />
        <Toast />
      </div>

      {/* FPS counter anchor — drei <Stats> mounts its DOM here. */}
      <div className="fps-anchor" ref={fpsAnchorRef} />
    </div>
  );
}
