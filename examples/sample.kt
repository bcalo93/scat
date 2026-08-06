import java.util.Objects

data class Highlighter(
    val language: String,
    var inBlockComment: Boolean = false
) {
    fun highlightLine(line: String): String {
        if (inBlockComment) {
            val endIndex = line.indexOf("*/")
            if (endIndex == -1) {
                return line
            }
            inBlockComment = false
            return line.substring(0, endIndex + 2)
        }

        if (line.startsWith("//")) {
            return line
        }

        if (line.contains("/*")) {
            if (!line.contains("*/")) {
                inBlockComment = true
            }
            return line
        }

        return line
    }
}

interface SyntaxEngine {
    fun process(input: String): List<String>
}

sealed class Token {
    data class Keyword(val value: String) : Token()
    data class String(val value: String) : Token()
    data class Number(val value: Int) : Token()
    data class Comment(val value: String) : Token()
    data class Plain(val value: String) : Token()
}

fun createEngine(language: String): SyntaxEngine {
    return object : SyntaxEngine {
        override fun process(input: String): List<String> {
            return input.lines()
        }
    }
}

fun main() {
    val highlighter = Highlighter("kotlin")
    val lines = listOf(
        "fun main() {",
        "    println(\"Hello, world!\")",
        "}"
    )

    for (line in lines) {
        println(highlighter.highlightLine(line))
    }

    val tokens = listOf(
        Token.Keyword("fun"),
        Token.Plain(" "),
        Token.Keyword("main"),
        Token.Plain("() {}")
    )
    println(tokens)
}
