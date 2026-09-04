/**
 * Main entry point for client
 * Exports all web components, registers dom and htmx events.
 */

import "htmx.org";
import "htmx.org/dist/ext/hx-preload.js";

import "./component/attribute_control.js";
import "./component/bitflag_input.js";
import "./component/clipboard_button.js";
import "./component/confirmation_button.js";
import "./component/confirmation_modal.js";
import "./component/duration_input.js";
import "./component/gauge_bar.js";
import "./component/gauge_radial.js";
import "./component/graph.js";
import "./component/locale_date.js";
import "./component/search_list.js";
import "./component/theme_selector.js";
import "./component/time_display.js";
import "./component/ttl_element.js";

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

const ConnectionState = (() => {
  const ERR_INDICATOR_QUERY = ".htmx-error-indicator";
  const ERR_INDICATOR_CLASSNAME = "htmx-error-active";

  /** @type{boolean} **/
  let is_failed = false;

  return {
    set_error: (/**@type {Error}*/ err) => {
      is_failed = true;

      for (const element of document.querySelectorAll(ERR_INDICATOR_QUERY)) {
        element.classList.add(ERR_INDICATOR_CLASSNAME);
      }
    },
    reset: () => {
      if (is_failed) {
        is_failed = false;
        for (const element of document.querySelectorAll(ERR_INDICATOR_QUERY)) {
          element.classList.remove(ERR_INDICATOR_CLASSNAME);
        }
      }
    },
  };
})();

document.body.addEventListener("htmx:error", (ev) => {
  ConnectionState.set_error(/**@type {Error}*/ (ev.detail.error));
});

document.body.addEventListener("htmx:after:request", (ev) => {
  const status = ev.detail.ctx.response?.status;

  if (status !== undefined && status < 400 && status >= 200) {
    ConnectionState.reset();
  }
});
