import React, { useState } from "react";

type Theme = "dark" | "light";

interface HighlighterProps {
  code: string;
  language: string;
  theme?: Theme;
}

interface Token {
  type: "keyword" | "string" | "comment" | "plain";
  value: string;
}

const Highlighter: React.FC<HighlighterProps> = ({ code, language, theme = "dark" }) => {
  const [copied, setCopied] = useState<boolean>(false);

  const handleCopy = async (): Promise<void> => {
    await navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={`highlighter highlighter-${theme}`}>
      <div className="header">
        <span className="language-tag">{language}</span>
        <button onClick={handleCopy}>
          {copied ? "Copied!" : "Copy"}
        </button>
      </div>
      <pre>
        <code>{code}</code>
      </pre>
    </div>
  );
};

export default Highlighter;
