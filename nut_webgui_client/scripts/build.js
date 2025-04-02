import * as esbuild from "esbuild";
import { readdirSync, copyFileSync, mkdirSync } from "node:fs";
import { join, parse } from "node:path";
import { spawnSync } from "node:child_process";

/**
 * @param {string | undefined} value
 * @returns {boolean}
 */
function parse_boolean(value) {
  switch (value?.toLowerCase()) {
    case "true":
    case "1":
    case undefined:
      return true;
    case "false":
    case "0":
      return false;
    default:
      throw new Error(`Expecting a boolean value, got '${value}'`);
  }
}

/**
 * @param {string | undefined} value
 * @returns {number}
 */
function parse_number(value) {
  const integer = parseInt(value);

  if (isNaN(integer)) {
    throw new Error(`Expecting a numeric value, got '${value}'`);
  } else {
    return integer;
  }
}

/**
 * @param {string | undefined} value
 * @returns {string}
 */
function parse_str(value) {
  if (value && value.length > 0) {
    return value;
  } else {
    throw new Error(`Expecting a string value, got '${value}'`);
  }
}

/**
 * Very basic cli arg parser instead of adding 100 more node_modules.
 *
 * @template {Record<string, "number" | "boolean" | "string" >} T
 * @param {T} config
 * @returns {Partial<{
 *    [Property in keyof T]: T[Property] extends "string"
 *        ? string
 *        : T[Property] extends "boolean"
 *            ? boolean
 *        : number
 * }>}
 */
function cli_arg_parser(config) {
  /** @type{T} **/
  let cli_args = {};

  for (const arg of process.argv.slice(2)) {
    const [argn, argv] = arg.trim().split("=", 2);

    try {
      if (argn && !argn.startsWith("--") && argn.length < 3) {
        throw new Error("Unknown flag");
      }

      const name = argn.slice(2);
      /** @type{"number"| "boolean" | "string" | undefined} **/
      const key_type = Reflect.get(config, name);

      switch (key_type) {
        case "string":
          cli_args[name] = parse_str(argv);
          break;
        case "boolean":
          cli_args[name] = parse_boolean(argv);
          break;
        case "number":
          cli_args[name] = parse_number(argv);
          break;
        default:
          throw new Error("Unknown flag");
      }
    } catch (err) {
      console.error(`Parsing ${argn} failed, ${err.message}.`);
      process.exit(1);
    }
  }

  return cli_args;
}

/**
 * @param {string} src
 * @param {string} dest
 */
function copy_dir(src, dest) {
  const files = readdirSync(src, {
    recursive: true,
  });

  for (const file of files) {
    const source_file = join(src, file);
    const target_file = join(dest, file);
    const base_dir = parse(target_file).dir;

    mkdirSync(base_dir, { recursive: true });

    console.log(`Copying ${source_file} -> ${target_file}`);
    copyFileSync(source_file, target_file);
  }
}

/**
 * @param {{
 *   minify?: boolean,
 *   outdir: string,
 * }} config
 */
async function build(config) {
  mkdirSync(config.outdir, { recursive: true });
  copy_dir("./static", config.outdir);

  await esbuild.build({
    bundle: true,
    entryPoints: ["./src/index.js"],
    format: "iife",
    minify: config.minify,
    target: ["firefox109", "chrome108", "safari15"],
    treeShaking: true,
    outdir: config.outdir,
  });

  const output_css = join(config.outdir, "style.css");

  // Don't want to deal with postcss, paths are fixed.
  let tailwind_process = spawnSync(
    "./node_modules/@tailwindcss/cli/dist/index.mjs",
    ["-i", "./src/style.css", "-o", output_css, "--minify"],
  );

  console.log(tailwind_process.stdout.toString());
  console.error(tailwind_process.stderr.toString());

  if (tailwind_process.error) {
    throw tailwind_process.error;
  } else if (tailwind_process.status != 0) {
    throw new Error("Tailwindcss bundling failed.");
  }
}

async function main() {
  const opts = cli_arg_parser({
    outdir: "string",
    minify: "boolean",
  });

  if (!opts.outdir || opts.outdir.length < 1) {
    console.error("Output directory is not configured.");
    process.exit(1);
  }

  try {
    await build(opts);
    console.log("Build Completed.");
  } catch (err) {
    console.error(err.message);
    process.exit(1);
  }
}

main();
