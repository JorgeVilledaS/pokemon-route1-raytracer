param([Parameter(Mandatory=$true)][string]$InputPath,[Parameter(Mandatory=$true)][string]$OutputPath)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Drawing
$bitmap=[System.Drawing.Bitmap]::FromFile((Resolve-Path -LiteralPath $InputPath).Path)
try {
    $stream=[IO.File]::Create([IO.Path]::GetFullPath($OutputPath))
    try {
        $header=[Text.Encoding]::ASCII.GetBytes("P6`n$($bitmap.Width) $($bitmap.Height)`n255`n")
        $stream.Write($header,0,$header.Length)
        for($y=0;$y -lt $bitmap.Height;$y++){for($x=0;$x -lt $bitmap.Width;$x++){
            $c=$bitmap.GetPixel($x,$y)
            if($c.A -ne 255){throw 'Usa PNG RGB opaco: PPM no admite transparencia.'}
            $stream.WriteByte($c.R);$stream.WriteByte($c.G);$stream.WriteByte($c.B)
        }}
    } finally {$stream.Dispose()}
} finally {$bitmap.Dispose()}
