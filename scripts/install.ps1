# Bodhya Installer for Windows (PowerShell)
# Version: 1.0
# License: MIT

#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

# Configuration
$RepoUrl = "https://github.com/vijayabose/bodhya"
$InstallDir = "$env:LOCALAPPDATA\Programs\Bodhya"
$BodhyaHome = "$env:USERPROFILE\.bodhya"

# Print colored message
function Write-ColoredMessage {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Message,
        [Parameter(Mandatory=$false)]
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

# Print section header
function Write-Header {
    param([string]$Title)
    Write-Host ""
    Write-ColoredMessage "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -Color Blue
    Write-ColoredMessage $Title -Color Blue
    Write-ColoredMessage "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -Color Blue
    Write-Host ""
}

# Check if command exists
function Test-CommandExists {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

# Check prerequisites
function Test-Prerequisites {
    Write-Header "Checking Prerequisites"

    # Check for Rust
    if (-not (Test-CommandExists "rustc")) {
        Write-ColoredMessage "⚠ Rust is not installed." -Color Yellow
        Write-ColoredMessage "Installing Rust via rustup..." -Color Yellow

        # Download and run rustup-init.exe
        $rustupUrl = "https://win.rustup.rs/x86_64"
        $rustupPath = "$env:TEMP\rustup-init.exe"

        Write-ColoredMessage "Downloading rustup-init.exe..." -Color Blue
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath

        Write-ColoredMessage "Running rustup-init..." -Color Blue
        Start-Process -FilePath $rustupPath -ArgumentList "-y" -Wait

        # Update PATH for current session
        $env:Path += ";$env:USERPROFILE\.cargo\bin"

        Write-ColoredMessage "✓ Rust installed successfully" -Color Green
    } else {
        $rustVersion = (rustc --version).Split(" ")[1]
        Write-ColoredMessage "✓ Rust installed: $rustVersion" -Color Green
    }

    # Check Cargo
    if (-not (Test-CommandExists "cargo")) {
        Write-ColoredMessage "✗ Error: Cargo not found. Please install Rust." -Color Red
        exit 1
    }

    # Check Git
    if (-not (Test-CommandExists "git")) {
        Write-ColoredMessage "✗ Error: Git is required but not installed." -Color Red
        Write-ColoredMessage "  Please install Git from: https://git-scm.com/download/win" -Color Yellow
        exit 1
    } else {
        Write-ColoredMessage "✓ Git installed" -Color Green
    }

    Write-ColoredMessage "✓ All prerequisites satisfied" -Color Green
}

# Clone repository
function Get-Repository {
    Write-Header "Setting Up Repository"

    $tmpDir = [System.IO.Path]::GetTempPath()
    $buildDir = Join-Path $tmpDir "bodhya-$(Get-Random)"

    Write-ColoredMessage "Cloning repository to temporary directory..." -Color Blue
    git clone --depth 1 $RepoUrl $buildDir

    if (-not $?) {
        Write-ColoredMessage "✗ Error: Failed to clone repository" -Color Red
        exit 1
    }

    return $buildDir
}

# Build Bodhya
function Build-Bodhya {
    param([string]$BuildDir)

    Write-Header "Building Bodhya"

    Push-Location $BuildDir

    try {
        Write-ColoredMessage "Building in release mode (this may take a few minutes)..." -Color Blue
        cargo build --release -p bodhya-cli

        if (-not $?) {
            Write-ColoredMessage "✗ Error: Build failed" -Color Red
            exit 1
        }

        Write-ColoredMessage "✓ Build successful" -Color Green
    }
    finally {
        Pop-Location
    }
}

# Install binary
function Install-Binary {
    param([string]$BuildDir)

    Write-Header "Installing Bodhya"

    # Create install directory
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    # Copy binary
    $sourceBinary = Join-Path $BuildDir "target\release\bodhya.exe"
    $targetBinary = Join-Path $InstallDir "bodhya.exe"

    Write-ColoredMessage "Installing binary to $InstallDir..." -Color Blue
    Copy-Item -Path $sourceBinary -Destination $targetBinary -Force

    Write-ColoredMessage "✓ Binary installed to $targetBinary" -Color Green
}

# Setup PATH
function Set-EnvironmentPath {
    Write-Header "Configuring PATH"

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")

    if ($userPath -notlike "*$InstallDir*") {
        Write-ColoredMessage "Adding $InstallDir to PATH..." -Color Blue

        $newPath = "$userPath;$InstallDir"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")

        # Update PATH for current session
        $env:Path += ";$InstallDir"

        Write-ColoredMessage "✓ PATH updated" -Color Green
        Write-ColoredMessage "  Please restart your PowerShell session for PATH changes to take effect" -Color Yellow
    } else {
        Write-ColoredMessage "✓ $InstallDir already in PATH" -Color Green
    }
}

# Initialize Bodhya
function Initialize-Bodhya {
    Write-Header "Initializing Bodhya"

    if (Test-Path $BodhyaHome) {
        Write-ColoredMessage "⚠ Bodhya is already initialized at $BodhyaHome" -Color Yellow
        $response = Read-Host "Do you want to reinitialize? (y/N)"

        if ($response -ne "y" -and $response -ne "Y") {
            Write-ColoredMessage "Skipping initialization" -Color Blue
            return
        }
    }

    # Run init command
    Write-ColoredMessage "Running: bodhya init --profile full" -Color Blue

    $bodhyaExe = Join-Path $InstallDir "bodhya.exe"

    try {
        & $bodhyaExe init --profile full

        if ($?) {
            Write-ColoredMessage "✓ Bodhya initialized successfully" -Color Green
        } else {
            Write-ColoredMessage "⚠ Initialization completed with warnings" -Color Yellow
        }
    }
    catch {
        Write-ColoredMessage "⚠ Initialization skipped (you can run 'bodhya init' later)" -Color Yellow
    }
}

# Cleanup
function Remove-TempFiles {
    param([string]$TmpDir)

    if (Test-Path $TmpDir) {
        Write-ColoredMessage "Cleaning up temporary files..." -Color Blue
        Remove-Item -Path $TmpDir -Recurse -Force
    }
}

# Print success message
function Write-Success {
    Write-Header "Installation Complete!"

    Write-ColoredMessage "✅ Bodhya has been successfully installed!" -Color Green
    Write-Host ""
    Write-ColoredMessage "Installation directory: $InstallDir" -Color Blue
    Write-ColoredMessage "Configuration directory: $BodhyaHome" -Color Blue
    Write-Host ""
    Write-ColoredMessage "Next steps:" -Color Yellow
    Write-Host "  1. Restart your PowerShell session"
    Write-Host "  2. Verify installation: bodhya --version"
    Write-Host "  3. View help: bodhya --help"
    Write-Host "  4. Run a task: bodhya run --domain code --task 'Create hello world'"
    Write-Host ""
    Write-ColoredMessage "Documentation:" -Color Blue
    Write-Host "  • User Guide: https://github.com/vijayabose/bodhya/blob/main/USER_GUIDE.md"
    Write-Host "  • Developer Guide: https://github.com/vijayabose/bodhya/blob/main/DEVELOPER_GUIDE.md"
    Write-Host ""
    Write-ColoredMessage "Happy coding with Bodhya! 🚀" -Color Green
}

# Main installation flow
function Main {
    Write-Header "Bodhya Installer v1.0"
    Write-ColoredMessage "This script will install Bodhya on your system." -Color Blue
    Write-Host ""

    # Check if running as Administrator
    $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

    if (-not $isAdmin) {
        Write-ColoredMessage "⚠ Warning: Not running as Administrator" -Color Yellow
        Write-ColoredMessage "  Some operations may fail without elevated privileges" -Color Yellow
        Write-Host ""
    }

    # Check prerequisites
    Test-Prerequisites

    # Setup repository
    $buildDir = Get-Repository

    try {
        # Build
        Build-Bodhya -BuildDir $buildDir

        # Install
        Install-Binary -BuildDir $buildDir

        # Setup PATH
        Set-EnvironmentPath

        # Initialize
        Initialize-Bodhya

        # Success
        Write-Success
    }
    finally {
        # Cleanup
        Remove-TempFiles -TmpDir $buildDir
    }
}

# Run main function
try {
    Main
}
catch {
    Write-ColoredMessage "✗ Error: $_" -Color Red
    Write-ColoredMessage "Installation failed. Please check the error message above." -Color Red
    exit 1
}
