export default class ConfirmationModal extends HTMLElement {
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
    const modal = document.createElement("upsmon-confirm");
    modal.setAttribute("title", options.title);
    modal.setAttribute("message", options.message);
    modal.setAttribute("ok-text", options.confirmText);
    modal.setAttribute("cancel-text", options.cancelText);
    document.body.appendChild(modal);

    modal.trigger_confirm();

    return new Promise((resolve) => {
      modal.addEventListener("close", (e) => {
        resolve(e.detail === "default");
        setTimeout(() => {
          modal.remove();
        });
      }, {once: true});
    });
  }

  /** @type {HTMLDialogElement} */
  #dialogElement;

  connectedCallback() {
    const title = this.getAttribute("title") ?? "Confirmation";
    const message = this.getAttribute("message") ?? "Are you confirm this action?";
    const confirmText = this.getAttribute("ok-text") ?? "Confirm";
    const cancelText = this.getAttribute("cancel-text") ?? "Cancel";

    this.#dialogElement = document.createElement("dialog");
    this.#dialogElement.className = "modal modal-bottom sm:modal-middle";
    this.#dialogElement.id = crypto.randomUUID();
    this.#dialogElement.innerHTML = `
     <div class="modal-box">
        <h3 class="font-bold text-lg">${title}</h3>
        <p class="py-4">${message}</p>
        <div class="modal-action">
          <form class="flex flex-row gap-3" method="dialog">
            <button value="cancel" class="btn">${cancelText}</button>
            <button value="default" class="btn btn-primary">${confirmText}</button>
          </form>
        </div>
      </div>`;

    this.#dialogElement.addEventListener("close", (e) => {
      this.dispatchEvent(new CustomEvent("close", {
        composed: true,
        bubbles: true,
        detail: this.#dialogElement.returnValue
      }));
    });

    this.appendChild(this.#dialogElement);
  }

  trigger_confirm() {
    this.#dialogElement.showModal();
  }
}

customElements.define("upsmon-confirm", ConfirmationModal);
Reflect.set(self, "ConfirmationModal", ConfirmationModal);