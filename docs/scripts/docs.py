"""Build documentation from runtime LuaLS metadata and runnable example projects."""
import argparse
import html
import json
import math
import struct
from html.parser import HTMLParser
import os
from pathlib import Path
import re
import shutil
import subprocess
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / 'docs'
EXAMPLES = ROOT / 'examples/lua'


def binary():
    return os.environ.get('KINEMATIC_BIN', str(ROOT / 'target/debug/kinematic'))


def run(*args, **kwargs):
    return subprocess.run(args, check=True, cwd=ROOT, **kwargs)


def api_pages(source):
    """Keep annotations verbatim: nested Lua types are not a second type grammar."""
    pages, variables, pending = {}, {}, []
    current = None
    for line in source.splitlines():
        if line.startswith('---'):
            pending.append(line)
            match = re.match(r'---@class Kinematic\.([\w]+)(.*)', line)
            if match:
                current = match[1]
                pages.setdefault(current, [])
            continue
        if not line.strip():
            if current and pending:
                pages[current].append('\n'.join(pending))
                pending, current = [], None
            continue
        local = re.match(r'local (\w+) =', line)
        function = re.match(r'function ([\w]+)([:.])(\w+)\((.*?)\) end', line)
        if local:
            if current:
                variables[local[1]] = current
                pages[current].append('\n'.join(pending))
            pending = []
            current = None
        elif function:
            owner, separator, name, arguments = function.groups()
            # Some resource-only classes have no local declaration.
            if current:
                class_lines = [p for p in pending if p.startswith('---@class') or p.startswith('---@field')]
                pages[current].append('\n'.join(class_lines))
                pending = [p for p in pending if p not in class_lines]
                current = None
            owner = variables.get(owner, owner)
            pages.setdefault(owner, []).append('\n'.join(pending + [line]))
            pending = []
        else:
            if current and pending:
                pages[current].append('\n'.join(pending))
            pending = []
            current = None
    if current and pending:
        pages[current].append('\n'.join(pending))
    aliases = [line for line in source.splitlines() if line.startswith('---@alias ')]
    if aliases:
        pages['Aliases'] = ['\n'.join(aliases)]
    return pages


def annotation_type(value):
    """Separate a LuaLS type from prose without splitting nested or quoted types."""
    depth, quote, escaped = 0, None, False
    for index, character in enumerate(value):
        if quote:
            if escaped:
                escaped = False
            elif character == '\\':
                escaped = True
            elif character == quote:
                quote = None
        elif character in {'"', "'"}:
            quote = character
        elif character in '(<[{':
            depth += 1
        elif character in ')>]}':
            depth -= 1
        elif character.isspace() and depth == 0:
            before, after = value[:index].rstrip(), value[index:].lstrip()
            if before and after and before[-1] not in '|,:' and after[0] not in '|,:':
                return before, after
    return value.strip(), ''


def api_token(value, role):
    return f'<span class="api-{role}">{html.escape(value)}</span>'


def api_tokens(value, type_names=()):
    """Highlight the roles in a LuaLS signature without interpreting its types."""
    parts = re.split(r'''("(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')''', value)
    for index in range(0, len(parts), 2):
        part = parts[index].replace('Kinematic.', '')
        parts[index] = re.sub(r'\s*([,:|])\s*',
                             lambda match: ' | ' if match[1] == '|' else match[1] + ' ', part)
    formatted = ''.join(parts).strip()
    pattern = r'''("(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')|([A-Za-z_]\w*)|(\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)|(\s+)|(.)'''
    tokens = []
    for token in re.finditer(pattern, formatted):
        string, name, number, space, punctuation = token.groups()
        if string:
            tokens.append(api_token(string, 'string'))
        elif number or name in {'true', 'false', 'nil'}:
            tokens.append(api_token(token[0], 'literal'))
        elif name:
            if name == 'fun':
                tokens.append(api_token(name, 'keyword'))
            elif formatted[token.end():].lstrip().startswith((':', '?:')):
                tokens.append(api_token(name, 'parameter'))
            elif name in type_names:
                tokens.append(f'<a class="api-type-name" href="{name}.html">{name}</a>')
            else:
                tokens.append(api_token(name, 'type-name'))
        elif space:
            tokens.append(space)
        else:
            tokens.append(api_token(punctuation, 'punctuation'))
    return ''.join(tokens)


def api_type(value, type_names=()):
    return f'<code class="api-signature">{api_tokens(value, type_names)}</code>'


def property_default(description):
    """Move explicit JSON defaults from runtime metadata into the signature."""
    match = re.search(r'\bDefault: ', description)
    if not match:
        return '', description
    try:
        value, end = json.JSONDecoder().raw_decode(description[match.end():])
    except ValueError:
        return '', description
    end += match.end()
    if description[end:] and not description[end:].startswith('.'):
        return '', description

    def lua_literal(value):
        if value is None:
            return 'nil'
        if isinstance(value, float) and value.is_integer():
            return str(int(value))
        if isinstance(value, list):
            return '{' + ', '.join(lua_literal(item) for item in value) + '}'
        if isinstance(value, dict):
            return '{' + ', '.join(f'[{lua_literal(key)}] = {lua_literal(item)}'
                                   for key, item in value.items()) + '}'
        return json.dumps(value, ensure_ascii=False)

    remaining = ' '.join(part.strip() for part in
                         (description[:match.start()], description[end + 1:]) if part.strip())
    return lua_literal(value), remaining


def api_members(title, members, type_names=()):
    text = [title, '', '<div class="api-members">', '']
    for name, value in members:
        kind, description = annotation_type(value)
        default, description = property_default(description)
        member = api_token(name.rstrip('?'), 'name')
        if name.endswith('?'):
            member += api_token('?', 'punctuation')
        member += api_token(': ', 'punctuation') + api_tokens(kind, type_names)
        if default:
            member += api_token(' = ', 'punctuation') + api_tokens(default)
        anchor = re.sub(r'[^a-z0-9_-]+', '-', name.lower()).strip('-')
        text.extend([
            '<section class="api-member">', '',
            f'### <code class="api-signature">{member}</code> {{#property-{anchor}}}', '',
        ])
        if description:
            text.extend(['<div class="api-description">', '', description, '', '</div>', ''])
        text.extend(['</section>', ''])
    text.extend(['</div>', ''])
    return text


def api_method(signature, parameters, returns, type_names):
    call, arguments = signature.split('(', 1)
    arguments = arguments[:-1].strip()
    separator = ':' if ':' in call else '.'
    owner, _, method = call.rpartition(separator)
    result = api_tokens(owner, type_names) + api_token(separator, 'punctuation')
    result += api_token(method, 'function') + api_token('(', 'punctuation')
    declarations = {name.rstrip('?'): (name, annotation_type(value)[0])
                    for name, value in parameters}
    values = []
    for argument in arguments.split(',') if arguments else []:
        name = argument.strip()
        declared, kind = declarations.get(name, (name, ''))
        value = api_token(name, 'parameter')
        if declared.endswith('?'):
            value += api_token('?', 'punctuation')
        if kind:
            value += api_token(': ', 'punctuation') + api_tokens(kind, type_names)
        values.append(value)
    result += api_token(', ', 'punctuation').join(values) + api_token(')', 'punctuation')
    if returns:
        result += api_token(': ', 'punctuation')
        result += api_token(', ', 'punctuation').join(
            api_tokens(annotation_type(value)[0], type_names) for value in returns)
    return f'<code class="api-signature">{result}</code>'


def render_api_block(block, include_aliases=False, type_names=()):
    text = []
    signature = re.search(r'^function (.+?) end$', block, re.MULTILINE)
    prose = '\n'.join(line[4:] for line in block.splitlines() if line.startswith('--- '))
    annotations = re.findall(r'^---@([\w]+)\s+(.+)$', block, re.MULTILINE)
    if signature:
        parameters = [value.split(None, 1) for tag, value in annotations if tag == 'param']
        if ':' in signature[1].split('(', 1)[0]:
            parameters = [member for member in parameters if member[0] != 'self']
        returns = [value for tag, value in annotations if tag == 'return']
        heading = api_method(signature[1], parameters, returns, type_names)
        anchor = re.sub(r'[^a-z0-9_-]+', '-', signature[1].lower()).strip('-')
        text.extend(['<section class="api-method">', '', f'## {heading} {{#{anchor}}}', '',
                     '<div class="api-description">', ''])
        if prose:
            text.extend([prose, ''])
        details = []
        for name, value in parameters:
            _, description = annotation_type(value)
            if description:
                details.append(f'- {api_token(name.rstrip("?"), "parameter")}: {description}')
        for value in returns:
            _, description = annotation_type(value)
            if description:
                details.append(f'- **Returns:** {description}')
        if details:
            text.extend(['<div class="api-arguments">', '', *details, '', '</div>', ''])
    elif prose:
        text.extend([prose, ''])

    for tag, value in annotations:
        if tag == 'class':
            declaration, separator, parent = value.partition(':')
            if separator:
                text.extend([f'**Inherits:** {api_type(parent.strip(), type_names)}', ''])
            if '<' in declaration:
                text.extend([f'**Type:** {api_type(declaration, type_names)}', ''])
        elif tag == 'generic':
            text.extend([f'**Type parameter:** {api_type(value, type_names)}', ''])

    if not signature:
        for tag, title in [('field', '## Properties'), ('alias', '## Types')]:
            members = [value.split(None, 1) for kind, value in annotations if kind == tag]
            if members and (tag != 'alias' or include_aliases):
                text.extend(api_members(title, members, type_names))

    for tag, title in [('overload', 'Overloads'), ('operator', 'Operators')]:
        values = [value for kind, value in annotations if kind == tag]
        if values:
            text.extend([f'**{title}**', ''])
            for value in values:
                kind, description = annotation_type(value)
                text.extend([api_type(kind, type_names), ''])
                if description:
                    text.extend([description, ''])
    if signature:
        text.extend(['</div>', '', '</section>', ''])
    return text


def update():
    run('pwsh', '-File', 'scripts/distribute.ps1', 'prepare')
    run('cargo', 'build', '--locked')
    examples()
    generate()
    media()


def generate():
    source = run(binary(), '--lua-definitions', capture_output=True, text=True).stdout
    source = re.sub(r'`(-?\d+)\.0`', r'`\1`', source)
    destination = DOCS / 'api'
    destination.mkdir(exist_ok=True)
    for old in destination.glob('*.md'):
        old.unlink()
    pages = api_pages(source)
    index = ['---', 'editLink: false', '---', '', '# Lua API reference', '', 'Browse the types, properties, and methods available in Lua scenes.', '',
             'Read [objects and composition](../guide/objects.md), [animation](../guide/animation.md), and the [examples](../examples/index.md). Signatures use LuaLS type notation: `name?` marks an optional value, `|` separates alternatives, and `T[]` denotes a list. The `:` operator supplies `self`.', '',
             'Methods are registered for specific types. Requirements for `Object` operations are described in each entry. Attachment requires a container, camera operations require a 3D canvas, and simulation requires a Lua object with state. Use each operation only with compatible types.', '']
    for name, blocks in sorted(pages.items()):
        index.append(f'- [{name}]({name}.md).')
        text = ['---', 'editLink: false', 'outline: 2', '---', '', f'# {name}', '', '::: v-pre', '', '<div class="api-reference">', '', '[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)', '',
                'Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/' + os.environ.get('DOCS_REF', 'main') + '/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/' + os.environ.get('DOCS_REF', 'main') + '/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/' + os.environ.get('DOCS_REF', 'main') + '/src/core).', '']
        for block in blocks:
            if not block:
                continue
            text.extend(render_api_block(block, include_aliases=name == 'Aliases', type_names=pages))
        text.extend(['</div>', '', ':::', ''])
        (destination / f'{name}.md').write_text('\n'.join(text))
    (destination / 'index.md').write_text('\n'.join(index) + '\n')
    (DOCS / 'public').mkdir(exist_ok=True)
    logo = (ROOT / 'assets/brand/logo.svg').read_text().replace('#000000', '#ffffff')
    (DOCS / 'public/logo.svg').write_text(logo)
    downloads = DOCS / 'public/downloads'
    if downloads.exists():
        shutil.rmtree(downloads)
    (DOCS / 'public/kinematic.lua').write_text(source)
    fonts = DOCS / 'public/fonts'
    fonts.mkdir(exist_ok=True)
    for filename in ('JetBrainsMonoNerdFont-Regular.ttf', 'JetBrainsMonoNerdFont-Bold.ttf'):
        shutil.copyfile(ROOT / 'assets/fonts' / filename, fonts / filename)
    licenses = DOCS / 'public/licenses'
    licenses.mkdir(exist_ok=True)
    for filename in ('OFL-JetBrainsMono.txt', 'NerdFonts-LICENSE.md'):
        shutil.copyfile(ROOT / 'assets/fonts' / filename, licenses / filename)
    for page in DOCS.rglob('*.md'):
        if any(part in {'api', 'internal', '.vitepress', '.cache', 'node_modules'} for part in page.parts):
            continue
        for include in re.findall(r'^<<< ([^\n{]+)', page.read_text(), re.MULTILINE):
            assert (page.parent / include.strip()).is_file(), f'{page}: missing snippet {include}'
    print(f'Generated {len(pages)} Lua API pages.')


def projects():
    return sorted(p.parent.parent for p in EXAMPLES.glob('*/kinematic/project.ron'))


def guide_projects():
    """Find the editable, versioned project for each guide topic."""
    return sorted(p.parent.parent for p in (DOCS / 'guide/projects').glob('*/kinematic/project.ron'))


def guide_demos():
    """Pair each page preview with its scene without modifying project sources."""
    demos = []
    names = set()
    for page in sorted((DOCS / 'guide').glob('*.md')):
        content = re.sub(r'^```[^\n]*\n.*?^```[ \t]*$', '', page.read_text(),
                         flags=re.MULTILINE | re.DOTALL)
        for name, attributes, body in re.findall(
                r'<GuideDemo name="([a-z0-9-]+)"([^>]*)>(.*?)</GuideDemo>',
                content, re.DOTALL):
            assert name not in names, f'Duplicate guide demo: {name}.'
            names.add(name)
            sources = re.findall(r'<!-- demo: ([a-z0-9-]+) -->\s*<<< ([^\n]+)', body)
            assert len(sources) == 1 and sources[0][0] == name, \
                f'{page}: preview and source must share the name {name}.'
            project = DOCS / 'guide/projects' / page.stem
            scene = (page.parent / sources[0][1].strip()).resolve()
            assert scene.is_relative_to((project / 'scenes').resolve()) and scene.is_file(), \
                f'{page}: missing scene in {project}: {sources[0][1]}.'
            manifest = (project / 'kinematic/project.ron').read_text()
            scene_list = re.search(r'scenes:\s*\[(.*?)\]', manifest, re.DOTALL)
            assert scene_list and json.dumps(scene.relative_to(project.resolve()).as_posix()) in scene_list[1], \
                f'{page}: scene is not listed in the project manifest: {scene.name}.'
            demos.append((f'guide-{name}', project, scene, bool(re.search(r'\bstill\b', attributes))))
    return demos


def examples(guide_only=False):
    guide_demos()
    for project in ([] if guide_only else projects()) + guide_projects():
        run(binary(), '--check', str(project))


def media_sources(guide_only=False):
    return ([(project.name, project, None, False) for project in projects()] if not guide_only else []) + guide_demos()


def media(guide_only=False):
    destination = DOCS / 'public/previews'
    destination.mkdir(parents=True, exist_ok=True)
    sources = media_sources(guide_only)
    names = {name for name, _, _, _ in sources}
    for stale in destination.iterdir():
        if guide_only and not stale.stem.startswith('guide-'):
            continue
        if stale.suffix in {'.png', '.mp4'} and stale.stem not in names:
            stale.unlink()
    for name, project, scene, still in sources:
        # Render one scene from a temporary copy without changing the source project.
        temporary = DOCS / '.cache/media' / name
        if temporary.exists():
            shutil.rmtree(temporary)
        shutil.copytree(project, temporary, ignore=shutil.ignore_patterns('output', 'cache', 'overrides.ron'))
        if scene is not None:
            manifest = temporary / 'kinematic/project.ron'
            relative = scene.relative_to(project.resolve()).as_posix()
            manifest.write_text(re.sub(r'scenes:\s*\[.*?\]',
                                       lambda _: f'scenes: [{json.dumps(relative)}]',
                                       manifest.read_text(), count=1, flags=re.DOTALL))
        run(binary(), '--render', str(temporary))
        video, = (temporary / 'output').glob('*.mp4')
        if still:
            (destination / f'{name}.mp4').unlink(missing_ok=True)
        else:
            shutil.copyfile(video, destination / f'{name}.mp4')
        poster = destination / f'{name}.png'
        poster.unlink(missing_ok=True)
        timestamp = '0' if still else '1.0' if name.startswith('guide-') else '1.5'
        run('ffmpeg', '-v', 'error', '-y', '-ss', timestamp,
            '-i', str(video), '-frames:v', '1', str(poster))
        assert poster.is_file(), f'{name}: no frame at the poster timestamp.'
    verify_media(guide_only)


def verify_media(guide_only=False):
    for name, project, _, still in media_sources(guide_only):
        video = DOCS / f'public/previews/{name}.{"png" if still else "mp4"}'
        info = json.loads(run('ffprobe', '-v', 'error', '-show_streams', '-of', 'json',
                              str(video), capture_output=True, text=True).stdout)
        stream, = [s for s in info['streams'] if s['codec_type'] == 'video']
        manifest = (project / 'kinematic/project.ron').read_text()
        width, height = re.search(r'resolution:\s*\((\d+),\s*(\d+)\)', manifest).groups()
        fps = re.search(r'fps:\s*(\d+)', manifest)[1]
        assert (stream['width'], stream['height']) == (int(width), int(height)), project
        if not still:
            assert stream['r_frame_rate'] == f'{fps}/1', project
            assert int(stream['nb_frames']) > 0, project
        # Decode every frame and sample, including frames after the poster timestamp.
        run('ffmpeg', '-v', 'error', '-xerror', '-i', str(video), '-f', 'null', '-')
    if guide_only:
        print('Decoded guide images and videos. Checked dimensions and video FPS.')
        return
    video = DOCS / 'public/previews/audio.mp4'
    samples = run('ffmpeg', '-v', 'error', '-i', str(video), '-f', 'f32le', '-ac', '2',
                  '-ar', '48000', 'pipe:1', capture_output=True).stdout
    frames = list(struct.iter_unpack('<ff', samples))
    def rms(start, end):
        window = frames[int(start * 48000):int(end * 48000)]
        assert window, 'Missing audio interval.'
        return [math.sqrt(sum(frame[channel] ** 2 for frame in window) / len(window))
                for channel in (0, 1)]
    assert max(rms(0, 0.35)) < 0.0001, 'Sound starts before the first impact.'
    left, right = rms(0.53, 0.7)
    assert left > 0.01 and left > right * 1.5, 'First impact must pan left.'
    left, right = rms(1.53, 1.7)
    assert left > 0.01 and abs(left - right) < 0.005, 'Second impact must be centered.'
    left, right = rms(2.53, 2.7)
    assert right > 0.01 and right > left * 1.5, 'Third impact must pan right.'
    assert min(rms(3.03, 3.2)) > 0.005, 'The WAV occurrence is silent.'
    print('Decoded all videos. Checked dimensions, FPS, and synchronized stereo occurrences.')


class Links(HTMLParser):
    def __init__(self):
        super().__init__()
        self.targets, self.ids = [], set()

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if 'id' in attrs:
            self.ids.add(attrs['id'])
        for attr in ('href', 'src', 'poster'):
            if attr in attrs:
                self.targets.append(attrs[attr])


def links():
    root = DOCS / '.vitepress/dist'
    base = os.environ.get('DOCS_BASE', '/kinematic/')
    parsed = {}
    for path in root.rglob('*.html'):
        parser = Links()
        parser.feed(path.read_text())
        parsed[path] = parser
    assert parsed, 'No built HTML pages.'
    for page, parser in parsed.items():
        for target in parser.targets:
            url = urlsplit(html.unescape(target))
            if url.scheme or url.netloc:
                continue
            name = unquote(url.path)
            if name.startswith('/'):
                assert name.startswith(base), f'{page}: outside base: {target}'
                resolved = root / name[len(base):]
            else:
                resolved = page.parent / name if name else page
            if resolved.is_dir():
                resolved /= 'index.html'
            elif not resolved.exists() and not resolved.suffix:
                resolved = resolved.with_suffix('.html')
            resolved = resolved.resolve()
            assert resolved.is_relative_to(root.resolve()), f'{page}: escaping link: {target}'
            assert resolved.is_file(), f'{page}: missing {target}'
            if url.fragment and resolved in parsed:
                assert unquote(url.fragment) in parsed[resolved].ids, f'{page}: missing anchor {target}'
    print(f'Checked links, anchors, and assets in {len(parsed)} pages.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['update', 'generate', 'examples', 'media', 'verify_media', 'links'])
    parser.add_argument('--guide-only', action='store_true', help='Validate or render only guide demos.')
    args = parser.parse_args()
    if args.guide_only and args.command not in {'examples', 'media', 'verify_media'}:
        parser.error('--guide-only requires examples, media, or verify_media.')
    if args.command in {'examples', 'media', 'verify_media'}:
        globals()[args.command](args.guide_only)
    else:
        globals()[args.command]()
