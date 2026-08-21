#Requires -Version 5.1
# ═══════════════════════════════════════════════════════════════════════
# Angles Code CLI — Installer (PowerShell)
# Usage: irm https://raw.githubusercontent.com/ZSJ305/angles-cli/main/install.ps1 | iex
# ═══════════════════════════════════════════════════════════════════════

[CmdletBinding()]
param(
    [switch]$Help,
    [switch]$DryRun,
    [switch]$NoPrompt,
    [switch]$NoGateway,
    [string]$RepoUrl = "https://github.com/ZSJ305/angles-cli",
    [string]$InstallDir = "",
    [string]$AnglesHome = ""
)

$ErrorActionPreference = "Stop"

# ── Colors (ANSI) ──
$ESC = [char]27
$BOLD        = "$ESC[1m"
$ACCENT      = "$ESC[38;2;90;200;255m"
$ACCENT_BRIGHT = "$ESC[38;2;130;220;255m"
$INFO        = "$ESC[38;2;136;146;176m"
$SUCCESS     = "$ESC[38;2;0;229;204m"
$WARN        = "$ESC[38;2;255;176;32m"
$ERROR_C     = "$ESC[38;2;230;57;70m"
$MUTED       = "$ESC[38;2;90;100;128m"
$NC          = "$ESC[0m"

# Enable VT100 color support
try { [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 } catch {}

# ── Configuration ──
$script:CONFIG = @{
    RepoUrl     = if ($RepoUrl) { $RepoUrl } else { $env:ANGLES_REPO -or "https://github.com/ZSJ305/angles-cli" }
    InstallDir  = if ($InstallDir) { $InstallDir } else { $env:ANGLES_INSTALL_DIR -or (Join-Path $env:USERPROFILE ".local\bin") }
    AnglesHome  = if ($AnglesHome) { $AnglesHome } else { $env:ANGLES_HOME -or (Join-Path $env:USERPROFILE ".angles") }
    NoPrompt    = $NoPrompt -or ($env:NO_PROMPT -eq "1")
    NoGateway   = $NoGateway -or ($env:NO_GATEWAY -eq "1")
    DryRun      = $DryRun -or ($env:DRY_RUN -eq "1")
    Help        = $Help -or ($env:HELP -eq "1")
    RustMinVer  = "1.75.0"
}

# ── Temp file management ──
$script:TMP_FILES = @()

function Add-TempItem {
    param([string]$Path)
    $script:TMP_FILES += $Path
}

function Clear-TempItems {
    foreach ($item in $script:TMP_FILES) {
        try {
            if (Test-Path $item) {
                Remove-Item -Recurse -Force $item -ErrorAction SilentlyContinue
            }
        } catch {
            Write-Warning "清理临时文件失败: $item"
        }
    }
}

function New-TempFile {
    $file = [System.IO.Path]::GetTempFileName()
    Add-TempItem $file
    return $file
}

function New-TempDirectory {
    $dir = Join-Path $env:TEMP ("angles-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
    Add-TempItem $dir
    return $dir
}

# ── UI Helpers ──
function Write-Info   { param([string]$Msg) Write-Host "  $INFO$Msg$NC" }
function Write-Success{ param([string]$Msg) Write-Host "  $SUCCESS[OK] $Msg$NC" }
function Write-Warn   { param([string]$Msg) Write-Host "  $WARN[!] $Msg$NC" }
function Write-Error  { param([string]$Msg) Write-Host "  $ERROR_C[X] $Msg$NC" }
function Write-Stage  { param([string]$Msg) Write-Host "`n  $ACCENT$BOLD[$Msg]$NC`n" }
function Write-KV     { param([string]$Key, [string]$Value) Write-Host ("  $MUTED{0,-14}$NC {1}" -f $Key, $Value) }
function Write-Section{ param([string]$Msg) Write-Host "`n  $ACCENT$BOLD$Msg$NC`n" }

function Test-Interactive {
    return ($script:CONFIG.NoPrompt -eq $false) -and [Environment]::UserInteractive
}

# ── System Detection ──
function Initialize-SystemInfo {
    $script:OS = "windows"
    
    $rawArch = $env:PROCESSOR_ARCHITECTURE
    if (-not $rawArch) {
        $rawArch = [Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE")
    }
    
    switch ($rawArch) {
        "AMD64" { $script:ARCH = "x64";   $script:ARCH_LABEL = "x86_64" }
        "ARM64" { $script:ARCH = "arm64"; $script:ARCH_LABEL = "aarch64" }
        "x86"   { $script:ARCH = "x64";   $script:ARCH_LABEL = "x86_64" }
        default {
            throw "不支持的处理器架构: $rawArch"
        }
    }
    
    Write-KV "操作系统" "Windows"
    Write-KV "架构" "$($script:ARCH_LABEL) ($($script:ARCH))"
}

# ── Network Helpers ──
function Invoke-Download {
    param([string]$Url, [string]$OutputPath)
    
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Write-Info "正在下载: $(Split-Path $Url -Leaf)"
        Invoke-WebRequest -Uri $Url -OutFile $OutputPath -UseBasicParsing -TimeoutSec 120
        return $true
    }
    catch {
        Write-Warn "下载失败: $Url"
        return $false
    }
}

function Test-UrlExists {
    param([string]$Url)
    
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $response = Invoke-WebRequest -Uri $Url -Method Head -UseBasicParsing -ErrorAction Stop
        return $response.StatusCode -eq 200
    }
    catch {
        return $false
    }
}

# ── Winget Helper ──
function Test-WingetAvailable {
    return [bool](Get-Command winget -ErrorAction SilentlyContinue)
}

function Install-WithWinget {
    param([string]$PackageId, [string]$DisplayName)
    
    if (-not (Test-WingetAvailable)) {
        Write-Warn "未找到 winget，请手动安装 $DisplayName"
        return $false
    }
    
    try {
        Write-Info "通过 winget 安装 $DisplayName..."
        $process = Start-Process -FilePath "winget" -ArgumentList @(
            "install",
            "--id", $PackageId,
            "-e",
            "--accept-source-agreements",
            "--accept-package-agreements",
            "--silent"
        ) -Wait -PassThru -NoNewWindow
        
        if ($process.ExitCode -eq 0) {
            # 刷新 PATH
            $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + 
                       [Environment]::GetEnvironmentVariable("PATH", "User")
            return $true
        }
        else {
            Write-Warn "$DisplayName 安装可能失败 (退出码: $($process.ExitCode))"
            return $false
        }
    }
    catch {
        Write-Warn "$DisplayName 安装异常: $_"
        return $false
    }
}

# ── Tool Installation Functions ──
function Install-BuildTools {
    Write-Info "检查编译工具..."
    
    # 检查 Git
    $gitInstalled = [bool](Get-Command git -ErrorAction SilentlyContinue)
    if (-not $gitInstalled) {
        Install-WithWinget -PackageId "Git.Git" -DisplayName "Git"
    }
    
    # 检查 Visual Studio Build Tools
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    $vsInstalled = $false
    
    if (Test-Path $vswhere) {
        $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($vsPath) { $vsInstalled = $true }
    }
    
    if (-not $vsInstalled) {
        Write-Warn "未检测到 Visual Studio Build Tools"
        if (Test-WingetAvailable) {
            Write-Info "尝试安装 Visual Studio Build Tools 2022 (C++ 工作负载)..."
            try {
                $process = Start-Process -FilePath "winget" -ArgumentList @(
                    "install",
                    "--id", "Microsoft.VisualStudio.2022.BuildTools",
                    "-e",
                    "--override", "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended",
                    "--accept-source-agreements",
                    "--accept-package-agreements"
                ) -Wait -PassThru -NoNewWindow
                
                if ($process.ExitCode -ne 0) {
                    Write-Warn "Build Tools 安装可能需要手动完成"
                }
            }
            catch {
                Write-Warn "Build Tools 安装失败: $_"
            }
        }
        else {
            Write-Warn "请手动安装 Visual Studio Build Tools (C++)"
        }
    }
}

function Install-Rust {
    if (Test-RustInstalled) {
        $version = (cargo --version 2>$null) -replace '^cargo ',''
        Write-Success "Rust $version"
        return
    }
    
    Write-Info "安装 Rust..."
    
    $tempExe = New-TempFile
    $exePath = [System.IO.Path]::ChangeExtension($tempExe, ".exe")
    
    $rustUrl = if ($script:ARCH -eq "arm64") {
        "https://win.rustup.rs/aarch64"
    } else {
        "https://win.rustup.rs/x86_64"
    }
    
    if (-not (Invoke-Download -Url $rustUrl -OutputPath $exePath)) {
        throw "Rust 安装程序下载失败"
    }
    
    $process = Start-Process -FilePath $exePath -ArgumentList "-y", "--default-toolchain", "stable" -Wait -PassThru -NoNewWindow
    
    if ($process.ExitCode -ne 0) {
        throw "Rust 安装失败 (退出码: $($process.ExitCode))"
    }
    
    # 加载 Rust 环境
    $cargoEnv = Join-Path $env:USERPROFILE ".cargo\env.ps1"
    if (Test-Path $cargoEnv) {
        . $cargoEnv
    }
    
    # 刷新 PATH
    $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" +
               [Environment]::GetEnvironmentVariable("PATH", "User")
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "Rust 安装后无法找到 cargo"
    }
    
    $version = (cargo --version 2>$null) -replace '^cargo ',''
    Write-Success "Rust $version"
}

function Test-RustInstalled {
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargo) { return $false }
    
    $versionLine = cargo --version 2>$null
    if (-not $versionLine) { return $false }
    
    $match = [regex]::Match($versionLine, '(\d+)\.(\d+)\.(\d+)')
    if (-not $match.Success) { return $false }
    
    $major = [int]$match.Groups[1].Value
    $minor = [int]$match.Groups[2].Value
    
    if ($major -lt 1 -or ($major -eq 1 -and $minor -lt 75)) {
        Write-Warn "Rust 版本过低 ($($match.Value) < $($script:CONFIG.RustMinVer))，需要升级"
        return $false
    }
    
    return $true
}

# ── Angles Installation ──
function Test-AnglesInstalled {
    $binary = Join-Path $script:CONFIG.InstallDir "angles.exe"
    if (Test-Path $binary) {
        $version = & $binary --version 2>$null | Select-Object -First 1
        if ($version) {
            Write-Info "发现已有安装: $version"
            return $true
        }
    }
    return $false
}

function Install-AnglesBinary {
    param([string]$Os, [string]$Arch)
    
    $binaryName = "angles-$Os-$Arch"
    $downloadUrl = "$($script:CONFIG.RepoUrl)/releases/latest/download/$binaryName.zip"
    
    Write-Info "检查预编译二进制: $binaryName"
    
    if (-not (Test-UrlExists $downloadUrl)) {
        Write-Info "预编译二进制不可用"
        return $false
    }
    
    # 下载
    $tempZip = New-TempFile
    if (-not (Invoke-Download -Url $downloadUrl -OutputPath $tempZip)) {
        return $false
    }
    
    # 解压
    $tempDir = New-TempDirectory
    try {
        Expand-Archive -Path $tempZip -DestinationPath $tempDir -Force
    }
    catch {
        Write-Warn "解压失败: $_"
        return $false
    }
    
    # 查找 exe 文件
    $exeFile = Get-ChildItem $tempDir -Recurse -Filter "*.exe" | Select-Object -First 1
    if (-not $exeFile) {
        Write-Warn "未找到可执行文件"
        return $false
    }
    
    # 创建安装目录
    if (-not (Test-Path $script:CONFIG.InstallDir)) {
        New-Item -ItemType Directory -Path $script:CONFIG.InstallDir -Force | Out-Null
    }
    
    # 复制文件
    $targetPath = Join-Path $script:CONFIG.InstallDir "angles.exe"
    Copy-Item $exeFile.FullName $targetPath -Force
    
    Write-Success "预编译二进制安装完成"
    return $true
}

function Install-AnglesFromSource {
    Write-Info "从源码编译..."
    
    $tempDir = New-TempDirectory
    $sourceDir = Join-Path $tempDir "angles-cli"
    
    # 克隆仓库
    Write-Info "克隆仓库..."
    $cloneResult = & git clone --depth 1 $script:CONFIG.RepoUrl $sourceDir 2>&1
    
    if ($LASTEXITCODE -ne 0) {
        throw "仓库克隆失败: $($cloneResult -join ' ')"
    }
    
    # 编译
    Push-Location $sourceDir
    try {
        Write-Info "编译中 (cargo build --release)..."
        $buildResult = & cargo build --release 2>&1
        
        if ($LASTEXITCODE -ne 0) {
            throw "编译失败: $($buildResult -join ' ')"
        }
        
        $binary = "target\release\angles.exe"
        if (-not (Test-Path $binary)) {
            throw "编译完成后未找到 angles.exe"
        }
        
        # 创建安装目录
        if (-not (Test-Path $script:CONFIG.InstallDir)) {
            New-Item -ItemType Directory -Path $script:CONFIG.InstallDir -Force | Out-Null
        }
        
        # 复制二进制
        $targetPath = Join-Path $script:CONFIG.InstallDir "angles.exe"
        Copy-Item $binary $targetPath -Force
        
        Write-Success "编译安装完成"
    }
    finally {
        Pop-Location
    }
}

function Set-PathEnvironment {
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    $pathParts = $userPath -split ";" | Where-Object { $_ -ne "" }
    
    if ($pathParts -contains $script:CONFIG.InstallDir) {
        Write-Info "PATH 已包含安装目录"
        return
    }
    
    if ($script:CONFIG.DryRun) {
        Write-Info "将添加 $($script:CONFIG.InstallDir) 到用户 PATH"
        return
    }
    
    $newPath = ($pathParts + $script:CONFIG.InstallDir) -join ";"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    $env:PATH = "$env:PATH;$($script:CONFIG.InstallDir)"
    
    Write-Success "已将 $($script:CONFIG.InstallDir) 添加到用户 PATH"
}

function Test-AnglesInstallation {
    $binary = Join-Path $script:CONFIG.InstallDir "angles.exe"
    
    if (-not (Test-Path $binary)) {
        Write-Error "angles 二进制不存在: $binary"
        return $false
    }
    
    $version = & $binary --version 2>$null | Select-Object -First 1
    if (-not $version) {
        Write-Warn "angles --version 无输出"
        return $false
    }
    
    Write-Success "angles $version"
    return $true
}

# ── Display Functions ──
function Show-Banner {
    Write-Host ""
    Write-Host "  $ACCENT$BOLD+===========================================+$NC"
    Write-Host "  $ACCENT$BOLD|   A  Angles Code CLI Installer            |$NC"
    Write-Host "  $ACCENT$BOLD|   created by ZSJ                          |$NC"
    Write-Host "  $ACCENT$BOLD+===========================================+$NC"
    Write-Host ""
}

function Show-Usage {
    Write-Host "用法:"
    Write-Host "  irm https://raw.githubusercontent.com/ZSJ305/angles-cli/main/install.ps1 | iex"
    Write-Host "  npm i -g @angleschina/angles; angles install"
    Write-Host ""
    Write-Host "参数:"
    Write-Host "  -Help            显示此帮助信息"
    Write-Host "  -DryRun          模拟运行，不做实际更改"
    Write-Host "  -NoPrompt        跳过所有交互提示"
    Write-Host "  -NoGateway       跳过网关设置向导"
    Write-Host "  -RepoUrl <url>   指定自定义仓库地址"
    Write-Host "  -InstallDir <path> 指定安装目录"
    Write-Host "  -AnglesHome <path> 指定配置目录"
    Write-Host ""
    Write-Host "也可以通过环境变量配置:"
    Write-Host "  `$env:ANGLES_REPO, `$env:ANGLES_INSTALL_DIR, `$env:ANGLES_HOME"
    Write-Host "  `$env:NO_PROMPT, `$env:NO_GATEWAY, `$env:DRY_RUN"
    Write-Host ""
}

function Show-InstallPlan {
    Write-Host "  $ACCENT${BOLD}安装计划$NC"
    Write-KV "操作系统" "Windows"
    Write-KV "架构" "$($script:ARCH_LABEL) ($($script:ARCH))"
    Write-KV "安装目录" $script:CONFIG.InstallDir
    Write-KV "配置目录" $script:CONFIG.AnglesHome
    Write-KV "安装方式" "预编译二进制 (备选: 源码编译)"
    Write-Host ""
    Write-Host "  $MUTED也可通过 npm 安装：$NC"
    Write-Host "    $ACCENT npm i -g @angleschina/angles; angles install$NC"
    Write-Host ""
}

function Show-Footer {
    Write-Host ""
    $line = "-" * 44
    Write-Host "  $MUTED$line$NC"
    Write-Host "  $MUTED`GitHub:$NC  https://github.com/ZSJ305/angles-cli"
    Write-Host "  $MUTED npm:$NC     npm i -g @angleschina/angles"
    Write-Host "  $MUTED`文档:$NC    https://github.com/ZSJ305/angles-cli#readme"
    Write-Host "  $MUTED`反馈:$NC    https://github.com/ZSJ305/angles-cli/issues"
    Write-Host ""
}

function Get-RandomMessage {
    param([bool]$IsUpgrade)
    
    if ($IsUpgrade) {
        $messages = @(
            "升级完成！新版本已就绪。",
            "焕然一新！更锋利的 A 到手。",
            "代码已更新，bug 已退散。",
            "升级成功！还是那个味，但更快了。"
        )
    } else {
        $messages = @(
            "终于安家了！准备好大干一场了吗？",
            "安装完成！你的终端从此不一样了。",
            "A 就位！开始写代码吧。",
            "搞定！angles 是你新的编码搭档。",
            "欢迎加入 Angles！让 AI 替你干脏活。"
        )
    }
    
    return $messages[(Get-Random -Maximum $messages.Count)]
}

# ── Main Function ──
function Main {
    # 显示帮助
    if ($script:CONFIG.Help) {
        Show-Usage
        return
    }
    
    # 开始安装
    try {
        Show-Banner
        
        # 系统检测
        Write-Info "检测系统环境..."
        Initialize-SystemInfo
        
        # 检查现有安装
        $isUpgrade = Test-AnglesInstalled
        
        # 显示安装计划
        Show-InstallPlan
        
        # Dry run
        if ($script:CONFIG.DryRun) {
            Write-Success "Dry run 完成 (未做任何更改)"
            return
        }
        
        # 创建必要目录
        if (-not (Test-Path $script:CONFIG.InstallDir)) {
            New-Item -ItemType Directory -Path $script:CONFIG.InstallDir -Force | Out-Null
        }
        if (-not (Test-Path $script:CONFIG.AnglesHome)) {
            New-Item -ItemType Directory -Path $script:CONFIG.AnglesHome -Force | Out-Null
        }
        
        # 步骤 1: 检查预编译二进制
        Write-Stage "步骤 1/4: 检查预编译二进制"
        $binaryAvailable = Install-AnglesBinary -Os $script:OS -Arch $script:ARCH
        
        if (-not $binaryAvailable) {
            Write-Info "预编译不可用，准备编译环境"
            
            # 安装编译工具
            Write-Stage "步骤 1/4: 安装编译工具"
            Install-BuildTools
            
            # 安装 Rust
            Write-Stage "步骤 1/4: 安装 Rust"
            Install-Rust
        }
        
        # 步骤 2: 安装 Angles
        Write-Stage "步骤 2/4: 安装 Angles"
        if (-not $binaryAvailable) {
            Install-AnglesFromSource
        }
        
        # 步骤 3: 配置 PATH
        Write-Stage "步骤 3/4: 配置 PATH"
        Set-PathEnvironment
        
        # 步骤 4: 验证安装
        Write-Stage "步骤 4/4: 验证安装"
        if (-not (Test-AnglesInstallation)) {
            throw "安装验证失败"
        }
        
        # 安装完成消息
        Write-Host ""
        Write-Host "  $MUTED$(Get-RandomMessage -IsUpgrade $isUpgrade)$NC"
        Write-Host ""
        
        # 显示安装详情
        Write-Section "安装详情"
        $binary = Join-Path $script:CONFIG.InstallDir "angles.exe"
        $version = & $binary --version 2>$null | Select-Object -First 1
        Write-KV "版本" $version
        Write-KV "位置" $binary
        Write-KV "配置" (Join-Path $script:CONFIG.AnglesHome "config.json")
        Write-KV "升级命令" "angles update"
        Write-Host ""
        
        # 首次安装运行网关设置
        if (-not $script:CONFIG.NoGateway) {
            $configFile = Join-Path $script:CONFIG.AnglesHome "config.json"
            if (-not (Test-Path $configFile)) {
                Write-Info "启动设置向导..."
                Write-Host ""
                
                if (Test-Interactive) {
                    & $binary gateway
                }
                else {
                    Write-Warn "设置向导需要交互式终端"
                    Write-Host "  运行以下命令手动配置: angles gateway"
                }
            }
            else {
                Write-Info "已有配置文件，跳过向导"
            }
        }
        
        # 最终提示
        Write-Host ""
        Write-Host "  $SUCCESS$BOLD[OK] 安装完成!$NC"
        Write-Host ""
        Write-Host "  运行以下命令开始:"
        Write-Host ""
        Write-Host "    $ACCENT`$env:PATH += `";$($script:CONFIG.InstallDir)`"$NC   # 刷新当前会话 PATH"
        Write-Host "    ${ACCENT}angles$NC              # 开始对话"
        Write-Host ""
        
        Show-Footer
    }
    catch {
        Write-Error "安装失败: $_"
        Write-Host ""
        Write-Host "  $MUTED如需帮助，请访问: https://github.com/ZSJ305/angles-cli/issues$NC"
        exit 1
    }
    finally {
        Clear-TempItems
    }
}

# ── Entry Point ──
Main
