import { getAttributeNumeric, link_host_styles } from "../util.js";

/** @typedef {"value" | "step" | "class" } GaugeAtrributes */

export default class BarGauge extends HTMLElement {
  /** @type {ShadowRoot} */
  #shadow_root;

  /** @type {number} */
  #step = 5;

  /** @type {number} */
  #value = 5;

  /** @type{Element | undefined} */
  #container;

  /** @type {GaugeAtrributes[]} */
  static observedAttributes = ["value", "step", "class"];

  constructor() {
    super();
    this.#shadow_root = this.attachShadow({ mode: "closed" });
    link_host_styles(this.#shadow_root);
  }

  connectedCallback() {
    this.#step = getAttributeNumeric(this, "step") ?? 5;
    this.#value = getAttributeNumeric(this, "value") ?? 0;

    const container = document.createElement("div");
    container.classList.add("bar-gauge-container", ...this.classList);

    this.#render(container, this.#value);
    this.#container = container;
    this.#shadow_root.append(container);
  }

  /**
   * @param {Element} target
   * @param {number} value
   */
  #render(target, value) {
    target.replaceChildren();
    const active = Math.round(value / (100 / this.#step));

    for (let i = 0; i < this.#step; i++) {
      const step_element = document.createElement("div");
      step_element.classList.add("bar-gauge-step");

      if (i < active) {
        step_element.setAttribute("active", "true");
      }

      target.insertAdjacentElement("afterbegin", step_element);
    }
  }

  /**
   * @param {Element} target
   * @param {number} value
   */
  #update_value(target, value) {
    const active = Math.round(value / (100 / this.#step));
    let idx = this.#step;

    for (const tick_element of target.querySelectorAll(".bar-gauge-step")) {
      if (idx <= active) {
        tick_element.setAttribute("active", "true");
      } else {
        tick_element.removeAttribute("active");
      }

      idx -= 1;
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
          this.#value = value;
          this.#update_value(target, value);
        }
        break;
      }
      case "step": {
        const step = Number(new_value);

        if (isNaN(step)) {
          console.warn("bar gauge: cannot change step, not a number");
        } else {
          this.#step = step;
          this.#render(target, this.#value);
        }
      }
      case "class": {
        target.className = this.className;
        target.classList.add("bar-gauge-container");
      }
      default:
        break;
    }
  }
}

customElements.define("nut-bar-gauge", BarGauge);
