$RepoOwner = "iffse"
$RepoName  = "pay-respects"
$InstallDir = "$HOME\AppData\Roaming\pay-respects"

$SysArch = $env:PROCESSOR_ARCHITECTURE
$PackageArch = switch ($SysArch) {
	"AMD64" { "x86_64" }
	"ARM64" { "aarch64" }
	Default { throw "Unsupported architecture: $SysArch" }
}

Write-Host "Detected Architecture: $SysArch" -ForegroundColor Cyan

$ApiUrl = "https://api.github.com/repos/$RepoOwner/$RepoName/releases/latest"
try {
	$Release = Invoke-RestMethod -Uri $ApiUrl -Method Get -Headers @{"User-Agent"="PowerShell-Downloader"}
} catch {
	throw "Failed to fetch release info: $_"
}

$Asset = $Release.assets | Where-Object { $_.name -like "*$PackageArch*" -and $_.name -like "*pc-windows*" }

if (-not $Asset) {
	throw "Could not find a asset for $PackageArch in version $($Release.tag_name)"
}

$DownloadUrl = $Asset.browser_download_url
$ZipFile = Join-Path $env:TEMP $Asset.name

Write-Host "Downloading $($Asset.name)..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipFile

if (-not (Test-Path $InstallDir)) {
	New-Item -Path $InstallDir -ItemType Directory | Out-Null
}

Write-Host "Extracting to $InstallDir..." -ForegroundColor Yellow
Expand-Archive -Path $ZipFile -DestinationPath $InstallDir -Force

# add to user level PATH
Write-Host "Updating PATH environment variable..." -ForegroundColor Cyan
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")

if ($CurrentPath -notlike "*$InstallDir*") {
	$NewPath = "$CurrentPath;$InstallDir"
		[Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
		$env:Path = $NewPath # Update current session path
			Write-Host "Successfully added $InstallDir to User PATH." -ForegroundColor Green
}

Write-Host "Installation of $($Release.tag_name) finished!"

Write-Host "pay-respects has an optional AI module to provide suggestions when no rules match"
Write-Host "The module works out-of-the-box with no data collection"
Write-Host "Do you want to keep the AI module? (Y/n)" -ForegroundColor Yellow
$Response = Read-Host
if ($Response -eq "n" -or $Response -eq "N") {
	$AIModulePath = Join-Path $InstallDir "_pay-respects-fallback-100-request-ai.exe"
	if (Test-Path $AIModulePath) {
		Remove-Item -Path $AIModulePath -Recurse -Force
		Write-Host "AI module removed." -ForegroundColor Green
	}
}
