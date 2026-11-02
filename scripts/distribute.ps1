param(
    [ValidateSet('prepare', 'dist')][string]$Task = 'dist',
    [string]$Tag = ''
)
$ErrorActionPreference = 'Stop'

Set-StrictMode -Version Latest

Set-Location (Split-Path $PSScriptRoot)

function Run([string]$Program, [string[]]$Arguments) {
    & $Program @Arguments

    if ($LASTEXITCODE -ne 0) {
        throw "$Program failed with exit code $LASTEXITCODE."
    }
}

if ([Environment]::Is64BitProcess -eq $false) {
    throw 'An x64 host is required.'
}

$platform = if ($env:OS -eq 'Windows_NT') {
    'windows'
} elseif ((uname -s) -eq 'Linux') {
    'linux'
} else {
    throw 'Only Windows and Linux are supported.'
}

$target = if ($platform -eq 'windows') { 'x86_64-pc-windows-msvc' } else { 'x86_64-unknown-linux-gnu' }
$suffix = if ($platform -eq 'windows') { '.exe' } else { '' }
$curl = (Get-Command curl -CommandType Application | Select-Object -First 1).Source
$version = [regex]::Match((Get-Content Cargo.toml -Raw), '(?m)^version = "([0-9]+\.[0-9]+\.[0-9]+)"').Groups[1].Value

if ($Tag -and ($Tag -cne "v$version")) {
    throw 'Tag must match the workspace version in Cargo.toml.'
}

$spec = (Get-Content scripts/dependencies.json -Raw | ConvertFrom-Json).$platform
$work = Join-Path $PWD "target/runtime/$platform"
$runtime = Join-Path $work 'runtime'
$bundle = Join-Path $work 'runtime.tar.gz'
$fingerprint = [Text.Encoding]::UTF8.GetBytes($platform + (Get-FileHash scripts/dependencies.json).Hash + (Get-FileHash $PSCommandPath).Hash)
$identity = ([BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($fingerprint))).Replace('-', '').ToLowerInvariant()

New-Item -ItemType Directory -Force $work | Out-Null

if (!(Test-Path "$work/identity") -or (Get-Content "$work/identity" -Raw) -ne $identity) {
    # Only generated files inside the workspace are replaced.
    $resolved = [IO.Path]::GetFullPath($runtime)

    if (!$resolved.StartsWith([IO.Path]::GetFullPath((Join-Path $PWD 'target/runtime')) + [IO.Path]::DirectorySeparatorChar)) {
        throw 'Generated directory is outside target/runtime.'
    }

    if (Test-Path $runtime) {
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }

    New-Item -ItemType Directory $runtime | Out-Null

    foreach ($name in @('luals', 'ffmpeg')) {
        $dependency = $spec.$name
        $archive = Join-Path $work ([IO.Path]::GetFileName($dependency.url))

        if (!(Test-Path $archive) -or (Get-FileHash $archive).Hash.ToLowerInvariant() -ne $dependency.sha256) {
            Run $curl @('--fail', '--location', '--retry', '3', '--output', "$archive.partial", $dependency.url)

            if ((Get-FileHash "$archive.partial").Hash.ToLowerInvariant() -ne $dependency.sha256) {
                throw "Invalid SHA-256 for $name."
            }

            Move-Item -LiteralPath "$archive.partial" -Destination $archive -Force
        }

        $destination = Join-Path $runtime $name
        New-Item -ItemType Directory $destination | Out-Null

        Run tar @('-xf', $archive, '-C', $destination)

        if ($name -eq 'ffmpeg') {
            $folder = @(Get-ChildItem $destination -Directory)

            if ($folder.Count -ne 1) {
                throw 'Unexpected FFmpeg archive layout.'
            }

            Get-ChildItem $folder[0].FullName | Move-Item -Destination $destination
            Remove-Item -LiteralPath $folder[0].FullName

            # Keep licenses and documentation alongside the executable.
            Get-ChildItem "$destination/bin" | Where-Object Name -NE "ffmpeg$suffix" | Remove-Item -Force
        }
    }

    foreach ($path in @("luals/bin/lua-language-server$suffix", "ffmpeg/bin/ffmpeg$suffix")) {
        if (!(Test-Path (Join-Path $runtime $path))) {
            throw "Missing dependency: $path."
        }
    }

    Run tar @('-czf', "$bundle.partial", '-C', $runtime, '.')
    Move-Item -LiteralPath "$bundle.partial" -Destination $bundle -Force

    [IO.File]::WriteAllText("$work/identity", $identity)
}

if ($Task -eq 'prepare') {
    return
}

$env:KINEMATIC_BUNDLE = $bundle
$env:KINEMATIC_DEPENDENCIES_ID = $identity
$savedFlags = $env:RUSTFLAGS

try {
    if ($platform -eq 'windows') {
        $env:RUSTFLAGS = "$savedFlags -C target-feature=+crt-static"
    }

    Run cargo @('build', '--release', '--locked', '--target', $target, '--features', 'bundled')
} finally {
    $env:RUSTFLAGS = $savedFlags
}

$dist = Join-Path $PWD 'target/dist'
New-Item -ItemType Directory -Force $dist | Out-Null
$output = Join-Path $dist "kinematic-$version-$platform-x64$suffix"
Copy-Item "target/$target/release/kinematic$suffix" $output -Force

if ($platform -eq 'windows') {
    $vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    $dumpbin = Get-ChildItem "$installation/VC/Tools/MSVC/*/bin/Hostx64/x64/dumpbin.exe" | Select-Object -First 1 -ExpandProperty FullName
    $imports = & $dumpbin /DEPENDENTS $output

    if ($LASTEXITCODE -ne 0 -or ($imports -match '(?i)^\s+(SDL|lua|skia|msvcp|vcruntime)[\w.-]*\.dll\s*$')) {
        throw 'The executable still imports a bundled library or the Visual C++ runtime.'
    }
} else {
    $imports = & ldd $output

    if ($LASTEXITCODE -ne 0 -or ($imports -match 'not found|libSDL|liblua|libskia')) {
        throw 'The executable has unresolved or non-static application libraries.'
    }
}

# The smoke test uses virtual audio, an empty user directory and no PATH.
$isolated = Join-Path $work ([guid]::NewGuid().ToString())
New-Item -ItemType Directory $isolated | Out-Null
$executable = Join-Path $isolated "kinematic$suffix"
Copy-Item $output $executable
$saved = @{}

foreach ($key in @('PATH', 'APPDATA', 'XDG_DATA_HOME', 'KINEMATIC_LUALS', 'SDL_AUDIODRIVER')) {
    $saved[$key] = [Environment]::GetEnvironmentVariable($key)
}

try {
    $env:PATH = ''
    $env:KINEMATIC_LUALS = $null
    $env:SDL_AUDIODRIVER = 'dummy'
    $env:APPDATA = $isolated
    $env:XDG_DATA_HOME = $isolated
    Push-Location $isolated

    try {
        Run $executable @('--self-test')

        # Repeat with a warm cache to exercise reuse.
        Run $executable @('--self-test')
    } finally {
        Pop-Location
    }
} finally {
    foreach ($key in $saved.Keys) {
        [Environment]::SetEnvironmentVariable($key, $saved[$key])
    }
}

$hash = (Get-FileHash $output).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$output.sha256", "$hash  $([IO.Path]::GetFileName($output))`n")
Write-Host "Validated distribution: $output."
