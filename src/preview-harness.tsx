import ReactDOM from "react-dom/client";

import { KeyboardPreview } from "./features/lighting/KeyboardPreview";
import type { KeyboardState } from "./domain/lighting";
import "./styles/theme.css";
import "./styles/globals.css";
import "./styles/layout.css";

const state: KeyboardState = {
  effect: "Static",
  primary_color: { r: 60, g: 140, b: 255 },
  secondary_color: null,
  brightness: 80,
  speed: 1,
  direction: "LeftToRight",
  enabled: true,
  zone_colors: null,
};

window.addEventListener("error", (event) => {
  const pre = document.createElement("pre");
  pre.style.cssText = "color:#ff6666;font-size:16px;white-space:pre-wrap";
  pre.textContent = `ERROR: ${event.message}\n${event.error?.stack ?? ""}`;
  document.body.prepend(pre);
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <KeyboardPreview state={state} />,
);
