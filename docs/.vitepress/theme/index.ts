import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import GuideDemo from './GuideDemo.vue'
import './style.css'

const BACKGROUND = [10, 10, 10] as const
const CONTRAST = 0.5

function applyTheme() {
  const root = document.documentElement
  const background = BACKGROUND.map((channel) => channel / 255)
  const contrast = Math.min(1, Math.max(0, CONTRAST))
  const linear = (channel: number) =>
    channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4
  const luminance =
    linear(background[0]) * 0.2126 +
    linear(background[1]) * 0.7152 +
    linear(background[2]) * 0.0722
  const foreground = luminance > 0.179 ? [0, 0, 0] : [1, 1, 1]
  const foregroundColor = `rgb(${foreground.map((channel) => channel * 255).join(' ')})`
  const mix = (amount: number) =>
    `rgb(${background.map((channel, index) => Math.round((channel + (foreground[index] - channel) * amount * contrast) * 255)).join(' ')})`

  root.style.setProperty('--kinematic-background', `rgb(${BACKGROUND.join(' ')})`)
  root.style.setProperty('--kinematic-foreground', foregroundColor)
  root.style.setProperty('--kinematic-surface', mix(0.08))
  root.style.setProperty('--kinematic-hover', mix(0.14))
  root.style.setProperty('--kinematic-active', mix(0.22))
  root.style.setProperty('--kinematic-border', mix(0.12))
  root.style.setProperty('--kinematic-disabled', mix(0.5))
}

if (typeof document !== 'undefined') applyTheme()

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('GuideDemo', GuideDemo)
  },
} satisfies Theme
