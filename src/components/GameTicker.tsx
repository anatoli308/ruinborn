import { useEffect } from "react";
import { useGameStore } from "../store/gameStore";

/** Initializes the connection to the Rust server tick loop */
export default function GameTicker() {
  const initListener = useGameStore((s) => s.initListener);

  useEffect(() => {
    initListener();
  }, [initListener]);

  return null;
}
