import { defineConfig } from 'vitepress'
import { readdirSync } from 'node:fs'

const repository = 'https://github.com/cubolateral/kinematic'
const version = process.env.DOCS_VERSION || 'Development'
const groups = [
  ['Get started', [['First steps', 'start'], ['Editor', 'editor'], ['Lua conventions', 'lua'], ['Projects and scenes', 'projects']]],
  ['Animate and compose', [['Animation', 'animation'], ['Easing', 'easing'], ['Sequences and events', 'sequences'], ['Objects and hierarchy', 'objects'], ['Coordinates and anchors', 'coordinates'], ['Groups and layout', 'layout']]],
  ['Scene content', [['2D graphics', 'graphics-2d'], ['Text', 'text'], ['Images', 'images'], ['LaTeX', 'latex'], ['Code', 'code'], ['Audio', 'audio'], ['Export and CLI', 'export']]],
  ['Go further', [['Signals', 'signals'], ['Random', 'random'], ['3D graphics', 'graphics-3d'], ['Cameras', 'cameras'], ['Canvases and projections', 'canvases'], ['Custom objects', 'custom-objects'], ['Procedural drawing', 'drawing'], ['Simulation', 'simulation'], ['Shaders', 'shaders']]],
  ['Reference and help', [['Troubleshooting', 'troubleshooting'], ['Contributing', 'contributing'], ['Versions and migrations', 'versions']]],
] as const
export default defineConfig({
  appearance: 'force-dark',
  lang: 'en-US', title: 'Kinematic', description: 'Animation with Lua, from your first project to the API reference.',
  base: process.env.DOCS_BASE || '/kinematic/',
  head: [['link', { rel: 'icon', type: 'image/svg+xml', href: `${process.env.DOCS_BASE || '/kinematic/'}logo.svg` }]],
  srcExclude: ['internal/**', 'README.md', 'guide/projects/**', '.cache/**', 'node_modules/**'],
  markdown: {
    languages: ['lua', 'rust', 'glsl'], languageAlias: { ron: 'rust' },
    config(md) {
      md.renderer.rules.code_inline = (tokens, index) => `<code v-pre>${md.utils.escapeHtml(tokens[index].content)}</code>`
    },
  },
  themeConfig: {
    logo: { src: '/logo.svg', alt: 'Kinematic' },
    siteTitle: false,
    search: {
      provider: 'local',
      options: { translations: {
        button: { buttonText: 'Search', buttonAriaLabel: 'Search documentation' },
        modal: {
          displayDetails: 'Show details', resetButtonTitle: 'Clear search',
          backButtonTitle: 'Close search', noResultsText: 'No results for',
          footer: { selectText: 'Select', navigateText: 'Navigate', closeText: 'Close' },
        },
      } },
    },
    sidebarMenuLabel: 'Menu', returnToTopLabel: 'Back to top',
    nav: [
      { text: 'Guide', link: '/guide/start' }, { text: 'Lua API', link: '/api/' },
      { text: 'Examples', link: '/examples/' },
      { text: version, items: [
        ...(process.env.DOCS_STABLE_AVAILABLE === 'true' || version !== 'Development'
          ? [{ text: 'Stable', link: 'https://cubolateral.github.io/kinematic/stable/' }] : []),
        { text: 'Development', link: 'https://cubolateral.github.io/kinematic/dev/' },
        { text: 'Releases', link: 'https://cubolateral.github.io/kinematic/versions/' },
      ] },
    ],
    socialLinks: [{ icon: 'github', link: repository }],
    sidebar: [
      ...groups.map(([text, pages]) => ({ text, items: pages.map(([title, slug]) => ({ text: title, link: `/guide/${slug}` })) })),
      { text: 'Examples', link: '/examples/' },
      { text: 'Lua API reference', collapsed: true, items: readdirSync(new URL('../api/', import.meta.url)).filter(name => name.endsWith('.md')).sort().map(name => ({ text: name.slice(0, -3), link: `/api/${name.slice(0, -3)}` })) },
    ],
    outline: { level: [2, 3], label: 'On this page' },
    docFooter: { prev: 'Previous', next: 'Next' },
    editLink: { pattern: `${repository}/edit/${process.env.DOCS_REF || 'main'}/docs/:path`, text: 'Edit this page on GitHub' },
    footer: { message: `Documentation · ${version} · MIT license` },
  },
})
