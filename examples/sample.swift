import Foundation

protocol Highlighter {
    func highlight(line: String) -> String
}

class SwiftHighlighter: Highlighter {
    private let language: String
    private var inBlockComment = false

    init(language: String) {
        self.language = language
    }

    func highlight(line: String) -> String {
        if inBlockComment {
            if let endRange = line.range(of: "*/") {
                inBlockComment = false
                return String(line[...endRange.upperBound])
            }
            return line
        }

        if line.hasPrefix("//") {
            return line
        }

        if line.contains("/*") {
            if !line.contains("*/") {
                inBlockComment = true
            }
            return line
        }

        return line
    }
}

enum Token {
    case keyword(String)
    case string(String)
    case number(Int)
    case comment(String)
    case plain(String)
}

struct Range {
    let start: Int
    let end: Int

    var length: Int {
        return end - start
    }
}

func createHighlighter(for language: String) -> Highlighter {
    switch language {
    case "swift":
        return SwiftHighlighter(language: language)
    default:
        fatalError("Unknown language: \(language)")
    }
}

let languages = ["swift", "rust", "go", "javascript"]
for lang in languages {
    print("Language: \(lang)")
}

let result: [String] = languages.compactMap { lang in
    guard !lang.isEmpty else { return nil }
    return lang.uppercased()
}
