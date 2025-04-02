import { link_host_styles } from "../utils.js";
import ApexCharts from "apexcharts";

/** @import {ApexOptions} from "apexcharts" */

/**
 * @typedef {"value" | "height" | "width" | "class" } GaugeAtrributes
 */

export default class Gauge extends HTMLElement {
  /** @type {ApexCharts | undefined} */
  #chart;

  /** @type {() => void} **/
  #theme_listener = () => {
    if (this.#chart) {
      /** @type {ApexOptions} **/
      const new_options = {
        plotOptions: {
          radialBar: {
            dataLabels: {
              value: {
                color: window.getComputedStyle(this).color,
              },
            },
          },
        },
        fill: {
          colors: [window.getComputedStyle(this).fill],
        },
      };

      this.#chart.updateOptions(new_options, false, false).catch(console.error);
    }
  };

  /** @type {AbortController | undefined} **/
  #abort_controller;

  /** @type {GaugeAtrributes[]} */
  static observedAttributes = ["value", "height", "width", "class"];

  constructor() {
    super();
  }

  connectedCallback() {
    const shadow_root = this.attachShadow({ mode: "closed" });
    const child = document.createElement("div");
    shadow_root.replaceChildren(child);
    link_host_styles(shadow_root);

    const value_text = this.getAttribute("value") ?? "0";
    const height = this.getAttribute("height") ?? "auto";
    const width = this.getAttribute("width") ?? "100%";

    let value_number = Number(value_text);
    value_number = isNaN(value_number) ? 0 : value_number;

    /** @type {ApexOptions} **/
    const options = {
      series: [value_number],
      chart: {
        height: height,
        width: width,
        type: "radialBar",
        offsetY: -20,
        sparkline: {
          enabled: true,
        },
      },
      plotOptions: {
        radialBar: {
          hollow: {
            size: "70",
            margin: 10,
          },
          startAngle: -90,
          endAngle: 90,
          track: {
            background: "#a0a0a0",
            strokeWidth: "90",
            margin: 10,
          },
          dataLabels: {
            name: {
              show: false,
            },
            value: {
              offsetY: -2,
              fontSize: "2.5rem",
              color: window.getComputedStyle(this).color,
            },
          },
        },
      },
      fill: {
        type: "solid",
        colors: [window.getComputedStyle(this).fill],
        opacity: 0.5,
      },
    };

    this.#abort_controller = new AbortController();
    this.#chart = new ApexCharts(child, options);
    this.#chart.render().catch(console.error);

    document.addEventListener("theme-change", this.#theme_listener, {
      signal: this.#abort_controller.signal,
    });
  }

  disconnectedCallback() {
    this.#chart?.destroy();
    this.#abort_controller?.abort();
  }

  /**
   * @param {GaugeAtrributes} name
   * @param {string} _old_value
   * @param {string} new_value
   */
  attributeChangedCallback(name, _old_value, new_value) {
    if (!this.#chart) return;

    switch (name) {
      case "value": {
        const series_value = Number(new_value) ?? 0;
        this.#chart.updateSeries([series_value], true).catch(console.error);
        break;
      }
      case "height":
        this.#chart
          .updateOptions({
            chart: {
              height: new_value ?? "auto",
            },
          })
          .catch(console.error);
        break;
      case "width":
        this.#chart
          .updateOptions({
            chart: {
              width: new_value ?? "100%",
            },
          })
          .catch(console.error);
        break;
      case "class": {
        this.#chart.updateOptions({}, false, false).catch(console.error);
        break;
      }
      default:
        break;
    }
  }
}

customElements.define("nut-gauge", Gauge);
