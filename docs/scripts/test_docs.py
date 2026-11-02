import html
import importlib.util
import re
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('docs', Path(__file__).with_name('docs.py'))
docs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(docs)


class DocumentationTests(unittest.TestCase):
    def test_api_signatures_preserve_types_and_descriptions_in_compact_highlighted_entries(self):
        def visible(markup):
            return html.unescape(re.sub(r'<[^>]+>', '', markup))

        block = '''--- Independent canvas.
---@class Kinematic.Canvas2D: Kinematic.Object
---@field signal fun(self: Kinematic.Canvas2D, callback: fun(object: Kinematic.Canvas2D, frame: {time:number, dt:number})): Kinematic.Signal Runs after evaluation.
---@field filter? Kinematic.Property<'nearest' | 'linear'> Sampling mode. Default: "nearest".
---@field zoom number Magnification. Default: 1.5. Minimum: 0.001.
---@field position Kinematic.Vec2 Initial position. Default: [0, 1].'''
        rendered = '\n'.join(docs.render_api_block(
            block, type_names={'Object', 'Property', 'Signal', 'Vec2'}))
        plain = visible(rendered)
        self.assertIn('frame: {time: number, dt: number})): Signal', plain)
        self.assertIn('Runs after evaluation.', plain)
        self.assertIn('filter?: Property<\'nearest\' | \'linear\'> = "nearest"', plain)
        self.assertIn('zoom: number = 1.5', plain)
        self.assertIn('Magnification. Minimum: 0.001.', plain)
        self.assertIn('position: Vec2 = {0, 1}', plain)
        self.assertIn('Sampling mode.', plain)
        self.assertNotIn('Default:', plain)
        self.assertIn('href="Object.html"', rendered)
        self.assertIn('href="Signal.html"', rendered)
        self.assertIn('class="api-string"', rendered)
        self.assertIn('class="api-literal"', rendered)
        self.assertIn('{#property-filter}', rendered)
        self.assertNotIn('---@', rendered)
        self.assertNotIn('<dl', rendered)

        method = '''--- Schedules an animation.
---@generic T
---@param self Kinematic.Property<T>
---@param easing? "soft_light" | "linear" Interpolation curve.
---@return Kinematic.Animation Prepared animation.
function Property:play(easing) end'''
        rendered = '\n'.join(docs.render_api_block(method, type_names={'Animation'}))
        plain = visible(rendered)
        self.assertIn('Property:play(easing?: "soft_light" | "linear"): Animation', plain)
        self.assertIn('Interpolation curve.', plain)
        self.assertIn('Prepared animation.', plain)
        self.assertIn('{#property-play-easing}', rendered)
        self.assertIn('href="Animation.html"', rendered)
        self.assertIn('class="api-function"', rendered)
        self.assertIn('class="api-parameter"', rendered)
        self.assertNotIn('self', plain)
        self.assertNotIn('### Parameters', plain)
        self.assertNotIn('### Returns', plain)
        self.assertNotIn('function Property', plain)

    def test_documented_defaults_preserve_unstructured_prose_and_lua_values(self):
        self.assertEqual(docs.property_default('Visible. Default: false.'), ('false', 'Visible.'))
        self.assertEqual(docs.property_default('Empty. Default: null.'), ('nil', 'Empty.'))
        for description in ('Default: nearest.', 'Default: [] (1.00).', 'Uses the default font.'):
            self.assertEqual(docs.property_default(description), ('', description))

    def test_registered_functions_are_preserved_once_with_their_annotations(self):
        source = docs.run(docs.binary(), '--lua-definitions', capture_output=True, text=True).stdout
        pages = docs.api_pages(source)
        generated = '\n'.join('\n'.join(blocks) for blocks in pages.values())
        for line in source.splitlines():
            if line.startswith('function ') or line.startswith('---@field '):
                self.assertIn(line, generated)
            if line.startswith('function '):
                self.assertEqual(generated.count(line + '\n'), source.count(line + '\n'))
        self.assertTrue(all(any(block.strip() for block in blocks) for blocks in pages.values()))
        self.assertIn('---@field source Kinematic.AudioResource', '\n'.join(pages['AudioOptions']))
        self.assertIn('---@alias Kinematic.Easing "linear"|', '\n'.join(pages['Aliases']))
        self.assertNotIn('Easing', pages)
        self.assertIn('Default: 128. Minimum: 0.', generated)
        self.assertIn('function Text2D:morph(text, group)', '\n'.join(pages['Text2D']))
        self.assertNotIn('function Text2D:morph', '\n'.join(pages['Circle']))
        self.assertIn('function Property:to(value)', '\n'.join(pages['Property']))

    def test_missing_anchor_is_rejected_after_html_rendering(self):
        from tempfile import TemporaryDirectory
        from unittest.mock import patch
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root / '.vitepress/dist'
            output.mkdir(parents=True)
            page = output / 'index.html'
            page.write_text('<h1 id="revisao">Revisão</h1><a href="#revisão">Link</a>')
            with patch.object(docs, 'DOCS', root), patch.object(docs, 'projects', return_value=[]):
                with self.assertRaisesRegex(AssertionError, 'missing anchor'):
                    docs.links()
                page.write_text('<h1 id="revisao">Revisão</h1><a href="#revisao">Link</a>')
                docs.links()

    def test_release_overlay_changes_only_the_requested_guide(self):
        from tempfile import TemporaryDirectory
        from unittest.mock import patch
        import json
        import docs_release
        for editorial_tag, expected in [('', 'Tagged guide.'), ('v0.1.0', 'Corrected guide.')]:
            with self.subTest(editorial_tag=editorial_tag), TemporaryDirectory() as temporary:
                root = Path(temporary)
                for name, content in {
                    'docs/index.md': 'Corrected guide.',
                    'docs/editorial.json': json.dumps({editorial_tag: 'a' * 40} if editorial_tag else {}),
                    'docs/package.json': '{}', 'docs/package-lock.json': '{}',
                    'docs/scripts/docs.py': '', 'docs/scripts/test_docs.py': '', 'docs/scripts/docs_release.py': '',
                }.items():
                    path = root / name
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text(content)
                (root / 'docs/node_modules').mkdir()
                (root / 'target/runtime').mkdir(parents=True)
                def execute(*args, cwd=root, **kwargs):
                    result = ''
                    if args[:3] == ('gh', 'api', '--paginate'):
                        result = 'v0.1.0\n'
                    elif args[:2] == ('gh', 'api'):
                        result = json.dumps({'tag_name': 'v0.1.0'})
                    elif args[:2] == ('git', 'archive'):
                        import io, tarfile
                        buffer = io.BytesIO()
                        with tarfile.open(fileobj=buffer, mode='w') as archive:
                            archive.add(root / 'docs/index.md', arcname='docs/index.md')
                            archive.add(root / 'docs/editorial.json', arcname='docs/editorial.json')
                        result = buffer.getvalue()
                    elif args[:2] == ('git', 'show'):
                        result = '--lua-definitions'
                    elif args[:3] == ('git', 'worktree', 'add'):
                        checkout = Path(args[-2])
                        (checkout / 'docs').mkdir(parents=True)
                        (checkout / 'docs/index.md').write_text('Tagged guide.')
                        (checkout / 'src').mkdir()
                        (checkout / 'src/api.lua').write_text('Tagged runtime.')
                    elif args == ('npm', 'run', 'docs:build'):
                        output = cwd / '.vitepress/dist'
                        output.mkdir(parents=True, exist_ok=True)
                        (output / 'index.html').write_text((cwd / 'index.md').read_text())
                    return docs.subprocess.CompletedProcess(args, 0, result)
                with patch.object(docs_release, 'ROOT', root), patch.object(docs_release, 'run', execute), patch.dict(docs.os.environ, {'GH_REPO': 'owner/kinematic'}):
                    docs_release.build()
                self.assertEqual((root / 'docs/.cache/pages/stable/index.html').read_text(), expected)
                self.assertEqual((root / 'docs/.cache/pages/versions/v0.1.0/index.html').read_text(), expected)
                self.assertEqual((root / 'docs/.cache/releases/v0.1.0/src/api.lua').read_text(), 'Tagged runtime.')

    def test_version_entry_points_use_the_generated_theme_and_available_version(self):
        from tempfile import TemporaryDirectory
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            assets = root / 'dev/assets'
            assets.mkdir(parents=True)
            (assets / 'style.test.css').write_text('body { background: black; }')
            docs.run('python3', str(docs.ROOT / 'docs/scripts/docs_landing.py'), str(root))
            self.assertIn('url=./dev/', (root / 'index.html').read_text())
            self.assertIn('../dev/assets/style.test.css', (root / 'versions/index.html').read_text())
            (root / 'stable').mkdir()
            (root / 'stable/index.html').write_text('Stable version.')
            (root / 'versions/v0.1.0').mkdir()
            docs.run('python3', str(docs.ROOT / 'docs/scripts/docs_landing.py'), str(root))
            self.assertIn('url=./stable/', (root / 'index.html').read_text())
            self.assertIn('./v0.1.0/', (root / 'versions/index.html').read_text())
