import { getAttributeNumeric, link_host_styles } from "../util.js";

// Uses browser's own localization and timezone for formatting.
const FORMATTER = new Intl.DateTimeFormat(undefined, {
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  month: "2-digit",
  second: "2-digit",
  timeZoneName: "short",
  year: "numeric",
});

/** @typedef {"timestamp"} LocalizedDateAttributes */

export default class LocalizedDateElement extends HTMLElement {
  /** @type{ShadowRoot } **/
  #shadow_root;

  constructor() {
    super();
    this.#shadow_root = this.attachShadow({ mode: "closed" });
    link_host_styles(this.#shadow_root);
  }

  static observedAttributes = ["timestamp"];

  connectedCallback() {
    this.#update();
  }

  #update() {
    const timestamp = getAttributeNumeric(this, "timestamp");

    this.#shadow_root.innerHTML =
      timestamp === undefined ? "N/A" : FORMATTER.format(timestamp);
  }

  /**
   * @param {LocalizedDateAttributes} name
   * @param {null | undefined | string} old_value
   * @param {null | undefined | string} new_value
   */
  attributeChangedCallback(name, old_value, new_value) {
    if (old_value === new_value) {
      return;
    }

    if (name === "timestamp") {
      this.#update();
    }
  }
}

customElements.define("nut-localized-date", LocalizedDateElement);
