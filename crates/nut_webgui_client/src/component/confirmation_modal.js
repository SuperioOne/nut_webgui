import { link_host_styles } from "../util.js";

export default class ConfirmationModal extends HTMLElement {
  /** @type{HTMLDialogElement | undefined | null} **/
  #dialog;

  /** @type{string} **/
  #template;

  /** @type{ShadowRoot} **/
  #shadow_root;

  /**
   * @param {string | null} [template] Template element id.
   **/
  constructor(template) {
    super();
    this.#shadow_root = this.attachShadow({ mode: "open" });
    this.#template = template ?? "confirm-modal";
    link_host_styles(this.#shadow_root);
  }

  /**
   * Create a confirmation modal programmatically
   * @param {{
   *   message?: string | null;
   *   title?: string | null;
   *   confirm?: string | null;
   *   cancel?: string | null;
   *   template?: string | null;
   * }} options
   * @return {Promise<boolean>}
   */
  static create(options) {
    /** @type {ConfirmationModal} */
    const modal = new ConfirmationModal(options.template);
    document.body.appendChild(modal);

    const title = document.createElement("span");
    title.slot = "title";
    title.textContent = options.title ?? null;

    const confirm = document.createElement("span");
    confirm.slot = "confirm_text";
    confirm.textContent = options.confirm ?? null;

    const cancel = document.createElement("span");
    cancel.slot = "cancel_text";
    cancel.textContent = options.cancel ?? null;

    const message = document.createElement("span");
    message.textContent = options.message ?? null;

    modal.append(title, confirm, cancel, message);
    modal.showModal();

    return new Promise((resolve) => {
      modal.addEventListener(
        "close",
        (ev) => {
          setTimeout(() => {
            modal.remove();
          });

          resolve(
            /** @type{CustomEvent<"default" | "cancel">} **/ (ev).detail ===
              "default",
          );
        },
        { once: true },
      );
    });
  }

  disconnectedCallback() {
    this.remove();
  }

  connectedCallback() {
    const template = /** @type {HTMLTemplateElement} **/ (
      document.getElementById(this.#template)
    );
    const content = document.importNode(template.content, true);
    this.#shadow_root.appendChild(content);
    this.#dialog = this.#shadow_root?.querySelector("dialog");

    if (this.#dialog) {
      this.#dialog.addEventListener("close", () => {
        this.dispatchEvent(
          new CustomEvent("close", {
            detail: this.#dialog?.returnValue,
            bubbles: true,
            composed: true,
          }),
        );
      });
    }
  }

  showModal() {
    this.#dialog?.showModal();
  }
}

customElements.define("nut-confirm", ConfirmationModal);
Reflect.set(self, "ConfirmationModal", ConfirmationModal);
