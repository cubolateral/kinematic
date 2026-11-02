"""Build published versions from tags with editorial corrections from this checkout."""
import json
import io
import tarfile
import os
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def run(*args, cwd=ROOT, **kwargs):
    return subprocess.run(args, cwd=cwd, check=True, **kwargs)


def build():
    repository = os.environ['GH_REPO']
    tags = run('gh', 'api', '--paginate', f'repos/{repository}/releases', '--jq',
               '.[] | select(.draft == false and .prerelease == false) | .tag_name',
               capture_output=True, text=True).stdout.splitlines()
    editorial = json.loads((ROOT / 'docs/editorial.json').read_text())
    latest = None
    if tags:
        latest = json.loads(run('gh', 'api', f'repos/{repository}/releases/latest',
                                capture_output=True, text=True).stdout)['tag_name']
    for tag in tags:
        if not re.fullmatch(r'v\d+\.\d+\.\d+(?:[-+][\w.-]+)?', tag):
            raise ValueError(f'Unsupported release tag: {tag}.')
        main = run('git', 'show', f'{tag}:src/main.rs', capture_output=True, text=True).stdout
        if '--lua-definitions' not in main:
            print(f'Skipping {tag}: predates generated Lua documentation.')
            continue
        checkout = ROOT / 'docs/.cache/releases' / tag
        run('git', 'worktree', 'add', '--detach', str(checkout), tag)
        # Persist editorial revisions separately from executable versions.
        if tag in editorial:
            revision = editorial[tag]
            if not isinstance(revision, str) or not re.fullmatch(r'[0-9a-f]{40}', revision):
                raise ValueError(f'Editorial revision for {tag} must be a full commit SHA.')
            archive = run('git', 'archive', revision, 'docs', capture_output=True).stdout
            overlay = ROOT / 'docs/.cache/editorial' / tag
            with tarfile.open(fileobj=io.BytesIO(archive)) as source:
                source.extractall(overlay, filter='data')
            shutil.rmtree(checkout / 'docs')
            shutil.copytree(overlay / 'docs', checkout / 'docs')
        for name in ('docs/package.json', 'docs/package-lock.json', 'docs/scripts/docs.py', 'docs/scripts/test_docs.py', 'docs/scripts/docs_release.py'):
            destination = checkout / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, destination)
        (checkout / 'docs/node_modules').symlink_to(ROOT / 'docs/node_modules', target_is_directory=True)
        (checkout / 'target').mkdir(exist_ok=True)
        environment = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / 'target'),
                           KINEMATIC_BIN=str(ROOT / 'target/debug/kinematic'),
                           DOCS_VERSION=tag, DOCS_REF=tag)
        run('pwsh', '-File', 'scripts/distribute.ps1', 'prepare', cwd=checkout, env=environment)
        run('cargo', 'build', '--locked', cwd=checkout, env=environment)
        for command in ('docs:test', 'docs:examples'):
            run('npm', 'run', command, cwd=checkout / 'docs', env=environment)
        run('xvfb-run', '-a', 'npm', 'run', 'docs:media', cwd=checkout / 'docs', env=environment)
        variants = [f'versions/{tag}'] + (['stable'] if tag == latest else [])
        for variant in variants:
            environment['DOCS_BASE'] = f'/{repository.split("/")[1]}/{variant}/'
            run('npm', 'run', 'docs:build', cwd=checkout / 'docs', env=environment)
            shutil.copytree(checkout / 'docs/.vitepress/dist', ROOT / 'docs/.cache/pages' / variant)


if __name__ == '__main__':
    build()
