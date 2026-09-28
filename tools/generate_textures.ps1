param(
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '..\assets\textures')
)

$ErrorActionPreference = 'Stop'
$Size = 32
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null

function Clamp-Byte([double]$Value) {
    [byte][Math]::Round([Math]::Max(0.0, [Math]::Min(255.0, $Value)))
}

function Grain([int]$X, [int]$Y, [int]$Seed) {
    (($X * 73 + $Y * 151 + $X * $Y * 19 + $Seed * 199) % 31) - 15
}

function Write-Ppm([string]$Name, [scriptblock]$Painter) {
    $header = [Text.Encoding]::ASCII.GetBytes("P6`n$Size $Size`n255`n")
    $pixels = New-Object byte[] ($Size * $Size * 3)
    for ($y = 0; $y -lt $Size; $y++) {
        for ($x = 0; $x -lt $Size; $x++) {
            $rgb = & $Painter $x $y
            $offset = ($x + $y * $Size) * 3
            $pixels[$offset] = Clamp-Byte $rgb[0]
            $pixels[$offset + 1] = Clamp-Byte $rgb[1]
            $pixels[$offset + 2] = Clamp-Byte $rgb[2]
        }
    }
    $data = New-Object byte[] ($header.Length + $pixels.Length)
    [Array]::Copy($header, 0, $data, 0, $header.Length)
    [Array]::Copy($pixels, 0, $data, $header.Length, $pixels.Length)
    [IO.File]::WriteAllBytes((Join-Path $OutputDirectory $Name), $data)
}

function Write-NormalMap([string]$Name, [scriptblock]$Height, [double]$Strength) {
    Write-Ppm $Name {
        param($x, $y)
        $left = & $Height (($x - 1 + $Size) % $Size) $y
        $right = & $Height (($x + 1) % $Size) $y
        $up = & $Height $x (($y - 1 + $Size) % $Size)
        $down = & $Height $x (($y + 1) % $Size)
        $nx = ($left - $right) * $Strength
        $ny = ($up - $down) * $Strength
        $nz = 1.0
        $length = [Math]::Sqrt($nx * $nx + $ny * $ny + $nz * $nz)
        @((128 + 127 * $nx / $length), (128 + 127 * $ny / $length), (128 + 127 * $nz / $length))
    }
}

# Paleta común: verdes menta, arena cálida, umber, blanco crema y gris azulado.
Write-Ppm 'grass_top.ppm' {
    param($x, $y)
    $g = Grain $x $y 1
    $sprig = (($x * 3 + $y * 5) % 17 -eq 0) -or (($x + $y * 7) % 29 -eq 2)
    if ($sprig) { @((64 + $g), (151 + $g), (79 + $g)) } else { @((103 + $g), (190 + $g), (108 + $g)) }
}

Write-Ppm 'grass_tall.ppm' {
    param($x, $y)
    $g = Grain $x $y 2
    $blade = (($x + [int]($y / 3)) % 7 -eq 0) -or (($x * 2 - $y) % 13 -eq 1)
    if ($blade) { @((42 + $g), (125 + $g), (64 + $g)) } else { @((78 + $g), (166 + $g), (87 + $g)) }
}

Write-Ppm 'dirt_path.ppm' {
    param($x, $y)
    $g = Grain $x $y 3
    $pebble = (($x * 5 + $y * 11) % 37 -lt 3)
    if ($pebble) { @((166 + $g), (128 + $g), (76 + $g)) } else { @((218 + $g), (187 + $g), (112 + $g)) }
}

$fenceHeight = { param($x, $y) 0.55 + 0.18 * [Math]::Sin($x * [Math]::PI / 4.0) + (Grain $x $y 4) / 180.0 }
Write-Ppm 'wood_fence.ppm' {
    param($x, $y)
    $g = Grain $x $y 4
    $seam = $x % 8 -eq 0 -or $x % 8 -eq 7
    $chip = (($x * 7 + $y * 3) % 41 -eq 0)
    if ($seam) { @((154 + $g), (145 + $g), (124 + $g)) }
    elseif ($chip) { @((190 + $g), (177 + $g), (146 + $g)) }
    else { @((232 + $g / 2), (226 + $g / 2), (199 + $g / 2)) }
}
Write-NormalMap 'wood_fence_normal.ppm' $fenceHeight 1.7

$barkHeight = { param($x, $y) 0.5 + 0.23 * [Math]::Sin($x * [Math]::PI / 3.7) + (Grain $x $y 5) / 140.0 }
Write-Ppm 'bark.ppm' {
    param($x, $y)
    $g = Grain $x $y 5
    $groove = (($x + [int]($y / 5)) % 7 -eq 0)
    if ($groove) { @((72 + $g), (48 + $g), (35 + $g)) } else { @((126 + $g), (82 + $g), (49 + $g)) }
}
Write-NormalMap 'bark_normal.ppm' $barkHeight 2.2

Write-Ppm 'leaves.ppm' {
    param($x, $y)
    $g = Grain $x $y 6
    $cluster = (([int]($x / 4) + [int]($y / 4) * 3) % 4)
    switch ($cluster) {
        0 { @((43 + $g), (126 + $g), (67 + $g)) }
        1 { @((64 + $g), (157 + $g), (76 + $g)) }
        2 { @((87 + $g), (181 + $g), (91 + $g)) }
        default { @((55 + $g), (143 + $g), (71 + $g)) }
    }
}

Write-Ppm 'sign_wood.ppm' {
    param($x, $y)
    $g = Grain $x $y 7
    $seam = $y % 8 -eq 0
    $border = $x -lt 2 -or $x -gt 29 -or $y -lt 2 -or $y -gt 29
    if ($border) { @((82 + $g), (50 + $g), (32 + $g)) }
    elseif ($seam) { @((119 + $g), (72 + $g), (39 + $g)) }
    else { @((170 + $g), (105 + $g), (52 + $g)) }
}

Write-Ppm 'sign_glow.ppm' {
    param($x, $y)
    $mark = (($x -ge 14 -and $x -le 17 -and $y -ge 7 -and $y -le 21) -or
             ($x -ge 14 -and $x -le 17 -and $y -ge 25 -and $y -le 28))
    if ($mark) { @(255, 107, 34) } else { @(0, 0, 0) }
}

$waterHeight = { param($x, $y) 0.5 + 0.12 * [Math]::Sin(($x + $y * 0.45) * [Math]::PI / 5.0) }
Write-Ppm 'water.ppm' {
    param($x, $y)
    $wave = (($x + $y * 2) % 11 -lt 2)
    $g = Grain $x $y 8
    if ($wave) { @((105 + $g), (204 + $g), (224 + $g)) } else { @((45 + $g), (142 + $g), (190 + $g)) }
}
Write-NormalMap 'water_normal.ppm' $waterHeight 1.1

Write-Ppm 'rock.ppm' {
    param($x, $y)
    $g = Grain $x $y 9
    $vein = (($x * 3 + $y * 2) % 19 -lt 2)
    if ($vein) { @((105 + $g), (119 + $g), (127 + $g)) } else { @((145 + $g), (158 + $g), (159 + $g)) }
}

Write-Output "Generated original $Size x $Size PPM P6 texture set in $OutputDirectory"
