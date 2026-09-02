import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";

import App from "./App";
import FloatingCard from "./FloatingCard";
import "./styles.css";

// Both windows load the same bundle; which component renders depends on
// which window this is (see apps/desktop/src-tauri/src/floating_card.rs).
const isFloatingCard = getCurrentWindow().label === "floating-card";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>{isFloatingCard ? <FloatingCard /> : <App />}</React.StrictMode>,
);
