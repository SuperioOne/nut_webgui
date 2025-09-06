const CHECKBOX_QUERY = "input[type='checkbox'][value]:not([disabled])";

export default class BitFlagInput extends HTMLElement {
  /** @type {ElementInternals} */
  #internals;

  /** @type{AbortController | undefined} */
  #abort_controller;

  /** @type{MutationObserver | undefined} */
  #mutation_observer;

  static formAssociated = true;

  constructor() {
    super();
    this.#internals = this.attachInternals();
    this.#internals.role = "input";
    this.value = null;
  }

  disconnectedCallback() {
    this.#internals.setFormValue(null);
    this.#abort_controller?.abort();
    this.#mutation_observer?.disconnect();
  }

  connectedCallback() {
    this.#attach_checkboxes();
    this.role = "input";

    this.#mutation_observer = new MutationObserver(() => {
      this.#attach_checkboxes();
    });

    this.#mutation_observer.observe(this, {
      subtree: true,
      childList: true,
    });
  }

  #attach_checkboxes() {
    this.#abort_controller?.abort();
    this.#abort_controller = new AbortController();

    /** @type{NodeListOf<HTMLInputElement>} */
    const checkboxes = this.querySelectorAll(CHECKBOX_QUERY);
    const change_handler = () => this.#update_value();

    for (const input of checkboxes) {
      input.addEventListener("change", change_handler, {
        signal: this.#abort_controller.signal,
      });
    }

    this.#update_value();
  }

  #update_value() {
    /** @type{NodeListOf<HTMLInputElement>} */
    const checkboxes = this.querySelectorAll(CHECKBOX_QUERY);

    let bitflags = 0;
    for (const input of checkboxes) {
      const value = parseInt(input.value, 10);

      if (input.checked && !isNaN(value)) {
        bitflags |= value;
      }
    }

    this.#internals.setFormValue(`${bitflags}`);
  }
}

customElements.define("nut-bitflag-input", BitFlagInput);
