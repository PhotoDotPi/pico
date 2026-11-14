$ErrorActionPreference = 'Stop'
$version = '1.0.0'
$url = "https://github.com/PhotoDotPi/pico/releases/download/pico-v$version/pico-$version-x86_64-pc-windows-msvc.zip"
$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$packageArgs = @{
  packageName    = 'pico'
  url            = $url
  unzipLocation  = $toolsDir
  checksum       = '1eaba2fcf1cc462d97893c24cd847fa7bff5dc7fb60b514cf0e1042a1357c3dc'
  checksumType   = 'sha256'
}
Install-ChocolateyZipPackage @packageArgs
