$ErrorActionPreference = 'Stop'
$repo = 'PhotoDotPi/pico'
$installDir = if ($env:PICO_INSTALL_DIR) { $env:PICO_INSTALL_DIR } else { Join-Path $env:USERPROFILE 'bin' }

$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest"
$tag = $release.tag_name
$version = $tag -replace '^pico-v', ''
$asset = "pico-$version-x86_64-pc-windows-msvc.zip"
$url = "https://github.com/$repo/releases/download/$tag/$asset"

$tmp = Join-Path $env:TEMP "pico-install"
if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp }
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
$zip = Join-Path $tmp $asset
Write-Host "Downloading $url ..."
Invoke-WebRequest -Uri $url -OutFile $zip
Expand-Archive -Path $zip -DestinationPath $tmp -Force

New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item (Join-Path $tmp "pico-$version-x86_64-pc-windows-msvc\pico.exe") (Join-Path $installDir 'pico.exe') -Force
Write-Host "Installed pico to $installDir\pico.exe"
Write-Host "Ensure $installDir is on your PATH."
