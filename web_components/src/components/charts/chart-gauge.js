import {GaugeChart, GaugeTypes} from "@carbon/charts";

/**
 * @typedef {"value" | "delta"  | "height" | "width" | "theme" | "status" | "color"} AttributeKeys
 */

export default class ChartGauge extends HTMLElement {
  /** @type {GaugeChart} */
  #chartInstance

  /** @type {HTMLDivElement} */
  #baseDiv

  /** @type {AttributeKeys[]} */
  static observedAttributes = [
    "value",
    "delta",
    "height",
    "width",
    "theme",
    "status",
    "color"
  ];

  constructor() {
    super();
  }

  connectedCallback() {
    const child = document.createElement("div");
    this.appendChild(child);
    this.#baseDiv = child;

    const fill = window.getComputedStyle(this.#baseDiv).fill;
    const valueText = this.getAttribute("value") ?? "0";
    const deltaText = this.getAttribute("delta") ?? "0";
    const height = this.getAttribute("height") ?? undefined;
    const width = this.getAttribute("width") ?? undefined;
    const theme = this.getAttribute("theme") ?? "g90";
    const color = this.getAttribute("color") ?? fill;
    const status = this.getAttribute("status") ?? "success";

    let valueNumber = Number(valueText);
    let deltaNumber = Number(deltaText);

    valueNumber = isNaN(valueNumber) ? 0 : valueNumber;
    deltaNumber = isNaN(deltaNumber) ? 0 : deltaNumber;

    this.#chartInstance = new GaugeChart(child, {
      data: [
        {
          group: "value",
          value: valueNumber
        },
        {
          group: "delta",
          value: deltaNumber
        }
      ],
      options: {
        animations: true,
        height: height,
        width: width,
        resizable: true,
        theme: theme,
        color: {
          scale: {
            value: color
          }
        },
        gauge: {
          type: GaugeTypes.SEMI,
          status: status,
          showPercentageSymbol: true,
        },
        toolbar: {
          enabled: false,
        }
      }
    });
  }

  disconnectedCallback() {
    this.#baseDiv?.remove();
  }

  /**
   * @param {AttributeKeys} name
   * @param {string} oldValue
   * @param {string} newValue
   */
  attributeChangedCallback(name, oldValue, newValue) {
    if (!this.#chartInstance) return;

    /** @type {{group:string, value:number}[]} */
    const data = this.#chartInstance.model.getData();

    /** @type {import('@carbon/charts').GaugeChartOptions} */
    const options = this.#chartInstance.model.getOptions();

    switch (name) {
      case "value":
        const valuePoint = data.find(e => e.group === "value");
        if (valuePoint) {
          valuePoint.value = Number(newValue) ?? 0;
        }
        break;
      case "delta":
        const delta = data.find(e => e.group === "delta");
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
      case "color" :
        options.color = {
          scale: {
            value: newValue
          }
        };
        break;
    }

    this.#chartInstance.update(true);
  }
}

customElements.define("chart-gauge", ChartGauge);