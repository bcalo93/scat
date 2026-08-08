import java.util.HashMap;
import java.util.List;
import java.util.ArrayList;

public class Highlighter {
    private String language;
    private boolean inBlockComment;

    public Highlighter(String language) {
        this.language = language;
        this.inBlockComment = false;
    }

    public String highlightLine(String line) {
        if (inBlockComment) {
            int endIndex = line.indexOf("*/");
            if (endIndex == -1) {
                return line;
            }
            inBlockComment = false;
            return line.substring(0, endIndex + 2);
        }

        if (line.startsWith("//")) {
            return line;
        }

        if (line.contains("/*")) {
            if (!line.contains("*/")) {
                inBlockComment = true;
            }
            return line;
        }

        return line;
    }

    public String getLanguage() {
        return this.language;
    }

    @Override
    public String toString() {
        return "Highlighter{" + language + "}";
    }

    public static void main(String[] args) {
        Highlighter h = new Highlighter("java");
        String[] lines = {
            "public static void main(String[] args) {",
            "    System.out.println(\"Hello, world!\");",
            "}"
        };

        for (String line : lines) {
            System.out.println(h.highlightLine(line));
        }

        HashMap<String, Integer> map = new HashMap<>();
        map.put("java", 1);
        map.put("kotlin", 2);

        List<String> keys = new ArrayList<>(map.keySet());
        System.out.println("Languages: " + keys);
    }
}

/* Block comment example
   This should be highlighted
*/
