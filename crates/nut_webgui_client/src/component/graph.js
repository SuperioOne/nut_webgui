import dagre from "@dagrejs/dagre";
import {
  getAttributeNumeric,
  into_debounced_fn,
  link_host_styles,
} from "../util.js";
import {
  select as d3_select,
  link as d3_link,
  curveBumpX,
  curveBumpY,
} from "d3";

/** @typedef {"rankdir" | "align" | "x" | "y" | "scale" } GraphAttributes */
/** @typedef {"TB" | "BT" | "LR" | "RL"} RankDirection */
/** @typedef {"UL" | "UR" | "DL" | "DR" } AlignDirection */

/**
 * @typedef Vector2
 * @property {number} x
 * @property {number} y
 */

/**
 * @typedef GraphState
 * @property {Vector2} position
 * @property {number} scale
 * @property {number} width
 * @property {number} height
 * @property {RankDirection} rankdir
 * @property {AlignDirection | undefined} align
 */

const PRIMARY_BUTTON_BITFLAG = 1;
const AUXILIARY_BUTTON_BITFLAG = 4;
const GRAB_STATE = "grabbing";

/** @typedef {0 | 1 | 2 | 3} PositionType */

/** @type {Record<string, PositionType>} */
const Position = {
  Top: 0,
  Right: 1,
  Bottom: 2,
  Left: 3,
};

class GraphNode extends HTMLElement {}
class GraphGroup extends GraphNode {}
class GraphLink extends GraphNode {}

/**
 * @param {import("@dagrejs/dagre").graphlib.Graph} context
 * @param {Element} root_node
 * @param {string | undefined | null} [parent]
 */
function process_nodes(context, root_node, parent) {
  for (const child of root_node.children) {
    switch (child.nodeName) {
      case "NUT-GRAPH-GROUP": {
        const group = child.id;

        context.setNode(group, {
          __type: "group",
        });

        process_nodes(context, child, group);
        break;
      }
      case "NUT-GRAPH-NODE": {
        const label = child.id;
        const width = getAttributeNumeric(child, "width") ?? child.clientWidth;
        const height =
          getAttributeNumeric(child, "height") ?? child.clientHeight;

        context.setNode(label, {
          height: height,
          width: width,
          label: child.innerHTML.trim(),
          __type: "leaf",
        });

        if (parent) {
          context.setParent(label, parent);
        }

        break;
      }
      case "NUT-GRAPH-LINK": {
        const from = child.getAttribute("from") ?? "";
        const to = child.getAttribute("to") ?? "";
        const weight = getAttributeNumeric(child, "weight") ?? 1;
        const from_offset = getAttributeNumeric(child, "from-offset") ?? 0;
        const to_offset = getAttributeNumeric(child, "to-offset") ?? 0;
        const label = child.innerHTML?.trim();

        if (label && label.length > 1) {
          context.setEdge(from, to, {
            class: child.className,
            label,
            label_height: child.clientHeight ?? 0,
            label_width: child.clientWidth ?? 0,
            weight,
            from_offset,
            to_offset,
          });
        } else {
          context.setEdge(
            from,
            to,
            { class: child.className, weight, from_offset, to_offset },
            child.id,
          );
        }

        break;
      }
      default:
        break;
    }
  }
}

/**
 * @param {string} value
 * @returns {AlignDirection | undefined}
 */
function try_into_align(value) {
  switch (value.trim().toUpperCase()) {
    case "DL":
    case "DR":
    case "UL":
    case "UR":
      return /** @type {AlignDirection} */ (value);
    default:
      return undefined;
  }
}

/**
 * @param {string} value
 * @returns {RankDirection | undefined}
 */
function try_into_rankdir(value) {
  switch (value.trim().toUpperCase()) {
    case "BT":
    case "LR":
    case "RL":
    case "TB":
      return /** @type {RankDirection} */ (value);
    default:
      return undefined;
  }
}

/**
 * @param {RankDirection} rankdir
 * @returns {{
 *  src_position: PositionType,
 *  dest_position: PositionType,
 *  curvature: import("d3").CurveFactory
 * }}
 */
function into_edge_config(rankdir) {
  switch (rankdir) {
    case "BT":
      return {
        src_position: Position.Top,
        dest_position: Position.Bottom,
        curvature: curveBumpY,
      };
    case "LR":
      return {
        src_position: Position.Right,
        dest_position: Position.Left,
        curvature: curveBumpX,
      };
    case "RL":
      return {
        src_position: Position.Left,
        dest_position: Position.Right,
        curvature: curveBumpX,
      };
    case "TB":
    default:
      return {
        src_position: Position.Bottom,
        dest_position: Position.Top,
        curvature: curveBumpY,
      };
  }
}

/**
 * @param {Element} element
 * @param {GraphAttributes} name
 * @returns {RankDirection | undefined}
 */
function getAttributeRankdir(element, name) {
  const value = element.getAttribute(name);
  return value ? try_into_rankdir(value) : undefined;
}

/**
 * @param {Element} element
 * @param {GraphAttributes} name
 * @returns {AlignDirection | undefined}
 */
function getAttributeAlign(element, name) {
  const value = element.getAttribute(name);
  return value ? try_into_align(value) : undefined;
}

/**
 * Calculates the edge connection point on the rectangle.
 *
 * # Example
 *
 *   Relative         Current
 *     Node            Node
 *
 *                    position: Position.Right
 *   ┌─────┐          ┌─────┐
 *   │ OH, │          │     │
 *   │ HI  │       ┌─►*  o◄─┼──┐
 *   │MARK!│       │  │     │  │
 *   └─────┘       │  └─────┘  │
 *                 │          rect x and y is the origin point of the rectangle
 *                 │
 *               the connection point function calculates
 *
 * @param {{x:number, y:number, width:number, height:number}} rect Rectangle object
 * @param {PositionType} position direction of the rectangle 'relative' to target node
 * @param {number} [offset] Optional offset value in pixels
 * @returns {Vector2}
 */
function get_connection_point(rect, position, offset) {
  const dw = rect.width / 2;
  const dh = rect.height / 2;
  const edge_offset = offset ?? 0;

  switch (position) {
    case Position.Left:
      return {
        x: rect.x - dw,
        y: rect.y + edge_offset,
      };
    case Position.Top:
      return {
        x: rect.x + edge_offset,
        y: rect.y - dh,
      };
    case Position.Right:
      return {
        x: rect.x + dw,
        y: rect.y + edge_offset,
      };
    case Position.Bottom:
    default:
      return {
        x: rect.x + edge_offset,
        y: rect.y + dh,
      };
  }
}

class Graph extends HTMLElement {
  /** @type {AbortController | undefined} */
  #abort_controller;

  /** @type {Element} */
  #container;

  /** @type{ElementInternals} */
  #internals;

  /** @type{Vector2 | undefined} */
  #pointer_state;

  /** @type{ResizeObserver} */
  #resize_observer;

  /** @type{MutationObserver} */
  #mutation_observer;

  /** @type {ShadowRoot} */
  #shadow_root;

  /** @type {import("d3").Selection<SVGSVGElement, any, null, undefined>} */
  #svg;

  /** @type{GraphState} */
  #state;

  /** @type {GraphAttributes[]} */
  static observedAttributes = ["x", "y", "scale", "rankdir", "align"];

  constructor() {
    super();

    this.#internals = this.attachInternals();
    this.#shadow_root = this.attachShadow({ mode: "closed" });
    link_host_styles(this.#shadow_root);

    const container = document.createElement("div");
    container.style.display = "inline-block";
    container.style.height = "100%";
    container.style.width = "100%";

    this.#container = container;
    this.#shadow_root.append(container);
    this.#state = {
      align: undefined,
      height: 0,
      position: { x: 0, y: 0 },
      rankdir: "TB",
      scale: 1,
      width: 0,
    };
    this.#pointer_state = undefined;
    this.#resize_observer = new ResizeObserver(
      into_debounced_fn((entries) => this.#resize(entries), { duration: 200 }),
    );
    this.#mutation_observer = new MutationObserver((records) => {
      for (const record of records) {
        if (record.target !== this) {
          const target = /** @type{ Element} */ (record.target);

          if (
            record.type === "attributes" &&
            record.attributeName &&
            record.oldValue === target.getAttribute(record.attributeName)
          ) {
            continue;
          } else {
            this.#lazy_render();
            return;
          }
        }
      }
    });
  }

  connectedCallback() {
    const rankdir = getAttributeRankdir(this, "rankdir") ?? "TB";
    const align = getAttributeAlign(this, "align");
    const pos_x = getAttributeNumeric(this, "x") ?? 0;
    const pos_y = getAttributeNumeric(this, "y") ?? 0;
    const scale = getAttributeNumeric(this, "scale") ?? 1;

    this.#abort_controller?.abort();
    this.#abort_controller = new AbortController();
    this.#mutation_observer.observe(this, {
      subtree: true,
      childList: true,
      attributeOldValue: true,
      attributes: true,
    });

    this.addEventListener(
      "pointermove",
      (e) => {
        if (this.#internals.states.has(GRAB_STATE)) {
          const cmp_flag =
            e.pointerType === "mouse"
              ? AUXILIARY_BUTTON_BITFLAG
              : PRIMARY_BUTTON_BITFLAG;

          if (e.buttons === cmp_flag && this.#pointer_state !== undefined) {
            const delta_x = this.#pointer_state.x - e.screenX;
            const delta_y = this.#pointer_state.y - e.screenY;
            const { x, y } = this.#state.position;

            this.#transform(x + delta_x, y + delta_y, this.#state.scale);
          }

          this.#pointer_state = { x: e.screenX, y: e.screenY };
        }
      },
      { signal: this.#abort_controller.signal, passive: true },
    );

    this.addEventListener(
      "pointerdown",
      (e) => {
        if (
          e.pointerType !== "mouse" ||
          (e.pointerType === "mouse" && e.buttons === AUXILIARY_BUTTON_BITFLAG)
        ) {
          this.#internals.states.add(GRAB_STATE);
        }
      },
      { signal: this.#abort_controller.signal, passive: true },
    );

    const cancel_action = () => {
      if (this.#internals.states.has(GRAB_STATE)) {
        this.#internals.states.delete(GRAB_STATE);
        this.#pointer_state = undefined;
      }
    };

    this.addEventListener("pointerup", cancel_action, {
      signal: this.#abort_controller.signal,
      passive: true,
    });

    this.addEventListener("pointercancel", (e) => {
      e.preventDefault();
      e.stopPropagation();
      cancel_action();
    });

    this.addEventListener(
      "wheel",
      (e) => {
        const pivot = { x: this.clientWidth / 2, y: this.clientHeight / 2 };
        const scale = Math.max(
          this.#state.scale - Math.sign(e.deltaY) * 0.1,
          0.1,
        );
        this.#scale(scale, pivot);
      },
      { signal: this.#abort_controller.signal, passive: true },
    );

    this.#state = {
      width: 0,
      height: 0,
      rankdir,
      align,
      position: { x: pos_x, y: pos_y },
      scale,
    };
    this.#render();
  }

  disconnectedCallback() {
    this.#resize_observer.disconnect();
    this.#mutation_observer.disconnect();
  }

  /**
   * @param {GraphAttributes} name
   * @param {string} old_value
   * @param {string} new_value
   */
  attributeChangedCallback(name, old_value, new_value) {
    const target = this.#container;

    if (!target || old_value === new_value) {
      return;
    }

    switch (name) {
      case "align": {
        this.#state.align = try_into_align(new_value);
        this.#lazy_render();
        break;
      }
      case "rankdir": {
        this.#state.rankdir = try_into_rankdir(new_value) ?? "TB";
        this.#transform(0, 0, this.#state.scale);
        this.#lazy_render();
        break;
      }
      case "x": {
        const pos_x = Number(new_value);

        if (!isNaN(pos_x)) {
          this.#transform(pos_x, this.#state.position.y, this.#state.scale);
        }
        break;
      }
      case "y": {
        const pos_y = Number(new_value);

        if (!isNaN(pos_y)) {
          this.#transform(this.#state.position.x, pos_y, this.#state.scale);
        }
        break;
      }
      case "scale": {
        const scale = Number(new_value);

        if (!isNaN(scale)) {
          const pivot = { x: this.clientWidth / 2, y: this.clientHeight / 2 };
          this.#scale(Math.max(scale, 0.1), pivot);
        }
        break;
      }
      default:
        break;
    }
  }

  #render() {
    this.#resize_observer.unobserve(this.#container);

    const { rankdir, align, position, scale } = this.#state;
    const temp_layout = document.createElement("div");

    for (const child of this.children) {
      temp_layout.appendChild(child.cloneNode(true));
    }

    this.#container.append(temp_layout);

    setTimeout(() => {
      const graph = new dagre.graphlib.Graph({
        multigraph: true,
        compound: true,
      })
        .setGraph({
          marginx: 16,
          marginy: 16,
          rankdir,
          align,
        })
        .setDefaultEdgeLabel(function () {
          return {};
        });

      process_nodes(graph, temp_layout);
      dagre.layout(graph);

      const svg_root = document.createElement("div");
      svg_root.style.display = "inline-block";
      svg_root.style.height = "100%";
      svg_root.style.width = "100%";

      const graph_label = graph.graph();
      const graph_height = Math.max(this.scrollHeight, graph_label.height ?? 0);
      const graph_width = Math.max(this.scrollWidth, graph_label.width ?? 0);
      const svg_container = d3_select(svg_root);
      const svg = svg_container
        .append("svg")
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .attr("height", graph_height)
        .attr("width", graph_width)
        .attr("min-height", graph_height)
        .attr("min-width", graph_width)
        .attr("style", "object-fit: none;")
        .attr("viewBox", `0 0 ${graph_width} ${graph_height}`);

      const group = svg
        .append("g")
        .attr(
          "transform",
          `translate(${-position.x} ${-position.y}) scale(${scale})`,
        );

      const edge_config = into_edge_config(rankdir);

      for (const v of graph.nodes()) {
        const node = graph.node(v);

        if (node.__type === "leaf") {
          const edges = graph.outEdges(v);

          if (edges) {
            for (const edge of edges) {
              const src_node = graph.node(edge.v);
              const dest_node = graph.node(edge.w);
              const graph_edge = graph.edge(edge);

              const start = get_connection_point(
                src_node,
                edge_config.src_position,
                graph_edge.from_offset,
              );
              const dest = get_connection_point(
                dest_node,
                edge_config.dest_position,
                graph_edge.to_offset,
              );

              const path = d3_link(edge_config.curvature)({
                source: [start.x, start.y],
                target: [dest.x, dest.y],
              });

              const svg_path = group.append("path").attr("d", path);

              if (graph_edge.class) {
                svg_path.attr("class", graph_edge.class);
              } else {
                svg_path.attr("style", "fill:none;stroke:green;stroke-width:3");
              }

              if (
                graph_edge.label &&
                graph_edge.label_width &&
                graph_edge.label_height
              ) {
                const labelX = start.x + (dest.x - start.x) / 2;
                const labelY = start.y + (dest.y - start.y) / 2;

                group
                  .append("foreignObject")
                  .attr("x", labelX - graph_edge.label_width / 2)
                  .attr("y", labelY - graph_edge.label_height / 2)
                  .attr("width", graph_edge.label_width)
                  .attr("height", graph_edge.label_height)
                  .append("xhtml:div")
                  .html(graph_edge.label);
              }
            }
          }

          group
            .append("foreignObject")
            .attr("x", node.x - node.width / 2)
            .attr("y", node.y - node.height / 2)
            .attr("width", node.width)
            .attr("height", node.height)
            .append("xhtml:div")
            .html(node.label ?? "");
        }
      }

      this.#state.height = graph_height;
      this.#state.width = graph_width;
      this.#container.replaceChildren(...svg_root.children);
      this.#svg = svg;
      this.#resize_observer.observe(this.#container);
    }, 0);
  }

  #lazy_render = into_debounced_fn(() => this.#render(), { duration: 33 });

  /**
   * @param {ResizeObserverEntry[]} entries
   */
  #resize(entries) {
    for (const entry of entries) {
      if (entry.target === this.#container) {
        const height = Math.max(entry.target.scrollHeight, this.#state.height);
        const width = Math.max(entry.target.scrollWidth, this.#state.width);

        this.#svg
          ?.attr("height", height)
          .attr("width", width)
          .attr("viewBox", `0 0 ${width} ${height}`);
        this.#state.height = height;
        this.#state.width = width;

        return;
      }
    }
  }

  /**
   * @param {number} x
   * @param {number} y
   * @param {number} scale
   */
  #transform(x, y, scale) {
    this.#state.position.x = x;
    this.#state.position.y = y;
    this.#state.scale = scale;
    this.setAttribute("x", x.toString());
    this.setAttribute("y", y.toString());
    this.setAttribute("scale", scale.toString());

    this.#svg
      ?.select("g")
      ?.attr("transform", `translate(${-x} ${-y}) scale(${scale})`);
  }

  /**
   * @param {number} value
   * @param {Vector2} screen_pivot
   */
  #scale(value, screen_pivot) {
    const { scale, position } = this.#state;
    const pivot_x = (position.x + screen_pivot.x) / scale;
    const pivot_y = (position.y + screen_pivot.y) / scale;
    const target_x = pivot_x * value - screen_pivot.x;
    const target_y = pivot_y * value - screen_pivot.y;

    this.#transform(target_x, target_y, value);
  }
}

customElements.define("nut-graph", Graph);
customElements.define("nut-graph-node", GraphNode);
customElements.define("nut-graph-link", GraphLink);
customElements.define("nut-graph-group", GraphGroup);
