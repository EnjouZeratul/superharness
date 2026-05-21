# Continuum Setup Script for PowerShell
# Run: ./setup.ps1

param(
    [string]$Provider = "anthropic",
    [string]$ApiKey = "",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

# Colors
$Green = "`e[32m"
$Yellow = "`e[33m"
$Cyan = "`e[36m"
$Reset = "`e[0m"

function Write-Header {
    param([string]$Text)
    Write-Host ""
    Write-Host "$Cyan========================================$Reset"
    Write-Host "$Cyan  $Text$Reset"
    Write-Host "$Cyan========================================$Reset"
    Write-Host ""
}

function Write-Success {
    param([string]$Text)
    Write-Host "$Green[OK]$Reset $Text"
}

function Write-Info {
    param([string]$Text)
    Write-Host "$Yellow[INFO]$Reset $Text"
}

# Main
Write-Header "Continuum Configuration Setup"

# 1. Create config directory
$ConfigDir = "$env:USERPROFILE\.continuum"
if (-not (Test-Path $ConfigDir)) {
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
    Write-Success "Created config directory: $ConfigDir"
} else {
    Write-Info "Config directory exists: $ConfigDir"
}

# 2. Copy config template
$ConfigFile = "$ConfigDir\config.toml"
$TemplateFile = "$PSScriptRoot\config.toml"

if ((Test-Path $ConfigFile) -and -not $Force) {
    Write-Info "Config file exists: $ConfigFile"
    Write-Info "Use -Force to overwrite"
} else {
    if (Test-Path $TemplateFile) {
        Copy-Item $TemplateFile $ConfigFile -Force
        Write-Success "Copied config template to: $ConfigFile"
    } else {
        Write-Info "Template not found, creating default config"
    }
}

# 3. Set environment variables
Write-Header "Environment Variables"

if ($ApiKey) {
    [Environment]::SetEnvironmentVariable("CONTINUUM_API_KEY", $ApiKey, "User")
    [Environment]::SetEnvironmentVariable("CONTINUUM_PROVIDER", $Provider, "User")
    Write-Success "Set CONTINUUM_API_KEY and CONTINUUM_PROVIDER"
} else {
    Write-Info "No API key provided. Set it manually:"
    Write-Host ""
    Write-Host "  `$env:CONTINUUM_API_KEY = 'your-api-key'"
    Write-Host "  [Environment]::SetEnvironmentVariable('CONTINUUM_API_KEY', 'your-api-key', 'User')"
    Write-Host ""
}

# 4. Verify
Write-Header "Verification"
Write-Host "Config directory: $ConfigDir"
Write-Host "Config file: $ConfigFile"
Write-Host "Provider: $Provider"

# 5. Test
Write-Header "Quick Test"
Write-Host "Testing Python SDK..."
python -c "from continuum_sdk import Agent, Config; c = Config.from_default(); print(f'Config: {c}')"

Write-Header "Setup Complete!"
Write-Host "Next steps:"
Write-Host "  1. Set your API key:"
Write-Host "     `$env:CONTINUUM_API_KEY = 'your-key'"
Write-Host ""
Write-Host "  2. Run Continuum:"
Write-Host "     continuum run 'your task'"
