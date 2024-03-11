/**
 * @typedef {"ttl" | "title"  | "closeable" | "type"} AttributeKeys
 */

export default class UpsMonNotification extends HTMLElement {
  /** @type {number|undefined} */
  #timer;

  constructor() {
    super();
    this.#timer = undefined
  }

  connectedCallback() {
    const ttl_value = Number(this.getAttribute("ttl"));
    const type = this.getAttribute("type") ?? "info";
    const ttl = isNaN(ttl_value) || ttl_value < 1 ? 3000 : ttl_value;
    this.className = `${this.className} alert alert-${type}`;

    this.#timer = setTimeout(() => {
      this?.remove();
    }, ttl);
  }

  disconnectedCallback() {
    if (this.#timer) {
      clearTimeout(this.#timer);
    }

    this.remove();
  }

  dismiss() {
    this.remove();
  }

  /** @type {string} */
  static dismiss(id) {
    /** @type {UpsMonNotification} */
    const element = document.getElementById(id);
    if (element) {
      element?.dismiss();
    } else {
      console.warn(`${id} does not exists in document.`);
    }
  }
}

customElements.define("upsmon-notification", UpsMonNotification);
Reflect.set(window, "UpsMonNotification", UpsMonNotification)