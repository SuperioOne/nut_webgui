import { link_host_styles } from "../utils.js";

/** @typedef {"value"} TimeDisplayAttrTypes */

const MINUTE = 60;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/**
 * @param {number} divisor
 * @param {number} dividend
 * @returns {[number,number]}
 */
function div_with_remainder(dividend, divisor) {
  // Internal only, no need for divide by 0 check.
  let quotient = Math.floor(dividend / divisor);
  let remainder = dividend - quotient * divisor;

  return [quotient, remainder];
}

/**
 * @param {number} input
 * @returns {string}
 */
function get_time_str(input) {
  /** @type{string[]} **/
  let sections = [];
  let remaining_time = input;

  if (remaining_time > DAY) {
    let [days, remaining] = div_with_remainder(remaining_time, DAY);
    remaining_time = remaining;
    sections.push(`${days}d`);
  }

  if (remaining_time > HOUR) {
    let [hours, remaining] = div_with_remainder(remaining_time, HOUR);
    remaining_time = remaining;
    sections.push(`${hours}h`);
  }

  if (remaining_time > MINUTE) {
    let [minutes, remaining] = div_with_remainder(remaining_time, MINUTE);
    remaining_time = remaining;
    sections.push(`${minutes}m`);
  }

  if (remaining_time > 0) {
    sections.push(`${remaining_time}s`);
  }

  return sections.join(" ");
}

export default class TimeDisplay extends HTMLElement {
  /** @type{TimeDisplayAttrTypes[]} **/
  static observedAttributes = ["value"];

  /** @type{ShadowRoot} **/
  #root;

  constructor() {
    super();
  }

  disconnectedCallback() {
    this.remove();
  }

  connectedCallback() {
    this.innerHTML = null;

    const value = Number(this.getAttribute("value"));
    this.#root = this.attachShadow({ mode: "open" });
    link_host_styles(this.#root);

    this.generate_display(value);
  }

  /** @param {number} value  **/
  generate_display(value) {
    if (this.#root) {
      if (isNaN(value)) {
        this.#root.innerHTML = "[ERR! NaN value]";
        return;
      }

      this.#root.innerHTML = get_time_str(Math.floor(value));
    }
  }

  /**
   * @param {TimeDisplayAttrTypes} name
   * @param {null | undefined | string} _old_value
   * @param {null | undefined | string} new_value
   */
  attributeChangedCallback(name, _old_value, new_value) {
    if (name === "value") {
      this.generate_display(Number(new_value));
    }
  }
}

customElements.define("nut-time-display", TimeDisplay);
