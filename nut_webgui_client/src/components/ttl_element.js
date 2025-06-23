const TRIGGER_QUERY =
  ".ttl-trigger button, .ttl-trigger a, .ttl-trigger [role=button]";

export default class TTLElement extends HTMLElement {
  /** @type {number|undefined} */
  #timer;

  constructor() {
    super();
    this.#timer = undefined;
  }

  connectedCallback() {
    const ttl_attr = Number(this.getAttribute("ttl"));
    const ttl = isNaN(ttl_attr) || ttl_attr < 1 ? 3000 : ttl_attr;
    const dismissElements = this.querySelectorAll(TRIGGER_QUERY);

    for (const element of dismissElements) {
      element.addEventListener("click", () => this?.remove());
    }

    this.role = "alert";
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
}

customElements.define("nut-ttl", TTLElement);
