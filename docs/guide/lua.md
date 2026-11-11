---
pageClass: guide-page
---

# Lua conventions

A scene file returns a function with no arguments. Kinematic calls it to construct
objects and schedule work. Lua does not own a frame loop. The scene-local `k` module
provides constructors, values, resources, and scheduling operations in every project module.

## Variables, tables, and loops

`local` declares a variable. `{}` constructs a table, used both for named options and
ordered lists. Array indices start at 1. A numeric `for` loop is useful for repetition:

<GuideDemo name="lua-row" caption="An ordered table becomes a row of red, green, and blue circles." still>

<!-- demo: lua-row -->
<<< ./projects/lua/scenes/01-lua-row.lua

</GuideDemo>

Indices 1, 2, and 3 become X positions -192, 0, and 192. Add a fourth color and predict
its position before running the scene. `ipairs` traverses the array in order.

## Functions and the dot and colon

`k.vec2(100, 0)` uses a dot to call a function. `world:add(ball)` uses a colon to pass
`world` as the first argument. It is equivalent to `world.add(world, ball)`.

```lua
local function move_right(object, distance)
    object:by {position = {x = distance}}:play(1)
end
```

Call `move_right(ball, 100)` after adding the ball. A helper can schedule work using
the same scene cursor. It does not need its own update loop.

## Values, handles, and channels

`k.vec2` and `k.color` create values. `ball.position` is a property handle, while
`ball.position:get()` returns its current value. A channel lets you edit one component:

<GuideDemo name="lua-channel" caption="Only X changes. The circle stays on the horizontal axis.">

<!-- focus: lua-channel -->
```lua
ball.position.x:to(256):play(2, "linear")
```

<template #source>

<!-- demo: lua-channel -->
<<< ./projects/lua/scenes/02-lua-channel.lua

</template>

</GuideDemo>

A vector has `x/y/z` channels, a color has `r/g/b/a`, and a quad has `a/b/c/d`.
Try animating `position.y` instead. Tables such as `{x = 256}` can also specify partial
channels in object animations.

## Modules and shared palettes

Put shared values in `modules/palette.lua`:

```lua
return {
    accent = k.color("#008CFF"),
    success = k.color("#40FF80"),
}
```

Then `local palette = require("modules.palette")` makes `palette.accent` available to
all scenes that import it. Use it in `fill = palette.accent` in the row above to give
all circles one shared color. Module paths and asset paths are relative to the
[project root](projects.md#assets-paths-and-modules).

## Enum strings

Enums use exact `snake_case` strings, for example `direction = "horizontal_reverse"`,
`camera_mode = "orthogonal"`, and `play(1, "cubic_out")`. Capitalization and alternative
spellings are rejected. Find accepted values in the [Lua API](../api/index.md).
