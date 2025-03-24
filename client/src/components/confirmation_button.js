import ConfirmationModal from "./confirmation_modal.js";

/** @typedef {"cancel-text" | "confirm-text" | "message" | "target-event" | "title" } AttributeKeys */

export default class ConfirmationButton extends HTMLButtonElement {
  /** @type {AbortController} */
  #abort_controller;

  /** @type {string | null} */
  #title;

  /** @type {string | null} */
  #cancel_text;

  /** @type {string | null} */
  #confirm_text;

  /** @type {string | null} */
  #target_event;

  /** @type {string | null} */
  #message;

  /** @type {AttributeKeys[]} */
  static observedAttributes = [
    "cancel-text",
    "confirm-text",
    "message",
    "target-event",
    "title",
  ];

  constructor() {
    super();
  }

  disconnectedCallback() {
    this.#abort_controller?.abort();
    this.remove();
  }

  connectedCallback() {
    this.#abort_controller = new AbortController();
    this.#cancel_text = this.getAttribute("cancel-text");
    this.#confirm_text = this.getAttribute("confirm-text");
    this.#message = this.getAttribute("message");
    this.#target_event = this.getAttribute("target-event");
    this.#title = this.getAttribute("title");
    this.role = "button";

    this.addEventListener(
      "click",
      () => {
        ConfirmationModal.create({
          message: this.#message,
          title: this.#title,
          confirmText: this.#confirm_text,
          cancelText: this.#cancel_text,
        })
          .then((is_confirmed) => {
            if (
              is_confirmed &&
              this.#target_event &&
              this.#target_event.length > 0
            ) {
              this.dispatchEvent(new CustomEvent(this.#target_event));
            }
          })
          .catch(console.error);
      },
      { signal: this.#abort_controller.signal },
    );
  }

  /**
   * @param {AttributeKeys} name
   * @param {string | null} new_value
   */
  attributeChangedCallback(name, _, new_value) {
    switch (name) {
      case "cancel-text":
        this.#cancel_text = new_value;
        break;
      case "confirm-text":
        this.#confirm_text = new_value;
        break;
      case "message":
        this.#message = new_value;
        break;
      case "target-event":
        this.#target_event = new_value;
        break;
      case "title":
        this.#title = new_value;
        break;
      default:
        break;
    }
  }
}

customElements.define("nut-confirm-button", ConfirmationButton, {
  extends: "button",
});
