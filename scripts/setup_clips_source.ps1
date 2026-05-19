$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath ".\\classes")) {
  throw "Eseguire script dal root del repo (cartella con classes/ !!!)."
}

$zip = "clips_642.zip"
$url = "https://sourceforge.net/projects/clipsrules/files/CLIPS/6.4.2/clips_core_source_642.zip/download"

Write-Host "Scaricando risorse CLIPS 6.4.2..."
curl.exe -L $url -o $zip

Remove-Item -Recurse -Force .\\clips_temp -ErrorAction SilentlyContinue
Expand-Archive -Force $zip -DestinationPath .\\clips_temp

New-Item -ItemType Directory -Force .\\clips_source | Out-Null
Copy-Item -Force -Recurse .\\clips_temp\\clips_core_source_642\\core\\* .\\clips_source\\

Remove-Item -Recurse -Force .\\clips_temp
Remove-Item -Force $zip

$clips = (Resolve-Path .\\clips_source).Path
Write-Host ""
Write-Host "OK. CLIPS sources copiate a: $clips"
Write-Host "Next (same PowerShell session):"
Write-Host "  `$env:CLIPS_SOURCE_DIR = `"$clips`""
Write-Host "  cargo run --features coco-server"

