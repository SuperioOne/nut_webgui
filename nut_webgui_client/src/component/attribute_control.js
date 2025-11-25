const INPUT_QUERY = "input[name], select[name], textarea[name]";
const RESET_BUTTON_QUERY = "button[type=reset]";

/** @type {"for"} Attributes */

export default class AttributeControl extends HTMLElement {
  /** @type{AbortController | undefined} */
  #abort_controller;

  /** @type{MutationObserver | undefined} */
  #mutation_observer;

  constructor() {
    super();
  }

  connectedCallback() {
    const target_id = this.getAttribute("for");

    if (!target_id) {
      return;
    }

    const target = document.getElementById(target_id);

    if (!target) {
      console.warn(
        `attr-control: no matching element found for "${target_id}"`,
      );
      return;
    }

    const input_elements = this.querySelectorAll(INPUT_QUERY);

    this.#abort_controller?.abort();
    this.#abort_controller = new AbortController();

    for (const element of input_elements) {
      element.addEventListener(
        "change",
        (e) => {
          const input =
            /**@type{HTMLInputElement | HTMLSelectElement} */
            (e.target);

          target.setAttribute(input.name, input.value);
        },
        {
          signal: this.#abort_controller.signal,
        },
      );
    }

    for (const button of this.querySelectorAll(RESET_BUTTON_QUERY)) {
      button.addEventListener("click", () => this.reset(), {
        signal: this.#abort_controller.signal,
      });
    }

    this.#mutation_observer = new MutationObserver(this.#mutation_callback);
    this.#mutation_observer.observe(target, {
      attributes: true,
      subtree: false,
      attributeOldValue: true,
      childList: false,
      characterData: false,
      characterDataOldValue: false,
      attributeFilter: [...input_elements].map((e) => e.name),
    });
  }

  reset() {
    for (const element of this.querySelectorAll(INPUT_QUERY)) {
      switch (element.nodeName) {
        case "SELECT": {
          const select = /** @type { HTMLSelectElement}*/ (element);

          for (const opt of select.options) {
            opt.selected = opt.defaultSelected;
          }

          select.dispatchEvent(new Event("change", { bubbles: true }));
          break;
        }
        case "INPUT": {
          const input = /** @type { HTMLInputElement}*/ (element);

          input.checked = input.defaultChecked;
          input.value = input.defaultValue;
          input.dispatchEvent(new Event("change", { bubbles: true }));
          break;
        }
        case "TEXTAREA": {
          const textarea = /** @type { HTMLTextAreaElement}*/ (element);
          textarea.value = textarea.defaultValue;
          textarea.dispatchEvent(new Event("change", { bubbles: true }));
          break;
        }
        default:
          break;
      }
    }
  }

  disconnectedCallback() {
    this.#abort_controller?.abort();
    this.#mutation_observer?.disconnect();
  }

  /**
   * @param {MutationRecord[]} mutations
   */
  #mutation_callback = (mutations) => {
    for (const entry of mutations) {
      const entry_target = /**  @type {Element}*/ (entry.target);
      if (entry.type === "attributes" && entry.attributeName) {
        const value = entry_target.getAttribute(entry.attributeName);

        if (entry.oldValue === value) {
          continue;
        }

        const inputs = this.querySelectorAll(
          `input[name=${entry.attributeName}], select[name=${entry.attributeName}], textarea[name=${entry.attributeName}]`,
        );

        for (const element of inputs) {
          switch (element.nodeName) {
            case "SELECT": {
              const select = /** @type { HTMLSelectElement}*/ (element);

              for (const opt of select.options) {
                opt.selected = opt.value === value;
              }

              select.dispatchEvent(new Event("change", { bubbles: true }));
              break;
            }
            case "INPUT": {
              const input = /** @type { HTMLInputElement}*/ (element);

              if (input.type === "checkbox" || input.type === "radio") {
                input.checked = input.value === value;
              } else {
                input.value = value ?? "";
              }

              input.dispatchEvent(new Event("change", { bubbles: true }));
              break;
            }
            case "TEXTAREA": {
              const textarea = /** @type { HTMLTextAreaElement}*/ (element);
              textarea.value = value ?? "";
              textarea.dispatchEvent(new Event("change", { bubbles: true }));
              break;
            }
            default:
              break;
          }
        }
      }
    }
  };
}

customElements.define("nut-attr-control", AttributeControl);
