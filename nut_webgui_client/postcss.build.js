#!/bin/node

import cssnano from "cssnano";
import fs from "node:fs";
import postcss from "postcss";
import tailwind from "@tailwindcss/postcss";
import { argv, exit } from "node:process";

const TARGET = argv[2];
const SOURCE = "./src/style.css";

if (!TARGET || TARGET.trim().length === 0) {
  console.error("no target path is defined");
  exit(1);
}

fs.readFile(SOURCE, (err, css) => {
  if (err) {
    console.error(err.message);
    exit(1);
  }

  postcss([
    tailwind,
    cssnano({
      preset: "default",
    }),
  ])
    .process(css, { from: SOURCE, to: TARGET })
    .then((result) => {
      fs.writeFile(TARGET, result.css, () => true);
    });
});

