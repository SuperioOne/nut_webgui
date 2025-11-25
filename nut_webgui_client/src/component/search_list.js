import htmx from "htmx.org";
import { into_debounced_fn, localCompareStr } from "../util.js";

/** @typedef {"for"} SearchListAttributes */

/**
 * Calculates string matching score based on local sequence alignment. (Smith-Waterman algorithm)
 *
 * @param {string} term
 * @param {string} input
 * @param { { score_weight?: number, gap_penalty?: number }? } opts
 * @returns {number}
 */
function calc_score(term, input, opts) {
  const { score_weight = 3, gap_penalty = 2 } = opts ?? {};
  const norm_term = term.toLocaleUpperCase();
  const norm_input = input.toLocaleUpperCase();
  const row_len = norm_term.length + 1;
  const col_len = norm_input.length + 1;

  /** @type {number[]} */
  const score_table = new Array(col_len * row_len).fill(0);
  let max_score = 0;

  for (let i = 1; i < row_len; i++) {
    for (let j = 1; j < col_len; j++) {
      const idx = j + i * (row_len - 1);
      const left_idx = idx - 1;
      const top_idx = idx - (row_len - 1);
      const adjacent_idx = top_idx - 1;

      const score_top = score_table[top_idx] - gap_penalty;
      const score_left = score_table[left_idx] - gap_penalty;
      const score_adjacent =
        score_table[adjacent_idx] +
        (norm_input[j - 1] === norm_term[i - 1] ? 1 : -1) * score_weight;

      const cell_score = Math.max(0, score_adjacent, score_top, score_left);
      max_score = max_score <= cell_score ? cell_score : max_score;

      score_table[idx] = cell_score;
    }
  }

  return max_score / Math.max(input.length, term.length);
}

export default class SearchList extends HTMLElement {
  /** @type {AbortController | undefined} **/
  #abort_controller;

  /** @type{MutationObserver | undefined} */
  #mutation_observer;

  /** @type {string | undefined} */
  #search_term;

  /** @type {SearchListAttributes[]} */
  static observedAttributes = ["for"];

  constructor() {
    super();
  }

  connectedCallback() {
    this.#attach_input(this.getAttribute("for"));
    this.#mutation_observer = new MutationObserver((records) => {
      for (const record of records) {
        for (const added of record.addedNodes) {
          if (
            added.nodeType !== this.ELEMENT_NODE ||
            this !== added.parentNode
          ) {
            continue;
          }

          this.#search(this.#search_term);
          break;
        }
      }
    });

    this.#mutation_observer.observe(this, { subtree: true, childList: true });
  }

  disconnectedCallback() {
    this.#abort_controller?.abort();
    this.#mutation_observer?.disconnect();
  }

  /**
   * @param {SearchListAttributes} name
   * @param {string | null} _
   * @param {string | null} new_value
   */
  attributeChangedCallback(name, _, new_value) {
    switch (name) {
      case "for":
        this.#attach_input(new_value);
        break;
      default:
        break;
    }
  }

  /** @param {string | undefined | null} term  */
  #search(term) {
    const nodes = this.querySelectorAll(":scope>li[search-value]");

    let children;

    if (term && term.length > 0) {
      /** @type {{node: Element, score: number}[]} */
      let search_results = [];
      let total_score = 0;

      for (const elem of nodes) {
        const search_val = elem.getAttribute("search-value") ?? "";
        const score =
          search_val.length < 1
            ? 0
            : calc_score(term, search_val, {
                gap_penalty: 3,
                score_weight: 1,
              });

        total_score += score;
        search_results.push({ score, node: elem });
      }

      const mean = total_score / search_results.length;

      children = search_results
        .sort((a, b) => b.score - a.score)
        .map((e, idx) => {
          const new_node = /** @type {Element} */ (e.node.cloneNode(true));

          if (e.score - mean < 0) {
            new_node.classList.add("hidden");
          } else {
            new_node.classList.remove("hidden");
          }

          return new_node;
        });
    } else {
      children = [...nodes]
        .sort((a, b) =>
          localCompareStr(
            a.getAttribute("search-value"),
            b.getAttribute("search-value"),
          ),
        )
        .map((e) => {
          const new_node = /** @type {Element} */ (e.cloneNode(true));
          new_node.classList.remove("hidden");

          return new_node;
        });
    }

    this.#mutation_observer?.disconnect();
    this.replaceChildren(...children);
    htmx.process(this);
    this.#mutation_observer?.observe(this, {
      subtree: true,
      childList: true,
    });
  }

  /**
   * @param {string | null | undefined} target_name
   */
  #attach_input(target_name) {
    const target_input = /** @type {HTMLInputElement | null} */ (
      document.querySelector(`input[name=${target_name}]`)
    );

    if (!target_input) {
      console.error(
        `Cannot initialize fuzzy search. Target input element not found 'input name=${target_name}'`,
      );
      return;
    }
    this.#abort_controller?.abort();
    this.#abort_controller = new AbortController();

    const listener = (/** @type {Event} */ ev) => {
      /** @type {HTMLInputElement} */
      const input_element = /** @type {HTMLInputElement}*/ (ev.target);
      const search_term = input_element.value;

      this.#search_term = search_term;
      this.#search(search_term);
    };

    const debounced_listener = into_debounced_fn(listener, {
      signal: this.#abort_controller.signal,
      duration: 200,
    });

    target_input.addEventListener("keydown", debounced_listener, {
      signal: this.#abort_controller.signal,
    });

    /// Initial search if input has value
    if (target_input.value && target_input.value.trim().length > 0) {
      this.#search_term = target_input.value;
      this.#search(target_input.value);
    }
  }
}

customElements.define("nut-search-list", SearchList);
