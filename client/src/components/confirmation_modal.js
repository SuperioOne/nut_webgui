import { link_host_styles } from "../utils";

const TEMPLATE = document.createElement("template");
TEMPLATE.innerHTML = `
<dialog class="modal modal-bottom sm:modal-middle">
   <div class="modal-box">
      <h3 class="font-bold text-lg">
        <slot name="title">Confirm</slot>
      </h3>
      <p class="py-4">
        <slot></slot>
      </p>
      <div class="modal-action">
        <form class="flex flex-row gap-3" method="dialog">
          <button value="cancel" class="btn">
            <slot name="cancel_text">Cancel</slot>
          </button>
          <button value="default" class="btn btn-primary">
            <slot name="confirm_text">Confirm</slot>
          </button>
        </form>
      </div>
    </div>
</dialog>`;

export default class ConfirmationModal extends HTMLElement {
  /** @type{HTMLDialogElement} **/
  #dialog;

  constructor() {
    super();
  }

  /**
   * Create a confirmation modal programmatically
   * @param {{message?: string, title?:string, confirmText?:string, cancelText?:string}} options
   * @return {Promise<boolean>}
   */
  static create(options) {
    /** @type {ConfirmationModal} */
    const modal = new ConfirmationModal();
    document.body.appendChild(modal);

    const title = document.createElement("span");
    title.slot = "title";
    title.textContent = options.title;

    const confirm = document.createElement("span");
    confirm.slot = "confirm_text";
    confirm.textContent = options.confirmText;

    const cancel = document.createElement("span");
    cancel.slot = "cancel_text";
    cancel.textContent = options.cancelText;

    const message = document.createElement("span");
    message.textContent = options.message;

    modal.append(title, confirm, cancel, message);
    modal.showModal();

    return new Promise((resolve) => {
      modal.addEventListener(
        "close",
        (/** @type{CustomEvent<"default" | "cancel">} **/ ev) => {
          setTimeout(() => {
            modal.remove();
          });

          resolve(ev.detail === "default");
        },
        { once: true },
      );
    });
  }

  disconnectedCallback() {
    this.remove();
  }

  connectedCallback() {
    const shadow_root = this.attachShadow({ mode: "open" });
    link_host_styles(shadow_root);
    shadow_root.appendChild(TEMPLATE.content.cloneNode(true));

    this.#dialog = this.shadowRoot.querySelector("dialog");
    this.#dialog.addEventListener("close", () => {
      this.dispatchEvent(
        new CustomEvent("close", {
          detail: this.#dialog.returnValue,
          bubbles: true,
          composed: true,
        }),
      );
    });
  }

  showModal() {
    this.#dialog.showModal();
  }
}

customElements.define("nut-confirm", ConfirmationModal);
Reflect.set(self, "ConfirmationModal", ConfirmationModal);
