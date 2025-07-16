import { link_host_styles } from "../utils.js";
import {
  arc as d3_arc,
  create as d3_create,
  interpolate as d3_interpolate,
} from "d3";

const START_ANGLE = -Math.PI / 2;
const END_ANGLE = Math.PI / 2;

/** @import {Selection, Arc, DefaultArcObject} from "d3" */

/**
 * @param {number} value
 * @returns {number}
 * */
function calc_angle(value) {
  let rounded = Math.floor(value);

  if (rounded >= 100) {
    return END_ANGLE;
  } else if (rounded <= 0) {
    return START_ANGLE;
  } else {
    return (Math.PI * rounded) / 100 + START_ANGLE;
  }
}

/**
 * @typedef {"value" | "height" | "width" | "class" } GaugeAtrributes
 */
export default class Gauge extends HTMLElement {
  /** @type { Selection<SVGTextElement, undefined, null, undefined> | undefined} */
  #text;

  /** @type { Selection<SVGPathElement, { endAngle: number; }, null, undefined> | undefined} */
  #value_track;

  /** @type {ShadowRoot} */
  #shadow_root;

  /** @type{Arc<any, DefaultArcObject> | undefined}*/
  #gauge_arc;

  /** @type {GaugeAtrributes[]} */
  static observedAttributes = ["value", "class"];

  constructor() {
    super();
    this.#shadow_root = this.attachShadow({ mode: "closed" });
    link_host_styles(this.#shadow_root);
  }

  connectedCallback() {
    const value_text = this.getAttribute("value") ?? "0";

    let value_number = Number(value_text);
    value_number = isNaN(value_number) ? 0 : value_number;

    const height = Math.min(500, this.clientWidth / 2);
    const outerRadius = height;
    const innerRadius = outerRadius * 0.85;
    const font_size = innerRadius / 3;

    const gauge = d3_create("svg").attr("viewBox", [
      0,
      0,
      this.clientWidth,
      height,
    ]);

    const tracks = gauge
      .append("g")
      .attr(
        "transform",
        `translate(${this.clientWidth / 2}, ${(height + outerRadius) / 2})`,
      );

    const arc = d3_arc()
      .outerRadius(outerRadius)
      .innerRadius(innerRadius)
      .startAngle(START_ANGLE);

    tracks
      .append("path")
      .datum({ endAngle: END_ANGLE })
      .style("fill", "#00000025")
      .attr("d", /** @type{any}*/ (arc));

    this.#value_track = tracks
      .append("path")
      .datum({ endAngle: calc_angle(value_number) })
      .attr("d", /** @type{any}*/ (arc));

    this.#text = gauge
      .append("text")
      .attr("text-anchor", "middle")
      .attr("transform", `translate(${this.clientWidth / 2}, ${height})`)
      .style("font-size", `${font_size}px`)
      .text(`${value_text}%`);

    this.#gauge_arc = arc;

    let node = gauge.node();

    if (node) {
      this.#shadow_root.appendChild(node);
    }
  }

  /** @param {unknown} value  */
  set_value(value) {
    const input_val = Number(value);

    if (Number.isNaN(input_val)) {
      console.error(`${value} is not a number. Unable to set gauge value`);
    } else {
      const new_angle = calc_angle(input_val);

      if (this.#value_track && this.#text && this.#gauge_arc) {
        const arc = this.#gauge_arc;

        this.#value_track
          .transition()
          .duration(500)
          .attrTween("d", (d) => {
            const interpolate = d3_interpolate(d.endAngle, new_angle);

            return (t) => {
              d.endAngle = interpolate(t);
              return arc(d);
            };
          });
        this.#text.text(`${value}%`);
      } else {
        console.error("Cannot set value, gauge nodes are empty.");
      }
    }
  }

  /**
   * @param {GaugeAtrributes} name
   * @param {string} _old_value
   * @param {string} new_value
   */
  attributeChangedCallback(name, _old_value, new_value) {
    if (!this.#value_track || !this.#text) {
      return;
    }

    switch (name) {
      case "value": {
        this.set_value(new_value);
        break;
      }
      default:
        break;
    }
  }
}

customElements.define("nut-gauge", Gauge);
