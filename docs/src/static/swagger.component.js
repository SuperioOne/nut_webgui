class NutSwagger extends HTMLElement {
  shadow_root;
  theme_observer;

  constructor() {
    super();
  }

  static observedAttributes = ["spec-url"];

  connectedCallback() {
    const target = document.querySelector("html");
    const callback = (mutationList) => {
      for (const mutation of mutationList) {
        if (
          mutation.type === "attributes" &&
          mutation.attributeName === "class"
        ) {
          const scheme =
            getComputedStyle(this).getPropertyValue("--color-scheme");

          if (scheme === "dark") {
            this.classList.add("dark-mode");
          } else {
            this.classList.remove("dark-mode");
          }

          return;
        }
      }
    };

    this.theme_observer = new MutationObserver(callback);
    this.theme_observer.observe(target, {
      attributes: true,
      childList: false,
      subtree: false,
    });
  }

  disconnectedCallback() {
    this.theme_observer.disconnect();
  }

  update() {
    const spec = this.getAttribute("spec-url");

    SwaggerUIBundle({
      url: spec,
      domNode: this,
      layout: "BaseLayout",
    });
  }

  attributeChangedCallback(name, old_value, new_value) {
    if (old_value === new_value) {
      return;
    }

    if (name === "spec-url") {
      this.update();
    }
  }
}

customElements.define("nut-swagger", NutSwagger);
