---
pageClass: guide-page
---

# Code

Code displays a source snippet in your scene. You can color its syntax, emphasize a
passage, edit individual lines, or transition to another version of the program.
The displayed source is part of the video. It is not executed as Lua.

The examples use short greetings so each operation is easy to follow. Every scene
can replace `scenes/intro.lua`. Previews use 960 × 540 at 30 FPS.

## Create a code block

Set `code` to the text you want to display, choose its language, and add it to the 2D
world. The default font size is enough for this short greeting.

<GuideDemo name="code-hello" caption="A Lua greeting at the center of the scene." still>

<!-- demo: code-hello -->
<<< ./projects/code/scenes/01-code-hello.lua

</GuideDemo>

A static code image needs no wait or animation. Add a duration only when something
changes over time.

### Multiple lines

Use `\n` inside a quoted string, or a Lua long string for several lines. Indentation
inside the long string is preserved:

```lua
code = [=[
local message = "Hello, Kinematic!"
print(message)]=]
```

Several examples below use a two-line version with a variable named `text` or `message`. The first line defines
the message. The second prints it. The Code object displays both statements without
running them. Changing a Lua variable used to construct this string does not update the
display automatically. Use an edit or transition when the text should change.

## Language and appearance

### Syntax language

`language` chooses the bundled syntax coloring. It does not translate or execute the
source. For another language, change both the displayed source and the language name.
Here is the same greeting in JavaScript:

```lua
local code = s:code {
    code = 'console.log("Hello, Kinematic!");',
    language = "javascript",
}
```

Use exact lowercase names such as `"lua"`, `"python"`, `"javascript"`, `"rust"`, or
`"json"`. The default, `"plain"`, draws the source using `fill`, without syntax coloring.
The [language options](../api/CodeOptions.md#property-language) list all supported values.
Abbreviations such as `"js"` and case variants such as `"Lua"` are rejected. No extra
grammar installation is needed.

### Font, size, and spacing

`size` sets the font size in local units. `line_spacing` multiplies the font's natural
line spacing. `tab_size` controls the visual tab stops. It does not change literal
spaces in the source.

```lua
local code = s:code {
    code = 'local function greet()\n\tprint("Hello, Kinematic!")\nend',
    language = "lua",
    size = 32,
    line_spacing = 1.2,
    tab_size = 2,
}
```

The indentation above is a tab character. A tab advances to the next tab stop, while
four typed spaces remain four spaces regardless of `tab_size`. Reduce `size` when a
line is too long: Code does not wrap or scroll internally.

To use a project font, include a TTF or OTF and its license, then pass the resource in
the constructor:

```lua
font = k.font("assets/your-mono-font.ttf"),
```

The default is Hack Mono. The [complete Code example](../examples/code.md) includes a
font and its licenses. Font resources stay fixed within an applied revision. Size,
spacing, and object transforms can be animated.

### Syntax colors

Each syntax category has its own color. Customize the categories you need rather than
building a new palette for every example.

```lua
keyword_color = k.color("#4080FF"),
number_color = k.color("#FFBF40"),
```

| Property | Colors |
| --- | --- |
| `fill` | Ordinary text and tokens without a special category. |
| `keyword_color` | Keywords and declarations recognized by the language. |
| `string_color` | String literals. |
| `number_color` | Numeric literals. |
| `comment_color` | Comments. |
| `operator_color` | Operators and punctuation recognized by the grammar. |

These are ordinary color properties. For example, `code.number_color:to(color):play(1)`
changes the numeric syntax color over one second. `fill` does not tint all the other
categories. Background emphasis uses `highlight_color`, described below.

## Choose part of the source

Ranges tell an operation which characters to use. The same range can be passed to
`select`, `highlight`, `replace`, or `erase`.

### Positions and ranges

A position is `{line, column}`, starting at `{1, 1}`. Columns count Unicode characters,
not UTF-8 bytes. A tab counts as one source character even when it spans several visual
columns.

A range is `{{start_line, start_column}, {end_line, end_column}}`. The start is included.
The end is excluded. In the greeting, `print` starts at column 1 and the opening
parenthesis is at column 6, so `{{1, 1}, {1, 6}}` includes only the function name:

<GuideDemo name="code-ranges" caption="The range covers print and stops before the opening parenthesis.">

<!-- focus: code-ranges -->
```lua
code:highlight({ { 1, 1 }, { 1, 6 } }):play(1)
```

<template #source>

<!-- demo: code-ranges -->
<<< ./projects/code/scenes/02-code-ranges.lua

</template>

</GuideDemo>

Change the end column to `7` to include the opening parenthesis. A range can cross line breaks, but
both endpoints must be valid. A position one character past the line end is valid for
an insertion. A zero-based or out-of-bounds position is not.

### Word and line helpers

`word(line, column, length?)` builds a range from coordinates. It does not search for a
word: the length tells it how many characters to include. Without a length, it continues
to the end of the line.

`lines(first, last?)` includes both numbered lines. Omit `last` for one line. If another
line follows, the range includes the selected line's newline, which is useful for
erasing a whole line.

```lua
code:select(code:word(1, 7, 7)):play(1)
s:wait(1)
code:select(code:lines(2)):play(1)
```

`word(1, 7, 7)` selects `message` in the first line. `lines(2)` selects the second line.
Use these helpers when the location matters. Use a search when you know the text.

### Find text

`find(text)` returns the first literal match or `nil`. `find_all(text)` returns a list
of non-overlapping matches. Neither query changes the display until you pass its result
to a visual operation.

```lua
code:highlight(code:find("message")):play(1)
s:wait(1)
code:highlight(code:find_all("message")):play(1)
```

Both methods accept `match_case` and `match_whole_word` booleans, which default to false:

```lua
code:find("message", true, true)
code:find_all("message", true, true)
```

With these options, `Message` and `message_count` do not match `message`. Letters, digits,
and underscores form words. Adjacent punctuation is allowed. Searches are literal, not
regular expressions, and an empty search string is rejected.

## Emphasize the code

### Selection

`select(ranges)` dims everything outside the selected text, preserving syntax colors.
Use it when the viewer should focus on one part of a larger block.

<GuideDemo name="code-select" caption="Both uses of text stay bright while the surrounding source dims.">

<!-- focus: code-select -->
```lua
code:select(code:find_all("text")):play(1)
```

<template #source>

<!-- demo: code-select -->
<<< ./projects/code/scenes/03-code-select.lua

</template>

</GuideDemo>

`dim_opacity` controls the opacity of unselected text and defaults to `0.25`. Set it to
`0` to hide the surrounding text or choose a larger value for gentler emphasis. Selection
does not change the source or its bounds.

Clear the selection with either form:

```lua
code:select(nil):play(1)
code:select({}):play(1)
```

### Highlight backgrounds

`highlight(ranges)` adds colored backgrounds without dimming the rest of the block.
Use it to connect occurrences while keeping their context equally readable.

<GuideDemo name="code-highlight" caption="The declaration and use receive backgrounds. The rest of the source stays bright.">

<!-- focus: code-highlight -->
```lua
code:highlight(code:find_all("text")):play(1)
```

<template #source>

<!-- demo: code-highlight -->
<<< ./projects/code/scenes/04-code-highlight.lua

</template>

</GuideDemo>

Set `highlight_color` to customize the background, including its alpha. Clear backgrounds
with `code:highlight(nil):play(1)` or `code:highlight({}):play(1)`. Selection and highlights
are independent, and both appear in the exported video.

Methods also accept lists that combine queries:

```lua
code:select({code:lines(2), code:find("Kinematic")}):play(1)
```

Nested results such as `find_all` can go directly into the list. Missing searches inside
a list are ignored. The corresponding property handles are `code.selection` and
`code.highlights`.

## Edit the source

Each editing method returns an animation definition. Preparing it does not change the
source. `play(1)` schedules the change for one second. The examples use the default
easing. Omit the duration only when you intentionally want an instant change.

Untouched tokens keep their identity during an edit. Changed tokens fade as whole words.
They are not typed letter by letter. Use morph when letters themselves should move.

### Insert

`insert(position, text)` adds text before the character at the given position. In this
shorter greeting, column 13 is the exclamation mark:

<GuideDemo name="code-insert" caption="The inserted text turns Hello! into Hello, Kinematic!.">

<!-- focus: code-insert -->
```lua
code:insert({ 1, 13 }, ", Kinematic"):play(1)
```

<template #source>

<!-- demo: code-insert -->
<<< ./projects/code/scenes/05-code-insert.lua

</template>

</GuideDemo>

The existing text stays in the statement. Include `\n` in the inserted string when the
addition should create a new line.

### Replace

`replace(range, text)` changes the chosen text. Find the name instead of counting its
columns:

<GuideDemo name="code-replace" caption="Only world changes to Kinematic. The greeting and punctuation stay.">

<!-- focus: code-replace -->
```lua
code:replace(code:find("world"), "Kinematic"):play(1)
```

<template #source>

<!-- demo: code-replace -->
<<< ./projects/code/scenes/06-code-replace.lua

</template>

</GuideDemo>

`find` can return `nil` when the text is absent. A missing single edit range raises an
error, so check the result when working with source that may vary. The fixed greeting
above always contains the name.

Pass `find_all` to rename every occurrence in one operation:

<GuideDemo name="code-replace-all" caption="Both occurrences of text become greeting in one edit.">

<!-- focus: code-replace-all -->
```lua
code:replace(code:find_all("text"), "greeting"):play(1)
```

<template #source>

<!-- demo: code-replace-all -->
<<< ./projects/code/scenes/07-code-replace-all.lua

</template>

</GuideDemo>

Each range receives the same replacement. The operation uses the original coordinates
for all its ranges, so you do not need to compensate for a longer replacement earlier
in the source. Overlapping edit ranges are rejected.

### Append and prepend

`append(text)` adds a new last line. It inserts a separating newline when needed, so the
string below does not need to start with `\n`:

```lua
code:append("print(message)"):play(1)
```

`prepend(text)` does the reverse: it adds a new first line and supplies a separating
newline when needed. Start with the print statement and add its definition:

```lua
code:prepend('local message = "Hello, Kinematic!"'):play(1)
```

Both methods accept multiple lines. `prepand` is a compatibility alias of `prepend`.
Use `prepend` in new scenes.

### Erase

`erase(ranges)` removes text while keeping the Code object alive. Here it removes the
print statement from a two-line program:

<GuideDemo name="code-erase" caption="The print line disappears, leaving the message definition.">

<!-- focus: code-erase -->
```lua
code:erase(code:lines(2)):play(1)
```

<template #source>

<!-- demo: code-erase -->
<<< ./projects/code/scenes/08-code-erase.lua

</template>

</GuideDemo>

A complete-line range removes the line and its separator rather than leaving a blank
row. Use `remove` when the entire scene object should end its lifetime instead.

### Query again after an edit {#ranges-do-not-follow-edits}

A query returns fixed coordinates. It does not follow a word when an earlier edit makes
that word longer or moves it to another line. Selection and highlight ranges have the
same behavior.

Clear old emphasis before restructuring the source, then search the new text:

```lua
code:highlight(nil):play(1)
code:replace(code:find_all("message"), "greeting"):play(1)
code:highlight(code:find_all("greeting")):play(1)
```

The last query reads the source after the replacement. Reusing the earlier ranges for
`message` would not select the full longer name. Schedule dependent text edits in
sequence. Use `all` for independent Code objects or properties such as opacity.

## Morph and fade

The two morph videos change `print("Hello, world!")` into an `add` function and a
call that displays its result. Only the grouping flag changes. The fade video then
shows a sequence of complete algorithms: Fibonacci, factorial, greatest common
divisor, and a primality check.

### Match characters: group = false

<GuideDemo name="code-morph-characters" caption="Individual matching characters move as a greeting becomes a function definition.">

<!-- focus: code-morph-characters -->
```lua
code:morph([=[local function add(a, b)
    return a + b
end

print(add(1, 1)) -- result: 2]=]):play(1)
```

<template #source>

<!-- demo: code-morph-characters -->
<<< ./projects/code/scenes/09-code-morph-characters.lua

</template>

</GuideDemo>

### Match words: group = true

<GuideDemo name="code-morph-groups" caption="The changed word fades as a group while matching words and symbols move together.">

<!-- focus: code-morph-groups -->
```lua
code:morph([=[local function add(a, b)
    return a + b
end

print(add(1, 1)) -- result: 2]=], true):play(1)
```

<template #source>

<!-- demo: code-morph-groups -->
<<< ./projects/code/scenes/10-code-morph-groups.lua

</template>

</GuideDemo>

### Fade between blocks

<GuideDemo name="code-fade" caption="The entire old snippet crossfades into the new snippet.">

<!-- focus: code-fade -->
```lua
code:fade([=[local function factorial(n)
    if n < 2 then return 1 end
    return n * factorial(n - 1)
end

print(factorial(5)) -- Result: 120.]=]):play(1)
```

<template #source>

<!-- demo: code-fade -->
<<< ./projects/code/scenes/11-code-fade.lua

</template>

</GuideDemo>

`false` matches individual characters. `true` matches complete words and symbols,
so changing a word does not reuse its letters independently. `fade` has no `group`
argument and crossfades both complete blocks.

A targeted edit such as `replace` keeps untouched tokens intact. Choose that for a
local source edit, and use morph when the movement of matching content helps explain
the change. Leave room for both layouts during any transition.

## Positioning and common issues

Code has the usual 2D transforms, opacity, filters, bounds, and anchors. It is centered
by default. Use `left_to` to align the block by its left edge, and `box_size()` to read
its dimensions. See [coordinates and anchors](coordinates.md) for framing without
building a separate positioning system.

`code.code:get()` reads the source at the current scene time. `code.code:set(text)`
replaces it immediately during construction. Use an edit, morph, or fade for a visible
transition.

| Symptom | Check |
| --- | --- |
| All text has one color. | The default language is `plain`. Choose the matching syntax language. |
| The wrong characters change. | Use one-based Unicode columns and an exclusive end. Query again after earlier edits. |
| An edit reports a missing range. | `find` returned `nil`. Check the text and search options. |
| An edit reports overlapping ranges. | Do not combine a whole line with a word inside that same line. |
| A transition is instant. | Pass a positive duration to `play`. |
| Source extends outside the frame. | Reduce size or adjust framing. There is no internal wrapping or scrolling. |

For full signatures, see [Code](../api/Code.md), [CodeOptions](../api/CodeOptions.md),
[CodePosition](../api/CodePosition.md), and [CodeRange](../api/CodeRange.md).
The [complete Code example](../examples/code.md) combines these operations in one project.
