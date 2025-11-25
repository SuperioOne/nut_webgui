/**
 * @param {ShadowRoot} root
 * @returns {void}
 */
export function link_host_styles(root) {
  for (const sheet of document.styleSheets) {
    if (!sheet.href) continue;

    const link = document.createElement("link");

    link.rel = "stylesheet";
    link.href = sheet.href;
    root.prepend(link);
  }
}

/**
 * @typedef DebounceControllerOptions
 * @property {number} [duration] Debounce duration in ms
 * @property {AbortSignal} [signal] Additional abort signal
 */

/** @type{DebounceControllerOptions} */
const DEFAULT_DEBOUNCE_OPTS = {
  duration: 200,
};

/**
 * @template {any[]} A
 * @template {CallableFunction } T
 */
class DebouncerController {
  /** @type {number | undefined} */
  #timer_id;

  /** @type {DebounceControllerOptions} */
  #options;

  /** @type {T} */
  #callback;

  /**
   * @param {T} callback
   * @param {DebounceControllerOptions} [opts]
   */
  constructor(callback, opts) {
    this.#callback = callback;
    this.#options = opts
      ? { ...DEFAULT_DEBOUNCE_OPTS, ...opts }
      : DEFAULT_DEBOUNCE_OPTS;

    this.#options.signal?.addEventListener("abort", () => this.abort(), {
      once: true,
    });
  }

  /**
   * Resets any queued timer and creates new timer with the args.
   * @param {A} args
   */
  set(args) {
    this.abort();
    this.#timer_id = setTimeout(
      this.#callback,
      this.#options.duration,
      ...args,
    );
  }

  /**
   * Aborts any awaiting timer.
   */
  abort() {
    if (this.#timer_id !== undefined) {
      clearTimeout(this.#timer_id);
      this.#timer_id = undefined;
    }
  }
}

/**
 * Wraps function with a debouncer controller.
 *
 * @template {CallableFunction} T
 * @param {T} target Callback function
 * @param {DebounceControllerOptions} [options] Debouncer options
 * @returns {T} Proxy function
 */
export function into_debounced_fn(target, options) {
  const controller = new DebouncerController(target, options);

  return /** @type {any} */ (
    (/** @type {unknown[]} */ ...args) => {
      controller.set(args);
    }
  );
}

/**
 * Determines whether two "nullable" strings are equivalent in the current or specified locale.
 *
 * @param {string | null | undefined} a
 * @param {string | null | undefined} b
 * @return {number}
 */
export function localCompareStr(a, b) {
  if (!a) {
    return -1;
  } else if (!b) {
    return 1;
  } else {
    return a.localeCompare(b);
  }
}

/**
 * Reads element attribute as number.
 *
 * @param {Element} element
 * @param {string} attribute
 * @return {number | undefined}
 */
export function getAttributeNumeric(element, attribute) {
  const text_value = element.getAttribute(attribute)?.trim();

  if (text_value && text_value.length > 0) {
    const value = Number(text_value);

    return isNaN(value) ? undefined : value;
  } else {
    return undefined;
  }
}
