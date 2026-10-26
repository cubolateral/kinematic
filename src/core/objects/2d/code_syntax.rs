use crate::core::{TrackChoices, TrackEnum, TrackValue, TrackValueType};
use std::{
    collections::VecDeque,
    ops::Range,
    sync::{Arc, LazyLock, Mutex},
};
use syntect::{
    easy::ScopeRegionIterator,
    parsing::{ParseState, Scope, ScopeStack, SyntaxDefinition, SyntaxSet},
    util::LinesWithEndings,
};

macro_rules! languages {
    ($($variant:ident => ($name:literal, $extension:literal)),* $(,)?) => {
        /// Syntax coloring language of a code object.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum CodeLanguage { $($variant,)* }

        impl Default for CodeLanguage {
            fn default() -> Self { Self::Plain }
        }

        impl CodeLanguage {
            fn extension(self) -> &'static str { match self { $(Self::$variant => $extension,)* } }
        }

        impl TrackValueType for CodeLanguage {
            type Input = Self;
            const CHOICES: TrackChoices = TrackChoices::Enum(Self::VARIANTS);
            fn into_track_value(self) -> TrackValue {
                TrackValue::Enum(match self { $(Self::$variant => $name,)* })
            }
            fn from_track_value(value: TrackValue) -> Option<Self> {
                match value { $(TrackValue::Enum($name) => Some(Self::$variant),)* _ => None }
            }
        }

        impl TrackEnum for CodeLanguage {
            const VARIANTS: &'static [&'static str] = &[$($name,)*];
        }
    };
}

languages! {
    Lua => ("lua", "lua"),
    Plain => ("plain", "txt"),
    JavaScript => ("javascript", "js"),
    TypeScript => ("typescript", "ts"),
    Jsx => ("jsx", "jsx"),
    Tsx => ("tsx", "tsx"),
    Html => ("html", "html"),
    Css => ("css", "css"),
    Scss => ("scss", "scss"),
    Python => ("python", "py"),
    C => ("c", "c"),
    Cpp => ("cpp", "cpp"),
    CSharp => ("csharp", "cs"),
    Java => ("java", "java"),
    Rust => ("rust", "rs"),
    Go => ("go", "go"),
    Php => ("php", "php"),
    Ruby => ("ruby", "rb"),
    Swift => ("swift", "swift"),
    Kotlin => ("kotlin", "kt"),
    Dart => ("dart", "dart"),
    Bash => ("bash", "sh"),
    PowerShell => ("powershell", "ps1"),
    Sql => ("sql", "sql"),
    R => ("r", "r"),
    Json => ("json", "json"),
    Jsonc => ("jsonc", "json"),
    Yaml => ("yaml", "yaml"),
    Toml => ("toml", "toml"),
    Xml => ("xml", "xml"),
    Ini => ("ini", "ini"),
    Markdown => ("markdown", "md"),
    Latex => ("latex", "tex"),
    Dockerfile => ("dockerfile", "dockerfile"),
    Makefile => ("makefile", "make"),
    CMake => ("cmake", "cmake"),
    Glsl => ("glsl", "glsl"),
    Hlsl => ("hlsl", "hlsl"),
    Wgsl => ("wgsl", "wgsl"),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TokenKind {
    Plain,
    Keyword,
    String,
    Number,
    Comment,
    Operator,
}

#[derive(Clone, Debug)]
pub(super) struct Token {
    pub range: Range<usize>,
    pub kind: TokenKind,
}

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(|| {
    let mut builder = two_face::syntax::extra_newlines().into_builder();
    builder.add(
        SyntaxDefinition::load_from_str(
            include_str!("../../../../assets/syntaxes/HLSL.sublime-syntax"),
            true,
            None,
        )
        .expect("Bundled HLSL syntax must be valid."),
    );
    builder.build()
});

static SCOPES: LazyLock<Vec<(Scope, TokenKind)>> = LazyLock::new(|| {
    use TokenKind::*;
    [
        ("comment", Comment),
        ("punctuation.definition.comment", Comment),
        ("string", String),
        ("punctuation.definition.string", String),
        ("constant.character", String),
        ("constant.numeric", Number),
        ("keyword.operator", Operator),
        ("keyword", Keyword),
        ("storage", Keyword),
        ("constant.language", Keyword),
        ("support.type", Keyword),
        ("support.function", Keyword),
        ("support.constant", Keyword),
        ("entity.name.tag", Keyword),
        ("entity.other.attribute-name", Keyword),
        ("markup.heading", Keyword),
        ("markup.bold", Keyword),
        ("markup.italic", String),
        ("punctuation.section.embedded", Operator),
    ]
    .into_iter()
    .map(|(scope, kind)| (Scope::new(scope).unwrap(), kind))
    .collect()
});

type CachedTokens = (CodeLanguage, String, Arc<Vec<Token>>);
static CACHE: Mutex<VecDeque<CachedTokens>> = Mutex::new(VecDeque::new());

pub(super) fn tokens(source: &str, language: CodeLanguage) -> Arc<Vec<Token>> {
    let mut cache = CACHE.lock().unwrap();
    if let Some(index) = cache
        .iter()
        .position(|(lang, text, _)| *lang == language && text == source)
    {
        let entry = cache.remove(index).unwrap();
        let tokens = entry.2.clone();
        cache.push_front(entry);
        return tokens;
    }
    drop(cache);
    let tokens = Arc::new(parse_tokens(source, language).unwrap_or_else(|| {
        let mut tokens = Vec::new();
        push_tokens(
            &mut tokens,
            source,
            0..source.len(),
            TokenKind::Plain,
            false,
        );
        tokens
    }));
    let mut cache = CACHE.lock().unwrap();
    cache.push_front((language, source.to_owned(), tokens.clone()));
    cache.truncate(32);
    tokens
}

fn kind(stack: &ScopeStack) -> TokenKind {
    static PUNCTUATION: LazyLock<Scope> = LazyLock::new(|| Scope::new("punctuation").unwrap());
    for scope in stack.scopes.iter().rev() {
        if let Some((_, kind)) = SCOPES
            .iter()
            .find(|(prefix, _)| prefix.is_prefix_of(*scope))
        {
            return *kind;
        }
    }
    // Literal punctuation inherits its enclosing string, comment, or number color.
    if stack
        .scopes
        .iter()
        .any(|scope| PUNCTUATION.is_prefix_of(*scope))
    {
        TokenKind::Operator
    } else {
        TokenKind::Plain
    }
}

fn parse_tokens(source: &str, language: CodeLanguage) -> Option<Vec<Token>> {
    if language == CodeLanguage::Plain {
        let mut tokens = Vec::new();
        push_tokens(
            &mut tokens,
            source,
            0..source.len(),
            TokenKind::Plain,
            false,
        );
        return Some(tokens);
    }
    let syntax = SYNTAXES
        .find_syntax_by_extension(language.extension())
        .expect("Every code language must have a bundled grammar.");
    let mut parser = ParseState::new(syntax);
    let mut stack = ScopeStack::new();
    let mut tokens = Vec::new();
    let mut offset = 0;
    let mut previous = TokenKind::Plain;
    let mut continuation = true;
    static LINE_COMMENT: LazyLock<Scope> = LazyLock::new(|| Scope::new("comment.line").unwrap());
    for line in LinesWithEndings::from(source) {
        let operations = parser.parse_line(line, &SYNTAXES).ok()?;
        for (text, operation) in ScopeRegionIterator::new(&operations, line) {
            stack.apply(operation).ok()?;
            let end = offset + text.len();
            let current = kind(&stack);
            continuation &= previous == current;
            let content_end = if current == TokenKind::Comment
                && stack
                    .scopes
                    .iter()
                    .any(|scope| LINE_COMMENT.is_prefix_of(*scope))
            {
                offset + text.trim_end_matches(['\r', '\n']).len()
            } else {
                end
            };
            push_tokens(
                &mut tokens,
                source,
                offset..content_end,
                current,
                continuation,
            );
            push_tokens(
                &mut tokens,
                source,
                content_end..end,
                TokenKind::Plain,
                false,
            );
            if !text.is_empty() {
                continuation = true;
            }
            previous = if content_end < end {
                TokenKind::Plain
            } else {
                current
            };
            offset = end;
        }
    }
    Some(tokens)
}

fn push_tokens(
    tokens: &mut Vec<Token>,
    source: &str,
    range: Range<usize>,
    kind: TokenKind,
    continuation: bool,
) {
    if range.is_empty() {
        return;
    }
    let literal = matches!(
        kind,
        TokenKind::String | TokenKind::Comment | TokenKind::Number
    );
    let word = |ch: char| ch.is_alphanumeric() || ch == '_';
    let text = &source[range.clone()];
    let mut start = range.start;
    for (byte, ch) in text.char_indices().skip(1).chain([(text.len(), '\0')]) {
        let end = range.start + byte;
        let first = source[start..].chars().next().unwrap();
        if end < range.end
            && (literal || word(first) == word(ch) && first.is_whitespace() == ch.is_whitespace())
        {
            continue;
        }
        if let Some(previous) = tokens.last_mut()
            && previous.range.end == start
            && previous.kind == kind
            && (literal && continuation
                || !literal && word(source[..start].chars().next_back().unwrap()) && word(first))
        {
            previous.range.end = end;
        } else {
            tokens.push(Token {
                range: start..end,
                kind,
            });
        }
        start = end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_kind(language: &'static str, source: &str, text: &str, expected: TokenKind) {
        let language = CodeLanguage::from_track_value(TrackValue::Enum(language)).unwrap();
        let spans = tokens(source, language);
        let mut end = 0;
        for token in spans.iter() {
            assert_eq!(token.range.start, end);
            assert!(
                source.is_char_boundary(token.range.start)
                    && source.is_char_boundary(token.range.end)
            );
            end = token.range.end;
        }
        assert_eq!(end, source.len());
        let start = source.find(text).unwrap();
        for token in spans
            .iter()
            .filter(|token| token.range.start < start + text.len() && start < token.range.end)
        {
            assert_eq!(token.kind, expected, "{language:?}: {text}: {token:?}");
        }
    }

    #[test]
    fn every_advertised_language_has_a_grammar_and_colors_real_syntax() {
        use TokenKind::*;
        let examples = [
            ("lua", "local value = 42", "local", Keyword),
            ("javascript", "const value = 42;", "const", Keyword),
            (
                "typescript",
                "interface Value { value: number }",
                "interface",
                Keyword,
            ),
            (
                "jsx",
                "const view = <Button label=\"Hi\" />;",
                "Button",
                Keyword,
            ),
            (
                "tsx",
                "const view: JSX.Element = <Button label=\"Hi\" />;",
                "Button",
                Keyword,
            ),
            ("html", "<div class=\"box\">Hello</div>", "div", Keyword),
            ("css", "body { color: red; }", "color", Keyword),
            ("scss", "$size: 42px; .box { width: $size; }", "42", Number),
            ("python", "def hello():\n    return 42", "return", Keyword),
            ("c", "int main(void) { return 42; }", "return", Keyword),
            (
                "cpp",
                "class Value { public: int value = 42; };",
                "class",
                Keyword,
            ),
            (
                "csharp",
                "public class Value { public int X = 42; }",
                "public",
                Keyword,
            ),
            (
                "java",
                "public class Value { int value = 42; }",
                "public",
                Keyword,
            ),
            ("rust", "pub fn value() -> u32 { 42 }", "pub", Keyword),
            ("go", "package main\nfunc main() {}", "func", Keyword),
            (
                "php",
                "<?php function value() { return 42; }",
                "return",
                Keyword,
            ),
            ("ruby", "def value\n  42\nend", "def", Keyword),
            ("swift", "let value: Int = 42", "let", Keyword),
            ("kotlin", "fun value(): Int = 42", "fun", Keyword),
            (
                "dart",
                "class Value { final int value = 42; }",
                "class",
                Keyword,
            ),
            ("bash", "if true; then echo \"Hi\"; fi", "then", Keyword),
            (
                "powershell",
                "function Get-Value { return 42 }",
                "return",
                Keyword,
            ),
            ("sql", "SELECT 42 FROM values_table;", "SELECT", Keyword),
            ("r", "value <- function() { 42 }", "function", Keyword),
            ("json", "{\"value\": 42}", "42", Number),
            (
                "jsonc",
                "{\"values\": [42, /* Note. */ 100]}",
                "Note.",
                Comment,
            ),
            ("yaml", "value: 42\n# Note.", "Note.", Comment),
            ("toml", "[values]\nvalue = 42", "42", Number),
            ("xml", "<value>42</value>", "value", Keyword),
            ("ini", "[values]\nvalue = 42\n; Note.", "Note.", Comment),
            (
                "markdown",
                "# Heading\nSome **bold** text.",
                "Heading",
                Keyword,
            ),
            ("latex", "\\section{Title}\nSome text.", "section", Keyword),
            (
                "dockerfile",
                "FROM ubuntu:24.04\nRUN echo hi",
                "FROM",
                Keyword,
            ),
            (
                "makefile",
                "all:\n\t@echo \"Hi\"\n# Note.",
                "Note.",
                Comment,
            ),
            (
                "cmake",
                "cmake_minimum_required(VERSION 3.20)",
                "cmake_minimum_required",
                Keyword,
            ),
            (
                "glsl",
                "void main() { gl_Position = vec4(1.0); }",
                "void",
                Keyword,
            ),
            (
                "hlsl",
                "float4 main() : SV_Target { return float4(1, 0, 0, 1); }",
                "float4",
                Keyword,
            ),
            (
                "wgsl",
                "@fragment fn main() -> @location(0) vec4f { return vec4f(1.0); }",
                "return",
                Keyword,
            ),
        ];
        assert_eq!(examples.len() + 1, CodeLanguage::VARIANTS.len());
        for (language, source, text, kind) in examples {
            assert_kind(language, source, text, kind);
        }
        assert_kind("plain", "local é = 42 -- Note.\n\"Text\"", "local", Plain);
    }

    #[test]
    fn adjacent_literals_keep_separate_identities_and_line_comments_exclude_newlines() {
        let source = "/* One. *//* Two. */";
        let comments = tokens(source, CodeLanguage::C);
        assert_eq!(
            comments
                .iter()
                .filter(|token| token.kind == TokenKind::Comment)
                .count(),
            2
        );
        let source = "-- One.\n-- Two.";
        let comments = tokens(source, CodeLanguage::Lua);
        assert!(
            comments
                .iter()
                .filter(|token| token.kind == TokenKind::Comment)
                .all(|token| !source[token.range.clone()].contains('\n'))
        );
        let strings = tokens("\"One.\"\"Two.\"", CodeLanguage::Python);
        assert!(
            strings
                .iter()
                .all(|token| token.range.end <= 6 || token.range.start >= 6)
        );
    }

    #[test]
    fn multiline_literals_interpolation_and_embedded_languages_keep_their_scopes() {
        use TokenKind::*;
        for (language, source, text, kind) in [
            (
                "python",
                "message = \"\"\"First.\nSecond.\"\"\"\nvalue = 42",
                "Second.",
                String,
            ),
            (
                "javascript",
                "const msg = `Olá ${42}`; /* First.\nSecond. */",
                "Olá",
                String,
            ),
            (
                "javascript",
                "const msg = `Olá ${42}`; /* First.\nSecond. */",
                "42",
                Number,
            ),
            (
                "javascript",
                "const msg = `Olá ${42}`; /* First.\nSecond. */",
                "Second.",
                Comment,
            ),
            (
                "html",
                "<script>const value = 42;</script>",
                "const",
                Keyword,
            ),
            (
                "tsx",
                "const view = <Button label=\"Olá\">{42}</Button>;",
                "Olá",
                String,
            ),
            (
                "jsonc",
                "{\n // First.\n \"text\": \"// Literal.\",\n \"values\": [42, /* Second. */ 100,],\n}",
                "Second.",
                Comment,
            ),
            (
                "jsonc",
                "{\"text\": \"// Literal.\"}",
                "// Literal.",
                String,
            ),
            (
                "hlsl",
                "// float4 is a type.\nfloat4 color;",
                "float4 is a type.",
                Comment,
            ),
            (
                "wgsl",
                "/* Outer. /* Inner. */ End. */\nvar value: i32 = 42;",
                "End.",
                Comment,
            ),
        ] {
            assert_kind(language, source, text, kind);
        }
    }
}
