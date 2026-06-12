Add-Type -AssemblyName System.Drawing
$sizes = @(32, 128, 256, 512)
$iconDir = "c:\Users\gonib\Downloads\ClipVault\src-tauri\icons"
if (!(Test-Path $iconDir)) { New-Item -ItemType Directory -Path $iconDir | Out-Null }
foreach ($s in $sizes) {
    $bmp = New-Object System.Drawing.Bitmap($s, $s)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = 'AntiAlias'
    $g.Clear([System.Drawing.Color]::Transparent)
    $brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(79,70,229))
    $g.FillEllipse($brush, 0, 0, $s-1, $s-1)
    $font = New-Object System.Drawing.Font('Segoe UI', [int]($s*0.4), [System.Drawing.FontStyle]::Bold)
    $textBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
    $fmt = New-Object System.Drawing.StringFormat
    $fmt.Alignment = 'Center'
    $fmt.LineAlignment = 'Center'
    $rect = New-Object System.Drawing.RectangleF(0, 0, $s, $s)
    $g.DrawString('CV', $font, $textBrush, $rect, $fmt)
    $g.Dispose()
    $bmp.Save("$iconDir\icon-$($s)x$($s).png", [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Host "wrote $iconDir\icon-$($s)x$($s).png"
}
# Also create a tray icon (light + dark variants)
$trayLight = New-Object System.Drawing.Bitmap(32, 32)
$g = [System.Drawing.Graphics]::FromImage($trayLight)
$g.SmoothingMode = 'AntiAlias'
$g.Clear([System.Drawing.Color]::Transparent)
$brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(82,82,91))
$g.FillEllipse($brush, 0, 0, 31, 31)
$font = New-Object System.Drawing.Font('Segoe UI', 13, [System.Drawing.FontStyle]::Bold)
$tb = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
$fmt = New-Object System.Drawing.StringFormat
$fmt.Alignment = 'Center'; $fmt.LineAlignment = 'Center'
$rect = New-Object System.Drawing.RectangleF(0, 0, 32, 32)
$g.DrawString('CV', $font, $tb, $rect, $fmt)
$g.Dispose()
$trayLight.Save("$iconDir\tray-light.png", [System.Drawing.Imaging.ImageFormat]::Png)
$trayLight.Dispose()

$trayDark = New-Object System.Drawing.Bitmap(32, 32)
$g = [System.Drawing.Graphics]::FromImage($trayDark)
$g.SmoothingMode = 'AntiAlias'
$g.Clear([System.Drawing.Color]::Transparent)
$brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(244,244,245))
$g.FillEllipse($brush, 0, 0, 31, 31)
$font = New-Object System.Drawing.Font('Segoe UI', 13, [System.Drawing.FontStyle]::Bold)
$tb = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(24,24,27))
$fmt = New-Object System.Drawing.StringFormat
$fmt.Alignment = 'Center'; $fmt.LineAlignment = 'Center'
$rect = New-Object System.Drawing.RectangleF(0, 0, 32, 32)
$g.DrawString('CV', $font, $tb, $rect, $fmt)
$g.Dispose()
$trayDark.Save("$iconDir\tray-dark.png", [System.Drawing.Imaging.ImageFormat]::Png)
$trayDark.Dispose()

# Generate a 128x128@2x.png and 32x32.png with the standard names
Copy-Item "$iconDir\icon-32x32.png" "$iconDir\32x32.png" -Force
Copy-Item "$iconDir\icon-128x128.png" "$iconDir\128x128.png" -Force
Copy-Item "$iconDir\icon-256x256.png" "$iconDir\128x128@2x.png" -Force
Copy-Item "$iconDir\icon-256x256.png" "$iconDir\icon.png" -Force

# Generate a simple .ico (32x32 + 128x128)
$ico = New-Object System.Drawing.Bitmap(256, 256)
$g = [System.Drawing.Graphics]::FromImage($ico)
$g.SmoothingMode = 'AntiAlias'
$g.Clear([System.Drawing.Color]::Transparent)
$brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(79,70,229))
$g.FillEllipse($brush, 0, 0, 255, 255)
$font = New-Object System.Drawing.Font('Segoe UI', 100, [System.Drawing.FontStyle]::Bold)
$tb = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
$fmt = New-Object System.Drawing.StringFormat
$fmt.Alignment = 'Center'; $fmt.LineAlignment = 'Center'
$rect = New-Object System.Drawing.RectangleF(0, 0, 256, 256)
$g.DrawString('CV', $font, $tb, $rect, $fmt)
$g.Dispose()
$icon = [System.Drawing.Icon]::FromHandle($ico.GetHicon())
$fs = [System.IO.File]::Create("$iconDir\icon.ico")
$icon.Save($fs)
$fs.Close()
$ico.Dispose()

Write-Host "Done."
