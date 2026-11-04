<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { withBase } from 'vitepress'

defineProps<{ name: string; caption: string; still?: boolean }>()
const video = ref<HTMLVideoElement>()
let preference: MediaQueryList | undefined

function updatePlayback() {
  if (!video.value) return
  if (preference?.matches) video.value.pause()
  else void video.value.play().catch(() => {})
}

onMounted(() => {
  preference = window.matchMedia('(prefers-reduced-motion: reduce)')
  preference.addEventListener('change', updatePlayback)
  updatePlayback()
})

onBeforeUnmount(() => preference?.removeEventListener('change', updatePlayback))
</script>

<template>
  <figure class="guide-demo">
    <img v-if="still" :src="withBase(`/previews/guide-${name}.png`)"
      :alt="caption" width="960" height="540" loading="lazy" />
    <video v-else ref="video" controls loop muted playsinline preload="metadata"
      :poster="withBase(`/previews/guide-${name}.png`)"
      :src="withBase(`/previews/guide-${name}.mp4`)"
      :aria-label="caption" width="960" height="540" />
    <div class="guide-demo-code"><slot /></div>
    <details v-if="$slots.source" class="guide-demo-source">
      <summary>Complete scene</summary>
      <slot name="source" />
    </details>
    <figcaption>{{ caption }}</figcaption>
  </figure>
</template>

<style scoped>
.guide-demo {
  margin: 28px 0 36px;
}

.guide-demo > :is(video, img) {
  display: block;
  width: 100%;
  height: auto;
  aspect-ratio: 16 / 9;
  object-fit: contain;
  border-radius: var(--kinematic-radius) var(--kinematic-radius) 0 0;
  background: #000;
}

.guide-demo :deep(div[class*='language-']) {
  margin: 0;
  border-radius: 0 !important;
  border: 0;
}

.guide-demo-code,
.guide-demo-source {
  border: 1px solid var(--vp-c-border);
  border-top: 0;
}

.guide-demo-source summary {
  margin: 0;
  padding: 10px 20px;
  font-size: 13px;
  cursor: pointer;
}

.guide-demo-source summary:hover {
  background: var(--vp-c-bg-soft);
}

.guide-demo figcaption {
  margin-top: 14px;
  color: var(--vp-c-text-2);
  font-size: 13px;
  line-height: 1.7;
}
</style>
