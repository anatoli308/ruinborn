import { useEffect } from "react";
import { useGameStore } from "../store/gameStore";

/** Initializes the WebSocket connection to the dedicated server */
export default function GameTicker() {
  const initConnection = useGameStore((s) => s.initConnection);

  useEffect(() => {
    initConnection("Spieler");
  }, [initConnection]);

  return null;
}
