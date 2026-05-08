import { useCallback } from "react";
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
import CharacterView from "./components/ui/CharacterView";
import InventoryWindow from "./components/ui/InventoryWindow";
import MouseSkillBar from "./components/ui/MouseSkillBar";
import WaypointTravel from "./components/ui/WaypointTravel";
import ClassSelectModal from "./components/ui/ClassSelectModal";
import SkillTreePanel from "./components/ui/SkillTreePanel";

export default function App() {
  const connected = useGameStore((s) => s.connected);
  const joining = useGameStore((s) => s.joining);
  const initConnection = useGameStore((s) => s.initConnection);

  const handleJoin = useCallback(
    (playerName: string) => {
      initConnection(playerName);
    },
    [initConnection]
  );

  // Show menu until connected
  if (!connected && !joining) {
    return <MainMenu onJoin={handleJoin} />;
  }

  return (
    <div className="game-container">
      {/* Server state listener */}
      <GameTicker />

      {/* 3D World (full screen) */}
      <GameWorld />

      {/* HTML Overlay UI */}
      <div className="hud-layer">
        <HUD />
        <Inventory />
        <Minimap />
        <TradePanel />
        <ActionBar />
        <BagBar />
        <MouseSkillBar />
        <WaypointTravel />
        <InventoryWindow />
        <CharacterView />
        <SkillTreePanel />
        <ClassSelectModal />
      </div>
    </div>
  );
}
