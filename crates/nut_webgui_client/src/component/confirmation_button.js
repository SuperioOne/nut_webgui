import ConfirmationModal from "./confirmation_modal.js";

export default class ConfirmationButton extends HTMLElement {
  /** @type {AbortController | undefined} */
  #abort_controller;

  /** @type {ElementInternals} */
  #internals;

  static formAssociated = true;

  constructor() {
    super();
    this.#internals = this.attachInternals();
    this.#internals.role = "button";
  }

  disconnectedCallback() {
    this.#internals.setFormValue(null);
    this.#abort_controller?.abort();
  }

  connectedCallback() {
    const val = this.getAttribute("value");
    this.#internals.setFormValue(val);
    this.#abort_controller = new AbortController();
    this.role = "button";

    const cancel = this.getAttribute("cancel-text");
    const confirm = this.getAttribute("confirm-text");
    const message = this.getAttribute("message");
    const target_event = this.getAttribute("target-event");
    const template = this.getAttribute("template");
    const title = this.getAttribute("title");

    this.addEventListener(
      "click",
      () => {
        ConfirmationModal.create({
          cancel,
          confirm,
          message,
          template,
          title,
        })
          .then((is_confirmed) => {
            if (is_confirmed && target_event && target_event.length > 0) {
              this.dispatchEvent(new CustomEvent(target_event));
            }
          })
          .catch(console.error);
      },
      { signal: this.#abort_controller.signal },
    );
  }
}

customElements.define("nut-confirm-button", ConfirmationButton);
