---
layout: home
hero:
  name: Kinematic
  text: Animation with Lua
  tagline: >
    Build 2D and 3D scenes, shape their timing, and export videos with audio.
  actions:
    - theme: brand
      text: Create your first animation
      link: /guide/start
    - theme: alt
      text: Explore examples
      link: /examples/
features:
  - title: Learn the concepts
    details: Understand objects, hierarchy, and timing before combining features.
    link: /guide/objects
  - title: Browse the API
    details: Find signatures, properties, defaults, and limits generated from the editor's source.
    link: /api/
  - title: Explore working projects
    details: Run focused examples of built-in objects, components, and custom objects.
    link: /examples/
---

<script setup>
import { withBase } from 'vitepress'
</script>

## Make your first animation

Create a circle, give it a color, and schedule movement. Each step in
[first steps](guide/start.md) has its own result to compare with your scene.

<figure>
<video controls autoplay loop muted playsinline preload="metadata" :poster="withBase('/previews/guide-first-combined.png')" :src="withBase('/previews/guide-first-combined.mp4')" aria-label="A circle moves and changes color." style="width:100%"></video>
</figure>

## Choose a learning path

1. Learn [Lua essentials](guide/lua.md) and build your [first scene](guide/start.md).
2. Arrange [objects](guide/objects.md) using [coordinates](guide/coordinates.md), [layout](guide/layout.md), and [images](guide/images.md).
3. Control [animation](guide/animation.md), [easing](guide/easing.md), and [sequences](guide/sequences.md).
4. Add [3D](guide/graphics-3d.md), [canvas composition](guide/canvases.md), or [custom behavior](guide/custom-objects.md).
5. Review the result in the [editor](guide/editor.md) and [export a video](guide/export.md).
