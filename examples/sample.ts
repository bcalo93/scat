interface Config {
  theme: "dark" | "light";
  lineNumbers: boolean;
  maxLineWidth?: number;
}

type Language =
  | "rust"
  | "javascript"
  | "typescript"
  | "go"
  | "python";

abstract class BaseHighlighter {
  protected language: Language;

  constructor(language: Language) {
    this.language = language;
  }

  abstract highlight(line: string): string;
}

class GenericHighlighter extends BaseHighlighter {
  private inBlockComment: boolean = false;

  constructor(language: Language) {
    super(language);
  }

  highlight(line: string): string {
    if (this.inBlockComment) {
      const endIndex = line.indexOf("*/");
      if (endIndex === -1) {
        return `\x1b[90m${line}`;
      }
      this.inBlockComment = false;
      return `\x1b[90m${line.substring(0, endIndex + 2)}`;
    }

    const keywords = ["fn", "let", "const", "return", "if", "else"];
    const tokens: string[] = [];

    for (const word of line.split(/\s+/)) {
      if (keywords.includes(word)) {
        tokens.push(`\x1b[35m${word}\x1b[0m`);
      } else {
        tokens.push(word);
      }
    }

    return tokens.join(" ");
  }
}

export function createHighlighter(lang: Language): BaseHighlighter {
  switch (lang) {
    case "rust":
      return new GenericHighlighter("rust");
    case "javascript":
    case "typescript":
      return new GenericHighlighter(lang);
    default:
      throw new Error(`Unknown language: ${lang}`);
  }
}

const result: readonly string[] = Object.freeze(["hello", "world"]);
const count: number = result.length;
const active: boolean = count > 0;
