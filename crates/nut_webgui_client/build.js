#!/bin/node

import commonjs from "@rollup/plugin-commonjs";
import cssnano from "cssnano";
import fs from "node:fs/promises";
import postcss from "postcss";
import tailwind from "@tailwindcss/postcss";
import terser from "@rollup/plugin-terser";
import { argv, exit } from "node:process";
import { babel } from "@rollup/plugin-babel";
import { nodeResolve } from "@rollup/plugin-node-resolve";
import { rollup } from "rollup";

/**
 * @param {string | undefined | null} value
 * @returns {value is undefined}
 */
function is_empty_str(value) {
  return typeof value !== "string" || value.trim().length === 0;
}

/**
 * @param {string | undefined | null} value
 * @returns {boolean}
 */
function try_parse_bool(value) {
  switch (value?.toLowerCase()) {
    case "true":
    case "1":
      return true;
    case "false":
    case "0":
      return false;
    default:
      throw new Error("unable to parse boolean value");
  }
}

/**
 * @param {string | undefined | null} value
 * @returns {value is string}
 */
function is_arg_value(value) {
  return !is_empty_str(value) && value[0] !== "-";
}

/**
 * @returns {{
 *  command: "bundle-js" | "bundle-css";
 *  minify: boolean;
 *  output: string;
 *  input:string
 * }}
 */
function parse_args() {
  const result = {
    command: undefined,
    minify: false,
    output: undefined,
    input: undefined,
  };

  switch (argv[2]) {
    case "bundle-js":
    case "bundle-css":
      result.command = argv[2];
      break;
    default:
      throw new Error(`unknown command: ${argv[2]}`);
  }

  const args_iter = argv.slice(3)[Symbol.iterator]();
  let current = args_iter.next();

  while (current.done !== true) {
    switch (current.value) {
      case "--input":
      case "-i": {
        const input = args_iter.next();

        if (is_arg_value(input.value)) {
          result.input = input.value;
        } else {
          throw new Error(`${current.value} option requires value.`);
        }
        break;
      }
      case "--output":
      case "-o": {
        const output = args_iter.next();

        if (is_arg_value(output.value)) {
          result.output = output.value;
        } else {
          throw new Error(`${current.value} option requires value.`);
        }
        break;
      }
      case "--minify": {
        const flag = args_iter.next();
        result.minify = try_parse_bool(flag.value);
        break;
      }
      default: {
        throw new Error(`unknown option: ${current.value}`);
      }
    }

    current = args_iter.next();
  }

  if (is_empty_str(result.output)) {
    throw new Error("output path is not set.");
  }

  if (is_empty_str(result.input)) {
    throw new Error("input path is not set");
  }

  return result;
}

/**
 * @param {string} src Input file
 * @param {string} dest Output directory path
 * @param {boolean} [minify] Minify the output
 */
async function bundle_css(src, dest, minify) {
  const css = await fs.readFile(src);
  const plugins = [tailwind];

  if (minify) {
    plugins.push(
      cssnano({
        preset: "default",
      }),
    );
  }

  const result = await postcss(plugins).process(css, { from: src, to: dest });
  await fs.writeFile(dest, result.css);
}

/**
 * @param {string} src Input file
 * @param {string} dest Output directory path
 * @param {boolean} [minify] Minify the output
 */
async function bundle_js(src, dest, minify) {
  const options = {
    treeshake: true,
    input: src,
    plugins: [
      nodeResolve(),
      commonjs({
        transformMixedEsModules: true,
      }),
      babel({
        babelHelpers: "bundled",
        presets: ["@babel/preset-env"],
        targets: {
          firefox: 109,
          chrome: 109,
          safari: 26,
        },
      }),
    ],
    output: {
      format: "iife",
      file: dest,
    },
  };

  if (minify) {
    options.plugins.push(terser());
  }

  const bundle = await rollup(options);
  await bundle.write(options.output);
  await bundle.close();
}

try {
  const args = parse_args();

  switch (args.command) {
    case "bundle-js":
      bundle_js(args.input, args.output, args.minify)
        .then(() => exit(0))
        .catch((err) => console.error(err?.message ?? "js bundler failed."));
      break;
    case "bundle-css":
      bundle_css(args.input, args.output, args.minify)
        .then(() => exit(0))
        .catch((err) => console.error(err?.message ?? "css bundler failed."));
      break;
  }
} catch (err) {
  console.error(err.message);
  exit(1);
}
