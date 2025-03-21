/** @typedef {"ttl" | "title"  | "closeable" | "type"} AttributeKeys */

const TRIGGER_QUERY =
  ".nut-notification-trigger button, .nut-notification-trigger a, .nut-notification-trigger [role=button]";

/**
 * @param {Element} element
 * @param {"info" | "success" | "error" | "warning"} alert_type
 * @returns { void }
 */
function set_alert_classes(element, alert_type) {
  /** @type {"alert-success" | "alert-error" | "alert-warning" | "alert-info"} */
  let type_class;

  switch (alert_type) {
    case "success":
      type_class = "alert-success";
      break;
    case "error":
      type_class = "alert-error";
      break;
    case "warning":
      type_class = "alert-warning";
      break;
    case "info":
    default:
      type_class = "alert-info";
      break;
  }

  element.classList.add(
    "alert",
    "alert-vertical",
    "sm:alert-horizontal",
    type_class,
  );
}

export default class NutNotification extends HTMLElement {
  /** @type {number|undefined} */
  #timer;

  constructor() {
    super();
    this.#timer = undefined;
  }

  connectedCallback() {
    const ttl_attr = Number(this.getAttribute("ttl"));
    const type = this.getAttribute("type") ?? "info";
    const ttl = isNaN(ttl_attr) || ttl_attr < 1 ? 3000 : ttl_attr;
    const dismissElements = this.querySelectorAll(TRIGGER_QUERY);

    for (const element of dismissElements) {
      element.addEventListener("click", () => this?.remove());
    }

    set_alert_classes(this, type);
    this.role = "alert";
    this.#timer = setTimeout(() => {
      this?.remove();
    }, ttl);
  }

  disconnectedCallback() {
    if (this.#timer) {
      clearTimeout(this.#timer);
    }

    this.remove();
  }

  dismiss() {
    this.remove();
  }

  /** @param {string} id */
  static dismiss(id) {
    /** @type {UpsMonNotification} */
    const element = document.getElementById(id);
    if (element) {
      element?.dismiss();
    } else {
      console.warn(`${id} does not exists in document.`);
    }
  }
}

customElements.define("nut-notification", NutNotification);
Reflect.set(window, "NutNotification", NutNotification);
