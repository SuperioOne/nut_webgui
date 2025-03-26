import htmx from "htmx.org";
import { into_debounced_fn, localCompareStr } from "../utils";

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
  /** @type {MutationObserver | undefined} */
  #mut_observer;

  /** @type {Set<Element>} */
  #target_nodes;

  /** @type {AbortController | undefined} **/
  #abort_controller;

  /** @type {SearchListAttributes[]} */
  static observedAttributes = ["for"];

  constructor() {
    super();
    this.#target_nodes = new Set();
  }

  connectedCallback() {
    const nodes = this.querySelectorAll(":scope>li[search-value]");
    this.#target_nodes = new Set(nodes);
    this.#attach_input(this.getAttribute("for"));
    this.#mut_observer = new MutationObserver((records) => {
      for (const record of records) {
        for (const added of record.addedNodes) {
          if (
            added.nodeType !== this.ELEMENT_NODE ||
            this !== added.parentNode
          ) {
            continue;
          }

          const element = /** @type {HTMLElement} */ (added);
          const search_value = element.getAttribute("search-value");

          if (search_value && search_value.length > 0) {
            this.#target_nodes.add(element);
          }
        }

        for (const removed of record.removedNodes) {
          if (removed.nodeType !== this.ELEMENT_NODE) {
            continue;
          }

          this.#target_nodes.delete(/** @type {HTMLElement} */ (removed));
        }
      }
    });

    this.#mut_observer.observe(this, { childList: true, subtree: true });
  }

  disconnectedCallback() {
    this.#mut_observer?.disconnect();
    this.#abort_controller?.abort();
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

      let children;

      if (search_term && search_term.length > 0) {
        let sum = 0;
        const results = [...this.#target_nodes].map((e) => {
          const search_val = e.getAttribute("search-value") ?? "";
          const score =
            search_val.length < 1
              ? 0
              : calc_score(search_term, search_val, {
                  gap_penalty: 3,
                  score_weight: 1,
                });

          sum += score;

          return {
            score,
            node: e,
          };
        });

        const mean = sum / results.length;

        children = results
          .sort((a, b) => b.score - a.score)
          .map((e) => {
            const new_node = /** @type {Element} */ (e.node.cloneNode(true));

            if (e.score - mean < 0) {
              new_node.classList.add("hidden");
            } else {
              new_node.classList.remove("hidden");
            }

            return new_node;
          });
      } else {
        children = [...this.#target_nodes]
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

      this.replaceChildren(...children);
      htmx.process(this);
    };

    const debounced_listener = into_debounced_fn(listener, {
      signal: this.#abort_controller.signal,
      duration: 200,
    });

    target_input.addEventListener("keydown", debounced_listener, {
      signal: this.#abort_controller.signal,
    });
  }
}

customElements.define("nut-search-list", SearchList);
