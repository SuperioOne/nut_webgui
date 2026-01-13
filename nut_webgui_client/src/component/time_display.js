import { getAttributeNumeric, link_host_styles } from "../util.js";

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

  /** @type{ShadowRoot } **/
  #shadow_root;

  constructor() {
    super();
    this.#shadow_root = this.attachShadow({ mode: "closed" });
    link_host_styles(this.#shadow_root);
  }

  connectedCallback() {
    this.#update();
  }

  #update() {
    const value = getAttributeNumeric(this, "value");

    this.#shadow_root.innerHTML =
      value === undefined
        ? "[ERR! NaN value]"
        : get_time_str(Math.floor(value));
  }

  /**
   * @param {TimeDisplayAttrTypes} name
   * @param {null | undefined | string} old_value
   * @param {null | undefined | string} new_value
   */
  attributeChangedCallback(name, old_value, new_value) {
    if (old_value === new_value) {
      return;
    }

    if (name === "value") {
      this.#update();
    }
  }
}

customElements.define("nut-time-display", TimeDisplay);
