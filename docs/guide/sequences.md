---
pageClass: guide-page
---

# Sequences, parallel work, and events

Scene code schedules work using a time cursor. A two-second `play` normally advances
that cursor by two seconds. Scopes decide how their children share it.

## Sequence and simultaneity

<GuideDemo name="sequence-all" caption="The white sequence takes two seconds. The blue movement takes one.">

<!-- demo: sequence-all -->
<<< ./projects/sequences/scenes/01-sequence-all.lua

</GuideDemo>

`chain` adds durations. `all` starts its children together and takes the longest child's
duration. Here `all` takes two seconds, followed by one second of stillness.

```text
Time:     0             1             2         3
White:    [ To center. ][ To right.  ][ Hold. ]
Blue:     [ To right.  ][       Hold.         ]
```

## Delay a sequence

`k.delay(seconds, schedule)` creates a sequential scope with an initial pause relative
to the current scheduling position. Seconds must be finite and nonnegative.

```lua
k.all(function()
    title.opacity:to(1):play(0.4)
    k.delay(0.2, function()
        subtitle.opacity:to(1):play(0.4)
        subtitle.position.y:by(-20):play(0.3)
    end)
end)
```

The delayed sequence starts at 0.2 seconds and finishes at 0.9 seconds, which is also
the duration of `all`. This is equivalent to a `chain` beginning with `k.wait(0.2)`.
The callback runs once during scene construction to schedule work at that offset.
Inside `parallel`, the delay and its children leave the parent duration unchanged.
Use [`:delay(seconds)`](animation.md#delay-an-animation) for a single animation.

## Work alongside the main sequence

`parallel` schedules work without advancing or extending its parent. Reserve time in the
parent or the scene can end before the parallel work is visible.

<GuideDemo name="parallel-loop" caption="A one-second bounce repeats while a separate two-second motion moves right.">

<!-- demo: parallel-loop -->
<<< ./projects/sequences/scenes/02-parallel-loop.lua

</GuideDemo>

The loop prepares one finite cycle and repeats it over the object's lifetime. The wait
makes the scene last two seconds. Try `wait(1)` to see both operations cut short.
Inside `parallel`, operations are sequential unless nested in `all`.

Do not create, attach, or remove objects or schedule audio inside a repeat cycle. Nested
loops and ambiguous concurrent writes are rejected. A Lua `while true` loop blocks scene
construction. It cannot animate a scene.

## Editable pauses

Use `k.wait(1)` for a fixed pause, or `k.event("Reading")` for a named pause you can
adjust in Timeline. An event gives the reader time to inspect a finished diagram without
changing its values. Resolve pending code edits before adjusting it. Event timing lives
in [scene metadata](projects.md#reloads-and-revisions), separately from Lua.

Scopes can nest: put a `chain` inside `all` when a sequence must run alongside another
animation. Sketch its time bars first to predict the final duration.
