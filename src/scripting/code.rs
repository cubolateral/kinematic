use super::{
    animation::AnimationDefinition,
    methods::{LuaMethod, text_transition},
    runtime::ObjectHandle,
};
use crate::core::{
    Task, TrackTarget, TrackValue,
    objects::{CodeEditState, CodeShape},
    types::{CodePosition, CodeRange, CodeRanges, code_interval, code_offset, code_position},
};
use mlua::{FromLuaMulti, IntoLua, Lua, MultiValue, Result, Table, Value};

pub(super) enum CodeAction {
    Morph(String, bool),
    Fade(String),
    Insert(CodePosition, String),
    Replace(CodeRanges, String),
    Erase(CodeRanges),
    Append(String),
    Prepend(String),
    Select(CodeRanges),
    Highlight(CodeRanges),
}

pub(super) fn resolve(object: &ObjectHandle, action: &CodeAction) -> Result<AnimationDefinition> {
    let TrackValue::String(mut source) = object.property("code")?.value()? else {
        unreachable!()
    };
    let mut edits = Vec::new();
    let mut replacement = String::new();
    match action {
        CodeAction::Select(ranges) | CodeAction::Highlight(ranges) => {
            for (ranges, _) in &ranges.0 {
                for range in ranges {
                    code_interval(&source, *range).map_err(mlua::Error::runtime)?;
                }
            }
            let name = if matches!(action, CodeAction::Select(_)) {
                "selection"
            } else {
                "highlights"
            };
            return Ok(AnimationDefinition::target(
                object.clone(),
                object.property(name)?.target,
                TrackValue::CodeRanges(ranges.clone()),
            ));
        }
        CodeAction::Morph(text, _) | CodeAction::Fade(text) => source.clone_from(text),
        CodeAction::Insert(position, text) => {
            edits.push([*position, *position]);
            replacement.clone_from(text);
        }
        CodeAction::Append(text) => {
            let end = code_position(&source, source.len());
            edits.push([end, end]);
            if !source.is_empty() && !source.ends_with('\n') && !text.is_empty() {
                replacement.push('\n');
            }
            replacement.push_str(text);
        }
        CodeAction::Prepend(text) => {
            edits.push([[1, 1], [1, 1]]);
            replacement.clone_from(text);
            if !source.is_empty() && !text.ends_with('\n') && !text.is_empty() {
                replacement.push('\n');
            }
        }
        CodeAction::Replace(ranges, text) => {
            edits.extend(
                ranges
                    .0
                    .iter()
                    .flat_map(|(ranges, _)| ranges.iter())
                    .copied(),
            );
            replacement.clone_from(text);
        }
        CodeAction::Erase(ranges) => edits.extend(
            ranges
                .0
                .iter()
                .flat_map(|(ranges, _)| ranges.iter())
                .copied(),
        ),
    }
    edits.sort_unstable();
    edits.dedup();
    let intervals = edits
        .iter()
        .map(|range| code_interval(&source, *range).map_err(mlua::Error::runtime))
        .collect::<Result<Vec<_>>>()?;
    if intervals
        .windows(2)
        .any(|pair| pair[0].end > pair[1].start || pair[0].start == pair[1].start)
    {
        return Err(mlua::Error::runtime("Code edits cannot overlap."));
    }
    for interval in intervals.into_iter().rev() {
        source.replace_range(interval, &replacement);
    }
    let active = !matches!(action, CodeAction::Morph(..) | CodeAction::Fade(_));
    let mut tasks = vec![text_transition::<CodeShape>(
        object.entity,
        source,
        matches!(action, CodeAction::Morph(..)),
        matches!(action, CodeAction::Morph(_, true)),
    )];
    for (info, value) in [
        (
            CodeEditState::active_property().info(),
            TrackValue::Bool(active),
        ),
        (
            CodeEditState::ranges_property().info(),
            TrackValue::CodeRanges(CodeRanges::new(edits)),
        ),
        (
            CodeEditState::replacement_property().info(),
            TrackValue::String(replacement),
        ),
    ] {
        tasks.push(Task::RuntimeTween {
            entity: object.entity,
            target: TrackTarget::property(std::any::TypeId::of::<CodeEditState>(), info),
            from: value.clone(),
            to: value,
            duration: 0.0,
            easing: Default::default(),
            mask: 1,
            relative: 0,
            implicit: true,
            rotation: None,
        });
    }
    AnimationDefinition::from_task(object.clone(), Task::All(tasks))
}

fn sequence(value: Value) -> Result<Vec<Value>> {
    let Value::Table(table) = value else {
        return Err(mlua::Error::runtime(
            "Code positions and ranges must be indexed tables.",
        ));
    };
    let length = table.raw_len();
    if table.clone().pairs::<Value, Value>().any(|entry| !matches!(entry, Ok((Value::Integer(key), _)) if key >= 1 && key as usize <= length)) {
        return Err(mlua::Error::runtime("Code positions and ranges require contiguous indexed values."));
    }
    (1..=length)
        .map(|index| {
            let value = table.raw_get::<Value>(index)?;
            if value.is_nil() {
                return Err(mlua::Error::runtime(
                    "Code positions and ranges cannot contain holes.",
                ));
            }
            Ok(value)
        })
        .collect()
}

fn position(value: Value) -> Result<CodePosition> {
    let entries = sequence(value)?;
    let [Value::Integer(line), Value::Integer(column)] = entries.as_slice() else {
        return Err(mlua::Error::runtime(
            "Code position requires two positive integers {line, column}.",
        ));
    };
    if *line <= 0 || *column <= 0 {
        return Err(mlua::Error::runtime(
            "Code positions start at line 1, column 1.",
        ));
    }
    Ok([*line as usize, *column as usize])
}

fn range(value: Value) -> Result<CodeRange> {
    let entries = sequence(value)?;
    if entries.len() != 2 {
        return Err(mlua::Error::runtime(
            "Code range requires {start, exclusive_end}.",
        ));
    }
    let start = position(entries[0].clone())?;
    let end = position(entries[1].clone())?;
    if end < start {
        return Err(mlua::Error::runtime("Code range end precedes its start."));
    }
    Ok([start, end])
}

pub(super) fn parse_ranges(_lua: &Lua, value: Value) -> Result<CodeRanges> {
    fn collect(value: Value, ranges: &mut Vec<CodeRange>, depth: usize) -> Result<()> {
        if value.is_nil() {
            return Ok(());
        }
        if depth > 64 {
            return Err(mlua::Error::runtime(
                "Code range nesting exceeds 64 levels.",
            ));
        }
        let Value::Table(table) = &value else {
            return Err(mlua::Error::runtime(
                "Expected a code range or a list of ranges.",
            ));
        };
        if let Value::Table(first) = table.raw_get::<Value>(1)?
            && matches!(
                first.raw_get::<Value>(1)?,
                Value::Integer(_) | Value::Number(_)
            )
        {
            ranges.push(range(value)?);
            return Ok(());
        }
        let mut entries = Vec::new();
        for entry in table.clone().pairs::<Value, Value>() {
            let (key, value) = entry?;
            let Value::Integer(key) = key else {
                return Err(mlua::Error::runtime(
                    "Code range lists require positive integer keys.",
                ));
            };
            if key < 1 {
                return Err(mlua::Error::runtime(
                    "Code range lists require positive integer keys.",
                ));
            }
            entries.push((key, value));
        }
        entries.sort_unstable_by_key(|(key, _)| *key);
        for (_, value) in entries {
            collect(value, ranges, depth + 1)?;
        }
        Ok(())
    }
    let mut ranges = Vec::new();
    collect(value, &mut ranges, 0)?;
    Ok(CodeRanges::new(ranges))
}

fn range_to_lua(lua: &Lua, range: CodeRange) -> Result<Table> {
    lua.create_sequence_from(
        range
            .into_iter()
            .map(|point| lua.create_sequence_from(point))
            .collect::<Result<Vec<_>>>()?,
    )
}

pub(super) fn ranges_to_lua(lua: &Lua, ranges: &CodeRanges) -> Result<Value> {
    let ranges = ranges
        .0
        .iter()
        .flat_map(|(ranges, weight)| ranges.iter().filter(move |_| *weight > 0.0))
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    lua.create_sequence_from(
        ranges
            .into_iter()
            .map(|r| range_to_lua(lua, r))
            .collect::<Result<Vec<_>>>()?,
    )?
    .into_lua(lua)
}

fn arguments<T: FromLuaMulti>(lua: &Lua, args: MultiValue, count: usize) -> Result<T> {
    if args.len() != count {
        return Err(mlua::Error::runtime(
            "Incorrect number of code method arguments.",
        ));
    }
    T::from_lua_multi(args, lua)
}

fn prepare(lua: &Lua, object: &ObjectHandle, action: CodeAction) -> Result<Value> {
    object.runtime()?.construction()?;
    AnimationDefinition::code(object.clone(), action).into_lua(lua)
}

fn coordinate(value: Value, zero: bool) -> Result<usize> {
    match value {
        Value::Integer(value) if value >= if zero { 0 } else { 1 } => {
            usize::try_from(value).map_err(mlua::Error::external)
        }
        _ => Err(mlua::Error::runtime(
            "Code coordinates require positive integers; lengths may be zero.",
        )),
    }
}

fn matches(lua: &Lua, object: &ObjectHandle, args: MultiValue) -> Result<Vec<CodeRange>> {
    let count = args.len();
    if !(1..=3).contains(&count) {
        return Err(mlua::Error::runtime(
            "Code search requires text and optional match_case and match_whole_word booleans.",
        ));
    }
    let (value, match_case, match_whole_word): (Value, Value, Value) = arguments(lua, args, count)?;
    let boolean = |value| match value {
        Value::Nil => Ok(false),
        Value::Boolean(value) => Ok(value),
        _ => Err(mlua::Error::runtime(
            "Code search options must be booleans.",
        )),
    };
    let match_case = boolean(match_case)?;
    let match_whole_word = boolean(match_whole_word)?;
    let Value::String(text) = value else {
        return Err(mlua::Error::runtime("Code search text must be a string."));
    };
    let text = text.to_str()?;
    if text.is_empty() {
        return Err(mlua::Error::runtime("Code search text cannot be empty."));
    }
    let TrackValue::String(source) = object.property("code")?.value()? else {
        unreachable!()
    };
    let word = |ch: char| ch.is_alphanumeric() || ch == '_';
    let mut result = Vec::new();
    let mut previous_end = 0;
    for (start, _) in source.char_indices() {
        if start < previous_end {
            continue;
        }
        let mut end = start;
        let mut chars = source[start..].char_indices();
        let found = text.chars().all(|expected| {
            let Some((offset, ch)) = chars.next() else {
                return false;
            };
            end = start + offset + ch.len_utf8();
            if match_case {
                ch == expected
            } else {
                ch.to_lowercase().eq(expected.to_lowercase())
            }
        });
        if !found
            || (match_whole_word
                && ((text.chars().next().is_some_and(word)
                    && source[..start].chars().next_back().is_some_and(word))
                    || (text.chars().next_back().is_some_and(word)
                        && source[end..].chars().next().is_some_and(word))))
        {
            continue;
        }
        result.push([code_position(&source, start), code_position(&source, end)]);
        previous_end = end;
    }
    Ok(result)
}

macro_rules! code_methods {
    ($(#[doc = $documentation:literal]
        fn $name:ident($lua:ident, $object:ident, $args:ident; $($param:ident: $ty:literal),*) -> $result:literal $body:block
    )*) => {
        $(pub(super) fn $name($lua: &Lua, $object: &ObjectHandle, $args: MultiValue) -> Result<Value> $body)*

        pub(super) const METHODS: &[LuaMethod] = &[
            $(LuaMethod {
                name: stringify!($name),
                documentation: $documentation,
                annotation: concat!(
                    $("---@param ", stringify!($param), " ", $ty, "\n",)*
                    "---@return ", $result, "\nfunction Native:", stringify!($name),
                    "(", stringify!($($param),*), ") end\n"
                ),
                invoke: $name,
            },)*
        ];
    };
}

code_methods! {
    #[doc = r#"Appends code on a new last line. Adds a line break when needed and fades in the entire addition, preserving existing tokens.

```lua
code:append('print(value)'):play(1)
```"#]
    fn append(lua, object, args; text: "string") -> "Kinematic.Animation" {
        let value: Value = arguments(lua, args, 1)?;
        let Value::String(text) = value else {
            return Err(mlua::Error::runtime("Code text must be a string."));
        };
        prepare(lua, object, CodeAction::Append(text.to_str()?.to_owned()))
    }

    #[doc = r#"Prepends code on a new first line. Adds a line break when needed and fades in the entire addition, preserving existing tokens.

```lua
code:prepend('local value = 42'):play(1)
```"#]
    fn prepend(lua, object, args; text: "string") -> "Kinematic.Animation" {
        let value: Value = arguments(lua, args, 1)?;
        let Value::String(text) = value else {
            return Err(mlua::Error::runtime("Code text must be a string."));
        };
        prepare(lua, object, CodeAction::Prepend(text.to_str()?.to_owned()))
    }

    #[doc = r#"Alias of prepend, adding code on a new first line with a fade.

```lua
code:prepand('local value = 42'):play(1)
```"#]
    fn prepand(lua, object, args; text: "string") -> "Kinematic.Animation" {
        prepend(lua, object, args)
    }

    #[doc = r#"Selects characters on one line: word(line, column, length). Line and column start at 1. Omit length to select through the end of the line. The result can be passed to select, highlight, replace, or erase.

```lua
-- Three characters starting at column 3 of line 1.
code:select(code:word(1, 3, 3)):play(1)
-- Column 3 through the end of line 1.
code:highlight(code:word(1, 3)):play(1)
```"#]
    fn word(lua, object, args; line: "integer", column: "integer", length: "integer|nil") -> "Kinematic.CodeRange" {
        let count = args.len();
        if !(2..=3).contains(&count) {
            return Err(mlua::Error::runtime(
                "Word requires line, column, and an optional length.",
            ));
        }
        let (line, column, length): (Value, Value, Option<Value>) = arguments(lua, args, count)?;
        let start = [coordinate(line, false)?, coordinate(column, false)?];
        let TrackValue::String(source) = object.property("code")?.value()? else {
            unreachable!()
        };
        code_offset(&source, start).map_err(mlua::Error::runtime)?;
        let end = match length {
            Some(length) => start[1]
                .checked_add(coordinate(length, true)?)
                .ok_or_else(|| mlua::Error::runtime("Code word length overflows."))?,
            None => {
                source
                    .split('\n')
                    .nth(start[0] - 1)
                    .unwrap()
                    .trim_end_matches('\r')
                    .chars()
                    .count()
                    + 1
            }
        };
        let range = [start, [start[0], end]];
        code_interval(&source, range).map_err(mlua::Error::runtime)?;
        range_to_lua(lua, range)?.into_lua(lua)
    }

    #[doc = r#"Selects whole lines: lines(first, last). Both line numbers start at 1 and are included. Omit last to select one line. The result can be passed to select, highlight, replace, or erase.

```lua
code:select(code:lines(1, 3)):play(1)
code:highlight(code:lines(2)):play(1)
code:select({code:find('1'), code:find_all('oi'), code:lines(1)}):play(1)
```"#]
    fn lines(lua, object, args; first: "integer", last: "integer|nil") -> "Kinematic.CodeRange" {
        let count = args.len();
        if !(1..=2).contains(&count) {
            return Err(mlua::Error::runtime(
                "Lines requires a first line and an optional last line.",
            ));
        }
        let (first, last): (Value, Option<Value>) = arguments(lua, args, count)?;
        let first = coordinate(first, false)?;
        let last = last
            .map(|value| coordinate(value, false))
            .transpose()?
            .unwrap_or(first);
        if last < first {
            return Err(mlua::Error::runtime("Last code line precedes the first."));
        }
        let TrackValue::String(source) = object.property("code")?.value()? else {
            unreachable!()
        };
        code_offset(&source, [first, 1]).map_err(mlua::Error::runtime)?;
        code_offset(&source, [last, 1]).map_err(mlua::Error::runtime)?;
        let end = if last < source.split('\n').count() {
            [last + 1, 1]
        } else {
            code_position(&source, source.len())
        };
        range_to_lua(lua, [[first, 1], end])?.into_lua(lua)
    }

    #[doc = r#"Moves matching characters and fades inserted or removed characters. Group matches whole words and individual symbols instead. Default: false. Preparing leaves the timeline unchanged.

```lua
code:morph('local value = 42\nprint(value)'):play(1)
```"#]
    fn morph(lua, object, args; text: "string", group: "boolean?") -> "Kinematic.Animation" {
        let count = args.len();
        if !(1..=2).contains(&count) {
            return Err(mlua::Error::runtime("Morph requires text and an optional group boolean."));
        }
        let (value, group): (Value, Value) = arguments(lua, args, count)?;
        let Value::String(text) = value else {
            return Err(mlua::Error::runtime("Code text must be a string."));
        };
        let group = match group {
            Value::Nil => false,
            Value::Boolean(group) => group,
            _ => return Err(mlua::Error::runtime("Morph group must be a boolean.")),
        };
        prepare(lua, object, CodeAction::Morph(text.to_str()?.to_owned(), group))
    }

    #[doc = r#"Crossfades the current code into a new block.

```lua
code:fade('print(42)'):play(1)
```"#]
    fn fade(lua, object, args; text: "string") -> "Kinematic.Animation" {
        let value: Value = arguments(lua, args, 1)?;
        let Value::String(text) = value else {
            return Err(mlua::Error::runtime("Code text must be a string."));
        };
        prepare(lua, object, CodeAction::Fade(text.to_str()?.to_owned()))
    }

    #[doc = r#"Inserts text at a one-based {line, column} position. Affected tokens fade as whole words; intact tokens keep their identity. Use morph for letter matching. Columns count Unicode characters, including each tab as one character. The position is checked against the code at each play.

```lua
-- Inserts before the first character of the second line.
code:insert({2, 1}, '-- Result.\n'):play(1)
```"#]
    fn insert(lua, object, args; position: "Kinematic.CodePosition", text: "string") -> "Kinematic.Animation" {
        let (point, text): (Value, Value) = arguments(lua, args, 2)?;
        let Value::String(text) = text else {
            return Err(mlua::Error::runtime("Code text must be a string."));
        };
        prepare(
            lua,
            object,
            CodeAction::Insert(position(point)?, text.to_str()?.to_owned()),
        )
    }

    #[doc = r#"Replaces one or several text intervals with the same text. Use find, find_all, word, or lines to choose the intervals; their results can be mixed in a list. Changed tokens fade as whole words. Edits use the code at play time. Overlapping intervals or a missing single search result raise an error.

```lua
-- Replaces the first occurrence.
code:replace(code:find('value'), 'answer'):play(1)
-- Replaces every occurrence of either name.
code:replace({code:find_all('foo'), code:find_all('bar')}, 'value'):play(1)
```"#]
    fn replace(lua, object, args; ranges: "Kinematic.CodeRanges|nil", text: "string") -> "Kinematic.Animation" {
        let (interval, text): (Value, Value) = arguments(lua, args, 2)?;
        if interval.is_nil() {
            return Err(mlua::Error::runtime("Code edit range is missing."));
        }
        let Value::String(text) = text else {
            return Err(mlua::Error::runtime("Code text must be a string."));
        };
        prepare(
            lua,
            object,
            CodeAction::Replace(parse_ranges(lua, interval)?, text.to_str()?.to_owned()),
        )
    }

    #[doc = r#"Erases the text chosen by find, find_all, word, or lines. Accepts one interval or a list mixing their results. Changed tokens fade as whole words. Overlapping intervals or a missing single search result raise an error. Use remove to remove the scene object.

```lua
-- Removes the first line.
code:erase(code:lines(1)):play(1)
-- Removes every occurrence.
code:erase(code:find_all('debug')):play(1)
```"#]
    fn erase(lua, object, args; ranges: "Kinematic.CodeRanges|nil") -> "Kinematic.Animation" {
        let interval: Value = arguments(lua, args, 1)?;
        if interval.is_nil() {
            return Err(mlua::Error::runtime("Code edit range is missing."));
        }
        prepare(lua, object, CodeAction::Erase(parse_ranges(lua, interval)?))
    }

    #[doc = r#"Dims the text outside the chosen intervals, preserving syntax colors. Choose text with find, find_all, word, or lines, and optionally mix their results in a list. Nil or {} restores all text. Missing searches inside a list are ignored. Intervals keep their coordinates after edits.

```lua
code:select(code:find_all('value')):play(1)
-- Combines searches and a complete line.
code:select({code:find('1'), code:find_all('oi'), code:lines(1)}):play(1)
code:select(nil):play(1)
```"#]
    fn select(lua, object, args; ranges: "Kinematic.CodeRanges|nil") -> "Kinematic.Animation" {
        let count = args.len().min(1);
        let value: Value = arguments(lua, args, count)?;
        prepare(lua, object, CodeAction::Select(parse_ranges(lua, value)?))
    }

    #[doc = r#"Marks the chosen text with highlight_color. Choose text with find, find_all, word, or lines, and optionally mix their results in a list. Nil or {} removes the backgrounds. Missing searches inside a list are ignored. Included in exports. Intervals keep their coordinates after edits.

```lua
code:highlight(code:find_all('value')):play(1)
code:to {highlight_color = k.color('#FFCC0066')}:play(1)
code:highlight({}):play(1)
```"#]
    fn highlight(lua, object, args; ranges: "Kinematic.CodeRanges|nil") -> "Kinematic.Animation" {
        let count = args.len().min(1);
        let value: Value = arguments(lua, args, count)?;
        prepare(
            lua,
            object,
            CodeAction::Highlight(parse_ranges(lua, value)?),
        )
    }


    #[doc = r#"Finds the first literal occurrence in the code at the current scene time. Match_case and match_whole_word default to false. Whole words use letters, digits, and underscores; adjacent punctuation is allowed. Returns a CodeRange to pass to select, highlight, replace, or erase, or nil when absent. The result is a fixed range, not a live query.

```lua
local range = code:find('value')
if range then
    code:replace(range, 'answer'):play(1)
end
```"#]
    fn find(lua, object, args; text: "string", match_case: "boolean?", match_whole_word: "boolean?") -> "Kinematic.CodeRange?" {
        match matches(lua, object, args)?.first() {
            Some(range) => range_to_lua(lua, *range)?.into_lua(lua),
            None => Ok(Value::Nil),
        }
    }

    #[doc = r#"Finds all non-overlapping literal occurrences at the current scene time. Match_case and match_whole_word default to false. Whole words use letters, digits, and underscores; adjacent punctuation is allowed. Returns an empty list when absent. Returned coordinates do not follow later edits.

```lua
code:highlight(code:find_all('value')):play(1)
```"#]
    fn find_all(lua, object, args; text: "string", match_case: "boolean?", match_whole_word: "boolean?") -> "Kinematic.CodeRange[]" {
        ranges_to_lua(lua, &CodeRanges::new(matches(lua, object, args)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::{ProjectContext, objects::TextFade},
        scripting::{LuaScene, Sources},
    };
    use std::path::PathBuf;

    fn load(source: &str) -> Result<LuaScene> {
        LuaScene::load(
            ProjectContext::new(".").unwrap(),
            "code.lua",
            Sources::from([(PathBuf::from("code.lua"), source.to_owned())]),
            (1280, 720),
            60,
            (0, 0),
        )
    }

    #[test]
    fn searches_support_case_and_word_options_next_to_punctuation() {
        load(
            r#"
return function(s)
    local c = s:code {code='Foo(foo):foo foo_bar food FOO\nÉ(é)'}
    s:world_2d():add(c)
    assert(#c:find_all('foo') == 6)
    assert(#c:find_all('foo', true) == 4)
    assert(#c:find_all('foo', false, true) == 4)
    assert(#c:find_all('foo', true, true) == 2)
    assert(c:find('foo')[1][2] == 1)
    assert(c:find('foo', true)[1][2] == 5)
    assert(c:find('OO', false, true) == nil)
    assert(#c:find_all('(', false, true) == 2)
    assert(#c:find_all('foo', nil, true) == 4)
    local accents = c:find_all('é')
    assert(#accents == 2)
    assert(accents[1][1][1] == 2 and accents[1][1][2] == 1 and accents[1][2][2] == 2)
    assert(accents[2][1][2] == 3 and accents[2][2][2] == 4)
    assert(#c:find_all('é', true, true) == 1)
    c:replace(c:find_all('foo', false, true), 'x'):play(1)
    assert(c.code:get() == 'x(x):x foo_bar food x\nÉ(é)')
    local repeated = s:code {code='aaaa'}
    s:world_2d():add(repeated)
    assert(#repeated:find_all('aa') == 2)
end
"#,
        )
        .unwrap();
        let definitions = crate::scripting::definitions();
        for method in ["find", "find_all"] {
            assert!(definitions.contains(&format!(
                "function Code:{method}(text, match_case, match_whole_word) end"
            )));
        }
    }

    #[test]
    fn morph_group_is_optional_typed_and_preserved_when_seeking() {
        for (builder, world, property) in [
            ("text_2d", "world_2d", "text"),
            ("text_3d", "world_3d", "text"),
            ("latex_2d", "world_2d", "text"),
            ("latex_3d", "world_3d", "text"),
            ("code", "world_2d", "code"),
        ] {
            let scene = load(&format!(
                r#"
return function(s)
    local c = s:{builder} {{id='object', {property}='abc'}}
    s:{world}():add(c)
    assert(not pcall(function() c:morph('def', 1) end))
    assert(not pcall(function() c:morph('def', true, false) end))
    c:morph('def', true):play(1, k.ease.linear)
    c:morph('ghi'):play(1, k.ease.linear)
    c:morph('jkl', false):play(1, k.ease.linear)
end
"#
            ))
            .unwrap();
            let entity = scene.entity("object").unwrap();
            for (time, group) in [(0.5, true), (1.5, false), (2.5, false), (0.5, true)] {
                scene.update(time).unwrap();
                let world = scene.scene().world();
                let fade = world.get::<&TextFade>(entity).unwrap();
                assert!(fade.active && fade.morph, "{builder}");
                assert_eq!(fade.group, group, "{builder} at {time}");
            }
        }
    }

    #[test]
    fn edits_resolve_at_play_and_emphasis_survives_backward_seeks() {
        let scene = load(
            r#"
return function(s)
    local c = s:code {id='code', code='é🦀\nprint(1)'}
    local group = s:group_2d {}
    s:world_2d():add(group)
    assert(group:insert(c, 1) == nil)
    local edit = c:insert({1, 2}, 'x')
    assert(c.code:get() == 'é🦀\nprint(1)')
    edit:play(1, k.ease.linear)
    edit:play(1, k.ease.linear)
    assert(c.code:get() == 'éxx🦀\nprint(1)')
    local found = c:find('🦀')
    assert(found[1][1] == 1 and found[1][2] == 4 and found[2][2] == 5)
    assert(c:find('missing') == nil)
    assert(#c:find_all('missing') == 0)
    s:all(function()
        c:select(c:find_all('print')):play(1, k.ease.linear)
        c:highlight(c:find_all('print')):play(1, k.ease.linear)
    end)
    s:all(function()
        c:select(nil):play(1, k.ease.linear)
        c:highlight({}):play(1, k.ease.linear)
    end)
    c:replace(c:find('1'), '42'):play(1, k.ease.linear)
    c:erase({{1, 2}, {1, 4}}):play(1, k.ease.linear)
    c:fade('done'):play(1, k.ease.linear)
end
"#,
        )
        .unwrap();
        let entity = scene.entity("code").unwrap();
        for (time, expected, highlight, outside) in [
            (7.0, "done", 0.0, 1.0),
            (2.5, "éxx🦀\nprint(1)", 0.5, 0.5),
            (3.0, "éxx🦀\nprint(1)", 1.0, 0.0),
            (3.5, "éxx🦀\nprint(1)", 0.5, 0.5),
            (0.0, "é🦀\nprint(1)", 0.0, 1.0),
            (5.5, "éxx🦀\nprint(42)", 0.0, 1.0),
            (6.0, "é🦀\nprint(42)", 0.0, 1.0),
        ] {
            scene.update(time).unwrap();
            let world = scene.scene().world();
            let shape = world.get::<&CodeShape>(entity).unwrap();
            assert_eq!(shape.code, expected, "Time: {time}.");
            assert!((shape.highlights.coverage([2, 1], false) - highlight).abs() < 0.001);
            assert!((shape.selection.coverage([1, 1], true) - outside).abs() < 0.001);
            let fade = world.get::<&TextFade>(entity).unwrap();
            if time == 5.5 {
                assert!(fade.active && !fade.morph);
                assert!((fade.progress - 0.5).abs() < 0.001);
            }
        }
    }

    #[test]
    fn mixed_queries_line_helpers_and_line_edits_share_the_same_ranges() {
        let scene = load(
            r#"
return function(s)
    local code = s:code {id='code', code='local one = 1\noi oi\nend'}
    s:world_2d():add(code)
    local word = code:word(1, 3, 3)
    assert(word[1][2] == 3 and word[2][2] == 6)
    assert(code:word(1, 3)[2][2] == 14)
    assert(code:lines(2)[2][1] == 3 and code:lines(2)[2][2] == 1)
    assert(code:lines(1, 3)[2][1] == 3 and code:lines(1, 3)[2][2] == 4)
    code:select({code:find('missing'), code:find_all('oi'), code:lines(1)}):play(1)
    code:highlight({code:find('1'), {code:find_all('oi')}, {}}):play(1)
    code:select(nil):play(1)
    code:highlight({}):play(1)
    assert(#code.selection:get() == 0 and #code.highlights:get() == 0)
    code:replace({code:word(1, 3, 3), code:find_all('oi')}, 'x'):play(1)
    assert(code.code:get() == 'lox one = 1\nx x\nend')
    code:erase(code:find_all('x')):play(1)
    assert(code.code:get() == 'lo one = 1\n \nend')
    code:append('print(1)'):play(1)
    code:prepand('-- Start.'):play(1)
    assert(code.code:get() == '-- Start.\nlo one = 1\n \nend\nprint(1)')
    assert(not pcall(function() return code.clear_selection end))
    assert(not pcall(function() return code.clear_highlight end))
end
"#,
        )
        .unwrap();
        let entity = scene.entity("code").unwrap();
        scene.update(7.5).unwrap();
        assert!(
            scene
                .scene()
                .world()
                .get::<&crate::core::objects::CodeEditState>(entity)
                .unwrap()
                .active
        );
        scene.update(0.0).unwrap();
        assert_eq!(
            scene
                .scene()
                .world()
                .get::<&CodeShape>(entity)
                .unwrap()
                .code,
            "local one = 1\noi oi\nend"
        );
    }

    #[test]
    fn malformed_arguments_and_out_of_bounds_edits_fail_without_scheduling() {
        for operation in [
            "c:insert({0, 1}, 'x'):play(1)",
            "c:insert({1, 99}, 'x'):play(1)",
            "c:insert({1.5, 1}, 'x'):play(1)",
            "c:insert({1, 1, other=2}, 'x'):play(1)",
            "c:replace(nil, 'x'):play(1)",
            "c:replace({{1, 3}, {1, 1}}, 'x'):play(1)",
            "c:erase({{1, 1}, {3, 1}}):play(1)",
            "c:select({{{1, 1}, {1, 99}}}):play(1)",
            "c:highlight({{{1, 1}, {1, 99}}}):play(1)",
            "c:find('')",
            "c:find_all('')",
            "c:find('a', 1)",
            "c:find_all('a', false, 'true')",
            "c:find('a', false, false, true)",
            "c:highlight(1)",
            "c:morph(5):play(1)",
            "c:word(1, 0)",
            "c:word(1, 2, 99)",
            "c:lines(2, 1)",
            "c:lines(4)",
            "c:replace({{{1, 1}, {1, 3}}, {{1, 2}, {1, 4}}}, 'x'):play(1)",
            "local ranges={}; ranges[1]=ranges; c:select(ranges):play(1)",
        ] {
            let source = format!(
                "return function(s) local c=s:code{{code='abc'}}; s:world_2d():add(c); assert(not pcall(function() {operation} end)); assert(c.code:get()=='abc'); c:morph('ok'):play(1); assert(c.code:get()=='ok') end"
            );
            let scene = load(&source).unwrap_or_else(|error| panic!("{operation}: {error}"));
            assert!((scene.duration() - 1.0).abs() < 0.001, "{operation}");
        }
        assert!(load("return function(s) s:code {language='unknown'} end").is_err());
    }

    #[test]
    fn language_choices_are_available_in_lua_and_generated_documentation() {
        use crate::core::{TrackEnum, objects::CodeLanguage};
        let definitions = crate::scripting::definitions();
        let field = definitions
            .lines()
            .find(|line| line.starts_with("---@field language? "))
            .unwrap();
        let mut source = String::from("return function(s)\n");
        for language in CodeLanguage::VARIANTS {
            assert!(field.contains(&format!("'{language}'")), "{language}");
            source.push_str(&format!("local code=s:code{{language='{language}'}}; assert(code.language:get()=='{language}')\n"));
        }
        source.push_str("end");
        load(&source).unwrap();
        let scene = load(include_str!(
            "../../examples/lua/2d/scenes/code_languages.lua"
        ))
        .unwrap();
        for time in [1.5, 9.5, 18.5, 0.0, scene.duration()] {
            scene.update(time).unwrap();
        }
    }

    #[test]
    fn example_and_generated_function_documentation_include_working_code() {
        let scene = load(include_str!("../../examples/lua/2d/scenes/code.lua")).unwrap();
        for time in [0.0, 1.0, scene.duration(), 2.0] {
            scene.update(time).unwrap();
        }
        let definitions = crate::scripting::definitions();
        for method in METHODS {
            assert!(method.documentation.contains("```lua"), "{}", method.name);
            assert!(!method.documentation.contains("snippet"));
            for tween in method.documentation.split(":play(").skip(1) {
                assert!(tween.starts_with("1)"), "{}", method.name);
            }
            assert!(definitions.contains(method.documentation.lines().next().unwrap()));
        }
        assert!(definitions.contains("---@class Kinematic.CodeRange\n"));
        assert!(!definitions.contains("---@alias Kinematic.CodeRange {[1]:"));
    }
}
