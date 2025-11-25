import { getAttributeNumeric } from "../util.js";

const TRIGGER_QUERY =
  ".ttl-trigger button, .ttl-trigger a, .ttl-trigger [role=button]";

export default class TtlElement extends HTMLElement {
  /** @type {number|undefined} */
  #timer;

  constructor() {
    super();
    this.#timer = undefined;
  }

  connectedCallback() {
    const ttl = getAttributeNumeric(this, "ttl") ?? -1;
    const dismiss_elements = this.querySelectorAll(TRIGGER_QUERY);

    for (const element of dismiss_elements) {
      element.addEventListener("click", () => this?.remove());
    }

    this.role = "alert";

    if (ttl > 0) {
      this.#timer = setTimeout(() => {
        this?.remove();
      }, ttl);
    }
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
}

customElements.define("nut-ttl", TtlElement);
