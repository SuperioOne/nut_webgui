import { GaugeChart, GaugeTypes } from "@carbon/charts";

/**
 * @typedef {"value" | "delta"  | "height" | "width" | "theme" | "status" | "color"} AttributeKeys
 */

export default class ChartGauge extends HTMLElement {
  /** @type {GaugeChart} */
  #chart_inst;

  /** @type {HTMLDivElement} */
  #base_div;

  /** @type {AttributeKeys[]} */
  static observedAttributes = [
    "value",
    "delta",
    "height",
    "width",
    "theme",
    "status",
    "color",
  ];

  constructor() {
    super();
  }

  connectedCallback() {
    const child = document.createElement("div");
    this.#base_div = child;
    this.replaceChildren(child);

    const fill = window.getComputedStyle(this).fill;
    const value_text = this.getAttribute("value") ?? "0";
    const delta_text = this.getAttribute("delta") ?? "0";
    const height = this.getAttribute("height") ?? undefined;
    const width = this.getAttribute("width") ?? undefined;
    const theme = this.getAttribute("theme") ?? "g90";
    const color = this.getAttribute("color") ?? fill;
    const status = this.getAttribute("status") ?? "success";

    let value_number = Number(value_text);
    let delta_number = Number(delta_text);

    value_number = isNaN(value_number) ? 0 : value_number;
    delta_number = isNaN(delta_number) ? 0 : delta_number;

    this.#chart_inst = new GaugeChart(child, {
      data: [
        {
          group: "value",
          value: value_number,
        },
        {
          group: "delta",
          value: delta_number,
        },
      ],
      options: {
        animations: true,
        height: height,
        width: width,
        resizable: true,
        theme: theme,
        color: {
          scale: {
            value: color,
          },
        },
        gauge: {
          type: GaugeTypes.SEMI,
          status: status,
          showPercentageSymbol: true,
        },
        toolbar: {
          enabled: false,
        },
      },
    });
  }

  disconnectedCallback() {
    this.#chart_inst?.destroy();
    this.#base_div?.remove();
  }

  /**
   * @param {AttributeKeys} name
   * @param {string} oldValue
   * @param {string} newValue
   */
  attributeChangedCallback(name, oldValue, newValue) {
    if (!this.#chart_inst) return;

    /** @type {{group:string, value:number}[]} */
    const data = this.#chart_inst.model.getData();

    /** @type {import('@carbon/charts').GaugeChartOptions} */
    const options = this.#chart_inst.model.getOptions();

    switch (name) {
      case "value":
        const point = data.find((e) => e.group === "value");
        if (point) {
          point.value = Number(newValue) ?? 0;
        }
        break;
      case "delta":
        const delta = data.find((e) => e.group === "delta");
        if (delta) {
          delta.value = Number(newValue) ?? 0;
        }
        break;
      case "height":
        options.height = newValue;
        break;
      case "width":
        options.width = newValue;
        break;
      case "theme":
        options.theme = newValue;
        break;
      case "status":
        if (options.gauge) {
          options.gauge.status = newValue;
        }
        break;
      case "color":
        options.color = {
          scale: {
            value: newValue,
          },
        };
        break;
    }

    this.#chart_inst.update(true);
  }
}

customElements.define("chart-gauge", ChartGauge);

