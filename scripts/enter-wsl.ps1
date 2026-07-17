param(
    [string]$Distro = "Ubuntu"
)

$ProjectPath = "/mnt/d/01_Projects/solana-crowdfunding-platform"

Write-Host "Opening WSL distro '$Distro' at $ProjectPath"
wsl -d $Distro --cd $ProjectPath
