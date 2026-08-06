import fs from "fs";
import path from "path";

const COLORS = {
  red: "\x1b[31m",
  green: "\x1b[32m",
  blue: "\x1b[34m",
};

export function readFile(filePath) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`File not found: ${filePath}`);
  }
  return fs.readFileSync(filePath, "utf-8");
}

export const highlight = async (content, language) => {
  const lines = content.split("\n");
  return lines.map((line) => {
    if (line.startsWith("//")) {
      return `${COLORS.red}${line}`;
    }
    return line;
  });
};

class HighlightEngine {
  #language;
  #state = null;

  constructor(language) {
    this.#language = language;
  }

  get language() {
    return this.#language;
  }

  async process(line) {
    const tokens = this.#tokenize(line);
    return tokens.join("");
  }

  #tokenize(line) {
    return line.split(/(\s+)/);
  }
}

/* Block comment example
   This spans multiple lines
   and should be highlighted
*/

var x = 42;
let result = x instanceof Number;
const fn = () => true;
