param([Parameter(Mandatory=$true)][string]$Archive)
$ErrorActionPreference = 'Stop'
$destination = Join-Path $env:RUNNER_TEMP ([IO.Path]::GetRandomFileName())
Expand-Archive -LiteralPath $Archive -DestinationPath $destination
$executable = Join-Path $destination 'IDE_C--.exe'
if (-not (Test-Path $executable)) { throw 'Missing IDE_C--.exe' }
# Deliberately avoid the MSYS2 toolchain PATH: runtime DLLs must come from the ZIP.
$env:PATH = "$env:SystemRoot\System32;$env:SystemRoot"
$process = Start-Process -FilePath $executable -WorkingDirectory $destination -PassThru
try {
    $ready = $false
    for ($attempt = 0; $attempt -lt 40; $attempt++) {
        Start-Sleep -Milliseconds 250
        $process.Refresh()
        if ($process.HasExited) { throw "Application exited with code $($process.ExitCode)" }
        if ($process.MainWindowHandle -ne 0) { $ready = $true; break }
    }
    if (-not $ready) { throw 'No application window after 10 seconds' }
    Write-Output "IDE_C-- window ready: $($process.MainWindowTitle)"
} finally {
    if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force }
}
