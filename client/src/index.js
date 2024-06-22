/**
 * Main entry point for client
 * Exports all web components, registers dom load events.
 **/

import htmx from "htmx.org/dist/htmx.esm.js";
import { Idiomorph } from "idiomorph/dist/idiomorph.esm.js";

export * from "./components/charts/gauge.js";
export * from "./components/notification.js";
export * from "./components/confirmation_modal.js";
export * from "./components/theme_selector.js";

function createMorphConfig(swapStyle) {
  if (swapStyle === "morph" || swapStyle === "morph:outerHTML") {
    return { morphStyle: "outerHTML" };
  } else if (swapStyle === "morph:innerHTML") {
    return { morphStyle: "innerHTML" };
  } else if (swapStyle.startsWith("morph:")) {
    return Function("return (" + swapStyle.slice(6) + ")")();
  }
}

htmx.defineExtension("morph", {
  isInlineSwap: function (swapStyle) {
    let config = createMorphConfig(swapStyle);
    return config.swapStyle === "outerHTML" || config.swapStyle == null;
  },
  handleSwap: function (swapStyle, target, fragment) {
    let config = createMorphConfig(swapStyle);
    if (config) {
      return Idiomorph.morph(target, fragment.children, config);
    }
  },
});

// document.body.addEventListener(
//   "htmx:sendError",
//   (
//     /** @type{CustomEvent<{elt: Element; target: Element; requestConfig: any; xhr: XMLHttpRequest}>} **/ details,
//   ) => {
//     console.debug(details.detail);
//   },
// );
