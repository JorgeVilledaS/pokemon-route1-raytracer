param([string]$Path='output/frames/material_preview.ppm')
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
$bytes=[IO.File]::ReadAllBytes((Resolve-Path $Path).Path)
$head=[Text.Encoding]::ASCII.GetString($bytes,0,[Math]::Min(128,$bytes.Length))
$match=[regex]::Match($head,'^P6\s+(\d+)\s+(\d+)\s+255\n')
if(!$match.Success){throw 'Expected P6 RGB header'}
$width=[int]$match.Groups[1].Value;$height=[int]$match.Groups[2].Value;$offset=$match.Length
$bitmap=New-Object System.Drawing.Bitmap($width,$height)
try {
    for($y=0;$y -lt $height;$y++){for($x=0;$x -lt $width;$x++){
        $i=$offset+($y*$width+$x)*3
        $bitmap.SetPixel($x,$y,[System.Drawing.Color]::FromArgb($bytes[$i],$bytes[$i+1],$bytes[$i+2]))
    }}
    $out=[IO.Path]::ChangeExtension((Resolve-Path $Path).Path,'.png')
    $bitmap.Save($out,[System.Drawing.Imaging.ImageFormat]::Png)
    Write-Output $out
} finally {$bitmap.Dispose()}
