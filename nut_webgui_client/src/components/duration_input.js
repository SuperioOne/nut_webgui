// 24 hours in ms
const DAY_OFFSET = 86400000;

/**
 * @returns {string}
 */
function get_min_date() {
  return new Date(Date.now() + DAY_OFFSET).toISOString().split("T")[0];
}

export default class DurationInput extends HTMLElement {
  /** @type {ElementInternals} */
  #internals;

  /** @type{AbortController | undefined} */
  #abort_controller;

  /** @type {ShadowRoot} */
  #shadow_root;

  static formAssociated = true;

  constructor() {
    super();
    this.#internals = this.attachInternals();
    this.#internals.role = "input";
    this.#shadow_root = this.attachShadow({ mode: "closed" });
  }

  disconnectedCallback() {
    this.#abort_controller?.abort();
    this.#internals.setFormValue(null);
  }

  connectedCallback() {
    this.#abort_controller?.abort();
    this.#abort_controller = new AbortController();

    const date_input = document.createElement("input");
    date_input.type = "date";
    date_input.min = this.getAttribute("min") ?? get_min_date();
    date_input.max = this.getAttribute("max") ?? "";
    date_input.style.all = "unset";
    date_input.style.width = "100%";

    date_input.addEventListener(
      "change",
      (ev) => {
        const target = /** @type{HTMLInputElement} */ (ev.target);

        if (target && target.value) {
          const target_date = new Date(target.value);
          const now = new Date();
          const duration = target_date.getTime() - now.getTime();

          if (isNaN(duration) || duration < 0) {
            this.#internals.setFormValue(null);
            this.#internals.setValidity(
              {
                badInput: true,
              },
              `This field value cannot be less than ${date_input.min}`,
              date_input,
            );
          } else {
            this.#internals.setFormValue(`${duration}`);
            this.#internals.setValidity({});
          }
        } else if (this.hasAttribute("required")) {
          this.#internals.setFormValue(null);
          this.#internals.setValidity(
            {
              valueMissing: true,
            },
            "This field is required",
            date_input,
          );
        }
      },
      { signal: this.#abort_controller.signal },
    );

    this.#shadow_root.replaceChildren(date_input);

    if (this.hasAttribute("required")) {
      this.#internals.setValidity(
        { valueMissing: true },
        "This field is required",
        date_input,
      );
    }

    this.role = "input";
  }
}

customElements.define("nut-duration-input", DurationInput);
