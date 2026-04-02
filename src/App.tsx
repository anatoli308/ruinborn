import GameWorld from "./components/GameWorld";
import GameTicker from "./components/GameTicker";
import HUD from "./components/ui/HUD";
import TradePanel from "./components/ui/TradePanel";
import Inventory from "./components/ui/Inventory";
import Minimap from "./components/ui/Minimap";

export default function App() {
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
      </div>
    </div>
  );
}
