# 安装指南

## 桌面应用 (GUI)

从 [GitHub Releases](https://github.com/us/null-e/releases/latest) 下载最新版本。

| 平台 | 文件 | 说明 |
|------|------|------|
| **macOS (Apple Silicon)** | `null-e_x.x.x_aarch64.dmg` | M1/M2/M3/M4 芯片 |
| **macOS (Intel)** | `null-e_x.x.x_x64.dmg` | Intel 处理器 |
| **Windows** | `null-e_x.x.x_x64-setup.exe` | 64位 Windows 10/11 |
| **Linux (deb)** | `null-e_x.x.x_amd64.deb` | Ubuntu, Debian |
| **Linux (AppImage)** | `null-e_x.x.x_amd64.AppImage` | 通用 Linux |

### macOS 安装

应用目前没有 Apple 开发者签名，macOS 会默认阻止。

1. 下载对应芯片的 `.dmg` 文件
2. 打开 DMG，将 `null-e.app` 拖到 `/Applications`
3. 打开终端运行：

```bash
xattr -rd com.apple.quarantine /Applications/null-e.app
```

4. 从"应用程序"或 Spotlight 打开 null-e

> **为什么需要这步？** macOS Gatekeeper 会阻止未签名的应用。`xattr` 命令移除系统添加的隔离标记。这是安全的——你可以在 [GitHub](https://github.com/us/null-e) 上验证源代码。

### Windows 安装

1. 下载并运行 `.exe` 安装程序
2. 如果 SmartScreen 发出警告，点击 **"更多信息" → "仍要运行"**
3. 从开始菜单启动

### Linux 安装

```bash
# Ubuntu/Debian
sudo dpkg -i null-e_x.x.x_amd64.deb

# AppImage（任何发行版）
chmod +x null-e_x.x.x_amd64.AppImage
./null-e_x.x.x_amd64.AppImage
```

## 命令行工具 (CLI)

```bash
# 通过 crates.io
cargo install null-e

# 通过 Homebrew
brew tap us/tap
brew install null-e
```

## 自动更新

桌面应用在启动时自动检查更新。当有新版本时，顶部会出现通知栏——点击 **"更新并重启"** 即可安装。

手动检查：**设置 → 关于 → 检查更新**

## 系统要求

- **macOS**: 10.15 (Catalina) 或更高版本
- **Windows**: Windows 10 或更高版本（64位）
- **Linux**: Ubuntu 20.04 或同等版本
- **磁盘空间**: 约 50 MB
