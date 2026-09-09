import { getAttributeNumeric, link_host_styles } from "../util.js";

/** @typedef {"value" | "step" | "class" } GaugeAtrributes */

export default class BarGauge extends HTMLElement {
  /** @type {ShadowRoot} */
  #shadow_root;

  /** @type{Element | undefined} */
  #container;

  /** @type {GaugeAtrributes[]} */
  static observedAttributes = ["value", "class"];

  constructor() {
    super();
    this.#shadow_root = this.attachShadow({ mode: "closed" });
    link_host_styles(this.#shadow_root);
  }

  connectedCallback() {
    const value = getAttributeNumeric(this, "value") ?? 0;
    const bar_gauge = document.createElement("div");
    const bar_gauge_fill = document.createElement("div");
    bar_gauge.classList.add("bar-gauge", ...this.classList);
    bar_gauge_fill.classList.add("bar-gauge-fill");
    bar_gauge.replaceChildren(bar_gauge_fill);

    this.#update_value(bar_gauge, value);
    this.#container = bar_gauge;
    this.#shadow_root.append(bar_gauge);
  }

  /**
   * @param {Element} target
   * @param {number} value
   */
  #update_value(target, value) {
    const height = Math.max(0, Math.min(100, value));
    /**@type {HTMLElement | null}*/
    const fill = target.querySelector(".bar-gauge-fill");

    if (fill) {
      fill.style.height = `${height}%`;
    }
  }

  /**
   * @param {GaugeAtrributes} name
   * @param {string} _old_value
   * @param {string} new_value
   */
  attributeChangedCallback(name, _old_value, new_value) {
    const target = this.#container;

    if (!target) {
      return;
    }

    switch (name) {
      case "value": {
        const value = Number(new_value);

        if (isNaN(value)) {
          console.warn("bar gauge: cannot change value, not a number");
        } else {
          this.#update_value(target, value);
        }
        break;
      }
      case "class": {
        target.className = this.className;
        target.classList.add("bar-gauge");
      }
      default:
        break;
    }
  }
}

customElements.define("nut-bar-gauge", BarGauge);
