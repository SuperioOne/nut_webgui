/**
 * @typedef {"theme-key"} ThemeSelectorAttributes
 */

/**
 * @typedef {CustomEvent<{theme: string}>} ThemeEvent
 */

const LOCAL_STORAGE_KEY = "app_theme";
const DATA_THEME_KEY = "data-theme";

export class ThemeSelector extends HTMLElement {
  /** @type{ThemeSelectorAttributes[]} **/
  static observedAttributes = ["theme-key"];
  /** @type{string | undefined | null} **/
  #theme_value;

  constructor() {
    super();
  }

  connectedCallback() {
    this.role = "button";
    this.addEventListener("click", this.update_theme);
    this.#theme_value = this.getAttribute("theme-key") ?? "default";
  }

  disconnectedCallback() {
    this.removeEventListener("click", this.update_theme);
  }

  /**
   * @param {ThemeSelectorAttributes} name
   * @param {string | null} old_value
   * @param {string | null} new_value
   * @returns {void}
   */
  attributeChangedCallback(name, old_value, new_value) {
    switch (name) {
      case "theme-key": {
        this.#theme_value = new_value;
        break;
      }
      default: {
        console.warn(`Unhandled attribute change ${name}`, this);
        break;
      }
    }
  }

  update_theme = () => {
    if (this.#theme_value) {
      localStorage.setItem(LOCAL_STORAGE_KEY, this.#theme_value);
      document.documentElement.setAttribute(DATA_THEME_KEY, this.#theme_value);
      document.dispatchEvent(
        new CustomEvent("theme-change", {
          detail: { theme: this.#theme_value },
          cancelable: false,
        }),
      );
    } else {
      console.warn("No theme value attached to the theme selector", this);
    }
  };
}

customElements.define("theme-selector", ThemeSelector);
