export default class ClipboardButton extends HTMLElement {
  /** @type {AbortController | undefined} */
  #abort_controller;

  constructor() {
    super();
  }

  connectedCallback() {
    this.role = "button";
    this.#abort_controller?.abort();

    const data = this.getAttribute("content");

    if (data) {
      this.#abort_controller = new AbortController();
      this.addEventListener(
        "click",
        () => {
          navigator.clipboard.writeText(data);

          this.dispatchEvent(
            new CustomEvent("clipboard", {
              composed: true,
              detail: data,
              cancelable: false,
            }),
          );
        },
        {
          signal: this.#abort_controller.signal,
        },
      );
    }
  }

  disconnectedCallback() {
    this.#abort_controller?.abort();
  }
}

customElements.define("nut-clipboard-button", ClipboardButton);
