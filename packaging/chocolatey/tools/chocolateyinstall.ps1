$ErrorActionPreference = 'Stop'
$version = '1.0.0'
$url = "https://github.com/PhotoDotPi/pico/releases/download/pico-v$version/pico-$version-x86_64-pc-windows-msvc.zip"
$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$packageArgs = @{
  packageName    = 'pico'
  url            = $url
  unzipLocation  = $toolsDir
  checksum       = 'REPLACE_WITH_SHA256'
  checksumType   = 'sha256'
}
Install-ChocolateyZipPackage @packageArgs
