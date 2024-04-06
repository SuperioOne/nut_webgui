/**
 * Main entry point for client
 * Exports all web components, registers dom load events.
 **/

import "htmx.org";
import { load_local_theme } from "./components/theme_selector.js";

export * from "./components/charts/chart-gauge.js";
export * from "./components/notification.js";
export * from "./components/confirmation_modal.js";
export * from "./components/theme_selector.js";

window.addEventListener(
  "DOMContentLoaded",
  () => {
    load_local_theme();
  },
  {
    once: true,
  },
);
