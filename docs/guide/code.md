# Code

`Code` is a graphic object that displays source code with syntax highlighting,
selection, highlighting, and animated edits. Its content appears in the scene. It is not
executed as Lua. The editor's **Code** panel is a separate editing tool.

## Create and style a Code object

Create one with `s:code {code=..., language="lua"}` and attach it to the 2D world.
`size` sets the font size, `line_spacing` sets the space between lines, and `tab_size`
controls the visual width of tabs. Use `font=k.font(path)` to load a TTF or OTF included
in the project. The example includes Hack, the font these objects use by default in this
checkout, together with its licenses. This keeps rendering independent of the build
machine's paths.

`language` accepts the choices listed in [CodeOptions](../api/CodeOptions.md). It
controls tokenization and colors, but does not change the text. Adjust the appearance
with properties such as `keyword_color`, `string_color`, `number_color`,
`comment_color`, `operator_color`, `highlight_color`, and `dim_opacity`.

```lua
local code = s:code {
    code = "local value = 1\nprint(value)",
    language = "lua",
    size = 40,
    line_spacing = 1.2,
}
s:world_2d():add(code)
```

Long source can exceed the view. Unlike the editor panel, the graphic object has no
scrolling controls. Plan the font size, line count, and camera for the final video
resolution.

## Positions and ranges

A position is `{line,column}`. Both values start at 1. Columns count Unicode characters
rather than UTF-8 bytes. A tab counts as one character regardless of its visual width. A
range is `{{start_line,start_column},{end_line,end_column}}`, with an exclusive end.

Use these helpers to avoid calculating ranges by hand:

| Query | Result |
| --- | --- |
| `word(line,column,length?)` | A character range on one line. Without `length`, it runs to the end of the line. |
| `lines(first,last?)` | Full lines including both endpoints. Without `last`, it selects one line. |
| `find(text,match_case?,match_whole_word?)` | The first literal match, or nil. |
| `find_all(text,match_case?,match_whole_word?)` | A list of non-overlapping literal matches. |

Despite its name, `word` does not search for a word. You provide the column and length.
`find` and `find_all` do not use regular expressions. Search options default to false.
Whole-word matching treats letters, digits, and underscores as word characters, so
adjacent punctuation is allowed.

## Selection and highlighting

`select(ranges)` dims text outside the ranges while preserving syntax colors.
`highlight(ranges)` draws backgrounds with `highlight_color`. These effects are part of
the scene and appear in the export.

```lua
code:select(code:lines(2)):play(0.5)
code:highlight(code:find_all("value")):play(0.5)

code:select(nil):play(0.5)
code:highlight({}):play(0.5)
```

Pass `nil` or `{}` to clear a selection or highlight. Lists can combine ranges and
search results, including nested lists. Missing searches inside a list are ignored.

## Animate edits

`insert(position,text)` inserts text at a position. `replace(ranges,text)` replaces
ranges, and `erase(ranges)` removes them. `append(text)` and `prepend(text)` add lines
at the end or beginning, adding a line break when needed. The historical alias `prepand`
still works, but use `prepend` in new code.

```lua
local range = code:find("1")
if range then
    code:replace(range, "42"):play(1)
end

code:append("local doubled = value * 2"):play(1)
```

Tokens affected by an edit fade as whole words, while unchanged tokens keep their
identity. Use `morph` to match individual characters across blocks.

Each method returns a definition. Call `play` to schedule the edit. Ranges are checked
against the text at execution time. Overlapping ranges, or a missing single search
result passed to `replace` or `erase`, cause errors. After replacing text, check the
result before reusing old coordinates.

## Ranges do not follow edits

A query returns coordinates based on the text when the edit is scheduled. It does not
repeat the search in the future. Search again after an edit. Selections and highlights
also keep their coordinates, so clear them before restructuring text if you do not want
the old ranges to remain.

```lua
code:replace(code:find("value"), "answer"):play(1)
code:highlight(code:find_all("answer")):play(0.5)
```

Reusing an edit definition with fixed coordinates after the text changes may affect a
different range or fail. This differs from a property `to` animation, whose destination
keeps the same meaning after its first run.

## Morph and fade

`morph(text,group?)` moves matching characters and fades in or out new ones.
`group=true` matches words and symbols as groups. The default is false. `fade(text)`
crossfades the entire text block. Both return definitions that use
`:play(duration,easing)`.

<<< ../../examples/lua/code/scenes/intro.lua

The [Code project](../examples/code.md) demonstrates highlighting, replacement,
appending, selection, and morph. The code shown is the object being demonstrated.

## Errors and limits

Check line and column indices, exclusive ends, nil search results, and overlapping
ranges. Searches are literal and do not follow later edits. If the layout looks wrong,
check the font, `size`, `line_spacing`, tabs, and framing. The graphic object does not
have the editor panel's controls.

See [Code](../api/Code.md), [CodeOptions](../api/CodeOptions.md),
[CodePosition](../api/CodePosition.md), [CodeRange](../api/CodeRange.md), and
[CodeRangeList](../api/CodeRangeList.md).
