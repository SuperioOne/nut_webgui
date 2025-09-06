/**
 * Main entry point for client
 * Exports all web components, registers dom and htmx events.
 */

/**
 * @typedef HtmxSendError
 * @property {HTMLElement} elt
 * @property {HTMLElement} target
 * @property {any} requestConfig
 * @property {XMLHttpRequest} xhr
 */

/**
 * @typedef HtmxAfterRequest
 * @property {HTMLElement} elt
 * @property {HTMLElement} target
 * @property {any} requestConfig
 * @property {XMLHttpRequest} xhr
 * @property {boolean} successful
 */

/** @typedef {CustomEvent<HtmxSendError>} HtmxSendErrorEvent **/
/** @typedef {CustomEvent<HtmxAfterRequest>} HtmxAfterRequestEvent **/

import htmx from "htmx.org";
import { Idiomorph } from "idiomorph/dist/idiomorph.esm.js";

import "./components/bitflag_input.js";
import "./components/duration_input.js";
import "./components/confirmation_button.js";
import "./components/confirmation_modal.js";
import "./components/gauge.js";
import "./components/search_list.js";
import "./components/theme_selector.js";
import "./components/time_display.js";
import "./components/ttl_element.js";
import "./components/clipboard_button.js";

/**
 * @param {string} attr_name
 * @param {Element} node
 * @param {"updated" | "removed"} mutation_type
 * @returns {boolean}
 */
function attr_preserve(attr_name, node, mutation_type) {
  const preserve = node.getAttribute("morph-preserve-attr");

  if (preserve) {
    const target_attrs = preserve.split(" ");
    return !(target_attrs.findIndex((e) => e === attr_name) > -1);
  } else {
    return true;
  }
}

/**
 * @param {string} swapStyle
 */
function create_morph_config(swapStyle) {
  let config;

  if (swapStyle === "morph" || swapStyle === "morph:outerHTML") {
    config = { morphStyle: "outerHTML" };
  } else if (swapStyle === "morph:innerHTML") {
    config = { morphStyle: "innerHTML" };
  } else if (swapStyle.startsWith("morph:")) {
    config = Function("return (" + swapStyle.slice(6) + ")")();
  }

  if (config) {
    config.callbacks = { beforeAttributeUpdated: attr_preserve };
    config.restoreFocus = true;
  }

  return config;
}

htmx.defineExtension("morph", {
  isInlineSwap: function (swapStyle) {
    const config = create_morph_config(swapStyle);
    return config?.morphStyle === "outerHTML" || config?.morphStyle === null;
  },
  handleSwap: function (swapStyle, target, fragment) {
    const config = create_morph_config(swapStyle);

    if (config) {
      return Idiomorph.morph(
        target,
        /** @type {Element} **/ (fragment).children,
        config,
      );
    }
  },
});

const ConnectionState = (() => {
  /** @type{boolean} **/
  let state = false;

  return {
    get is_lost() {
      return state;
    },
    set is_lost(value) {
      state = value;
    },
  };
})();

const HTMX_INDICATOR_QUERY = ".htmx-send-error-indicator";
const HTMX_INDICATOR_CLASSNAME = "htmx-send-error-active";

document.body.addEventListener(
  "htmx:sendError",
  (/** @type{HtmxSendErrorEvent} **/ ev) => {
    if (!ConnectionState.is_lost) {
      ConnectionState.is_lost = true;

      const indicators = document.querySelectorAll(HTMX_INDICATOR_QUERY);

      console.warn(
        "Poll send request is failed, check your connection.",
        ev.detail.xhr,
      );

      for (const element of indicators) {
        element.classList.add(HTMX_INDICATOR_CLASSNAME);
      }
    }
  },
);

document.body.addEventListener(
  "htmx:afterRequest",
  (/** @type{HtmxAfterRequestEvent} **/ ev) => {
    if (ConnectionState.is_lost) {
      const indicators = document.querySelectorAll(HTMX_INDICATOR_QUERY);

      for (const element of indicators) {
        element.classList.remove(HTMX_INDICATOR_CLASSNAME);
      }

      ConnectionState.is_lost = false;
    }
  },
);
