package main

import (
	"fmt"
	"os"
	"strings"
)

// Highlighter represents a syntax highlighting engine
type Highlighter struct {
	language     string
	inBlockComment bool
}

// NewHighlighter creates a new highlighter for the given language
func NewHighlighter(lang string) *Highlighter {
	return &Highlighter{
		language: lang,
	}
}

// HighlightLine processes a single line of code
func (h *Highlighter) HighlightLine(line string) string {
	if h.inBlockComment {
		idx := strings.Index(line, "*/")
		if idx == -1 {
			return line
		}
		h.inBlockComment = false
		return line[:idx+2] + h.HighlightLine(line[idx+2:])
	}

	if strings.HasPrefix(line, "//") {
		return line
	}

	if idx := strings.Index(line, "/*"); idx != -1 {
		endIdx := strings.Index(line[idx+2:], "*/")
		if endIdx == -1 {
			h.inBlockComment = true
			return line
		}
		return line[:idx+endIdx+4] + h.HighlightLine(line[idx+endIdx+4:])
	}

	return line
}

func main() {
	h := NewHighlighter("go")
	lines := []string{
		"package main",
		"",
		"func main() {",
		"    fmt.Println(\"Hello, world!\")",
		"}",
	}

	for _, line := range lines {
		fmt.Println(h.HighlightLine(line))
	}

	_ = os.Stdout
}
