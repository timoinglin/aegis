# scripts/capture-aegis.ps1
#
# Capture the Aegis window with PrintWindow flag 2 (PW_RENDERFULLCONTENT, the
# magic flag that actually works on WebView2/Chromium surfaces). Used by
# regen-screenshots.ps1 — but you can also call it standalone:
#
#   .\scripts\capture-aegis.ps1 -OutFile assets\img\screenshots\status.png
#
# Captures whatever is currently shown in the foreground Aegis window. Saves
# a PNG of the client area only (no titlebar / border noise).

param(
    [Parameter(Mandatory)][string]$OutFile
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$source = @'
using System;
using System.Runtime.InteropServices;

public static class Win {
    [DllImport("user32.dll")]   public static extern IntPtr FindWindow(string c, string n);
    [DllImport("user32.dll")]   public static extern bool  PrintWindow(IntPtr h, IntPtr d, uint flags);
    [DllImport("user32.dll")]   public static extern bool  GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")]   public static extern bool  SetForegroundWindow(IntPtr h);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
}
'@
if (-not ('Win' -as [type])) { Add-Type -TypeDefinition $source }

# Find the Aegis window. It uses "Aegis" as its title from tauri.conf.json.
$hwnd = [Win]::FindWindow($null, "Aegis")
if ($hwnd -eq [IntPtr]::Zero) { throw "Couldn't find an Aegis window — is the app running?" }

[Win]::SetForegroundWindow($hwnd) | Out-Null
Start-Sleep -Milliseconds 200

# Size by client area, not whole-window, so we don't capture the titlebar twice.
$r = New-Object Win+RECT
[Win]::GetClientRect($hwnd, [ref]$r) | Out-Null
$w = $r.R - $r.L; $h = $r.B - $r.T
if ($w -le 0 -or $h -le 0) { throw "Window has no client area — is it minimized?" }

$bmp = New-Object System.Drawing.Bitmap($w, $h)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$dc = $g.GetHdc()
# flag 2 = PW_RENDERFULLCONTENT, required for WebView2 / Chromium to render.
$ok = [Win]::PrintWindow($hwnd, $dc, 2)
$g.ReleaseHdc($dc); $g.Dispose()
if (-not $ok) { throw "PrintWindow returned false." }

# Make sure the destination folder exists.
$dir = Split-Path -Parent $OutFile
if ($dir -and -not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }

$bmp.Save($OutFile, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

Write-Output "Saved $OutFile ($($w)x$($h))"
