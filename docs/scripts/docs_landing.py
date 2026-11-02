"""Write entry points after assembling development and stable sites."""
from pathlib import Path
import sys
import html

root = Path(sys.argv[1])
version = 'stable' if (root / 'stable/index.html').is_file() else 'dev'
stylesheet, = (root / 'dev/assets').glob('style.*.css')
style = stylesheet.relative_to(root).as_posix()
(root / 'index.html').write_text(f'''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Kinematic</title>
<link rel="stylesheet" href="./{style}">
<meta http-equiv="refresh" content="0;url=./{version}/"></head>
<body><a href="./{version}/">Kinematic documentation</a></body></html>
''')
(root / '.nojekyll').touch()

versions = root / 'versions'
versions.mkdir(exist_ok=True)
links = ''.join(f'<li><a href="./{html.escape(path.name)}/">{html.escape(path.name)}</a></li>'
                for path in sorted(versions.iterdir(), reverse=True) if path.is_dir())
(versions / 'index.html').write_text(f'''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Versions — Kinematic</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="../{style}"></head>
<body><main style="max-width:80ch;margin:auto;padding:2rem">
<h1>Documented versions</h1><ul>{links}</ul>
<a href="../dev/">Development</a></main></body></html>
''')
