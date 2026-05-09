import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/index.css";
import "./styles/game-menu.css";

// Disable native context menu (no "save image as / copy image / inspect" in production).
window.addEventListener("contextmenu", (e) => e.preventDefault());

// Block dev-tools shortcuts in production builds.
if (!import.meta.env.DEV) {
  window.addEventListener("keydown", (e) => {
    if (e.key === "F12") {
      e.preventDefault();
      return;
    }
    if ((e.ctrlKey || e.metaKey) && e.shiftKey) {
      const k = e.key.toLowerCase();
      if (k === "i" || k === "j" || k === "c") e.preventDefault();
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "u") {
      e.preventDefault();
    }
  });
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
