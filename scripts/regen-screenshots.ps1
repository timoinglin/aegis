# scripts/regen-screenshots.ps1
#
# Drives the running Aegis window through each main tab and saves a fresh PNG
# per tab to assets\img\screenshots\. Assumes Aegis is already running.
#
# Sidebar layout from src/App.tsx (top to bottom):
#   Brand | Status | Server | Accounts | Characters | Backup | Restore
#   | Maintenance | Add-ons | [spacer] | Other tools | Settings | About
#
# Clicks are sent into the window's client area at known offsets. We force
# the window size + position first so click coords are predictable.

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

$src = @'
using System;
using System.Runtime.InteropServices;
public static class W {
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string n);
    [DllImport("user32.dll")] public static extern bool   SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool   GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool   ClientToScreen(IntPtr h, ref POINT p);
    [DllImport("user32.dll")] public static extern bool   SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int hh, uint flags);
    [DllImport("user32.dll")] public static extern void   mouse_event(uint flags, uint dx, uint dy, uint data, IntPtr extra);
    [DllImport("user32.dll")] public static extern bool   SetCursorPos(int x, int y);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
    [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
    public const uint MOUSEEVENTF_LEFTDOWN = 0x0002;
    public const uint MOUSEEVENTF_LEFTUP   = 0x0004;
}
'@
if (-not ('W' -as [type])) { Add-Type -TypeDefinition $src }

$hwnd = [W]::FindWindow($null, "Aegis")
if ($hwnd -eq [IntPtr]::Zero) { throw "No Aegis window - start the app first." }

# Force a known size + position so clicks land where we compute them to.
[W]::SetWindowPos($hwnd, [IntPtr]::Zero, 40, 40, 1000, 720, 0x40) | Out-Null
Start-Sleep -Milliseconds 300
[W]::SetForegroundWindow($hwnd) | Out-Null
Start-Sleep -Milliseconds 300

$rect = New-Object W+RECT
[W]::GetClientRect($hwnd, [ref]$rect) | Out-Null

function Click-Client([int]$cx, [int]$cy) {
    $p = New-Object W+POINT
    $p.X = $cx; $p.Y = $cy
    [W]::ClientToScreen($hwnd, [ref]$p) | Out-Null
    [W]::SetCursorPos($p.X, $p.Y) | Out-Null
    Start-Sleep -Milliseconds 80
    [W]::mouse_event([W]::MOUSEEVENTF_LEFTDOWN, 0, 0, 0, [IntPtr]::Zero)
    Start-Sleep -Milliseconds 40
    [W]::mouse_event([W]::MOUSEEVENTF_LEFTUP,   0, 0, 0, [IntPtr]::Zero)
    Start-Sleep -Milliseconds 400  # let the tab render
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$capture   = Join-Path $scriptDir "capture-aegis.ps1"
$shotDir   = Join-Path (Split-Path -Parent $scriptDir) "assets\img\screenshots"

# Health banner is ~52px tall. Brand block ~48px. Nav buttons follow at
# ~40px stride. X centered in the 192px sidebar (~96).
# Bottom group (Other tools / Settings / About) is anchored to the bottom.
$navX = 96
$tabs = @(
    @{ name="status";       y=140 },
    @{ name="server";       y=180 },
    @{ name="accounts";     y=220 },
    @{ name="characters";   y=260 },
    @{ name="backup";       y=300 },
    @{ name="restore";      y=340 },
    @{ name="maintenance";  y=380 },
    @{ name="addons";       y=420 },
    @{ name="other-tools";  y=564 },
    @{ name="settings";     y=604 },
    @{ name="about";        y=644 }
)

foreach ($tab in $tabs) {
    Click-Client $navX $tab.y
    & $capture -OutFile (Join-Path $shotDir ("{0}.png" -f $tab.name))
}

# Back to Status so the app's left in a tidy state.
Click-Client $navX 140
Write-Output "Done."
