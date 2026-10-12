$ErrorActionPreference = 'Stop'

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$target = 'x86_64-pc-windows-msvc'
# Resolve the Rust tools up front so missing prerequisites produce a clear error.
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
$rustup = Get-Command rustup -ErrorAction SilentlyContinue

# Run a native executable and turn its exit code into a terminating PowerShell error.
function Invoke-NativeCommand {
    param(
        [Parameter(Mandatory)] [scriptblock] $Command,
        [Parameter(Mandatory)] [string] $FailureMessage
    )

    $previousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        & $Command
        $exitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $previousErrorActionPreference
    }

    if ($exitCode -ne 0) {
        throw $FailureMessage
    }
}

if (-not $cargo -or -not $rustup) {
    throw 'Install Rust with rustup (including the MSVC C++ build tools) before running this script.'
}

# Run Cargo from the repository root regardless of the caller's current directory.
Push-Location $repositoryRoot -ErrorAction Stop
try {
    # Read the package version for the name of the copied release artifact.
    $metadataJson = Invoke-NativeCommand -FailureMessage 'cargo metadata failed.' -Command {
        & cargo metadata --locked --no-deps --format-version 1
    }

    $metadata = $metadataJson | ConvertFrom-Json -ErrorAction Stop
    $package = $metadata.packages | Where-Object { $_.name -eq 'worgen_x' } | Select-Object -First 1
    if (-not $package) {
        throw 'Could not find the worgen_x package in Cargo metadata.'
    }

    # Install the Windows MSVC target if it is not already available.
    Invoke-NativeCommand -FailureMessage "Could not install Rust target $target." -Command {
        & rustup target add $target
    }

    $outputDirectory = Join-Path $repositoryRoot 'dist'
    New-Item -ItemType Directory -Path $outputDirectory -Force -ErrorAction Stop | Out-Null

    # Build each mode separately because the cli and gui features are mutually exclusive.
    foreach ($mode in @('cli', 'gui')) {
        Invoke-NativeCommand -FailureMessage "Cargo $mode release build failed." -Command {
            & cargo build --locked --release --features $mode --target $target
        }

        # Cargo writes both feature variants to the same target path, so copy each before rebuilding.
        $binaryPath = Join-Path $repositoryRoot "target/$target/release/worgen_x.exe"
        if (-not (Test-Path $binaryPath -PathType Leaf)) {
            throw "Expected $mode build output not found: $binaryPath"
        }

        $artifactPath = Join-Path $outputDirectory "worgenx_${mode}_$($package.version)_amd64.exe"
        Copy-Item -Path $binaryPath -Destination $artifactPath -Force -ErrorAction Stop
        Write-Host "Windows $mode executable created: $artifactPath"
    }
}
finally {
    # Restore the caller's original working directory even if a step fails.
    Pop-Location
}