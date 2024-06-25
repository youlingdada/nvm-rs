#!/bin/bash

echo "Installing NVM (Node Version Manager)..."

# 设置NVM_HOME的默认值
NVM_HOME="$HOME/.nvm"

# 让用户输入安装目录,如果用户未输入,则使用默认值
read -p "Please enter the installation directory of NVM [$NVM_HOME]: " input
if [ -n "$input" ]; then
    NVM_HOME="$input"
fi

# 设置NVM_SYMLINK的默认值
NVM_SYMLINK="$HOME/.nvm/node"

# 让用户输入软链接目录,如果用户未输入,则使用默认值
read -p "Please enter the symlink directory of NVM [$NVM_SYMLINK]: " input
if [ -n "$input" ]; then
    NVM_SYMLINK="$input"
fi

echo "NVM will be installed in: $NVM_HOME"
echo "NVM will be symlinked to: $NVM_SYMLINK"

# 判断是否存在，先删除再创建
if [ -d "$NVM_HOME" ]; then
    sudo rm -rf "$NVM_HOME"
fi
sudo mkdir -p "$NVM_HOME"

# 安装NVM
# 1、移动target/release/nvm-win-rust到NVM_HOME目录
sudo cp ../../target/release/nvm-win-rust "$NVM_HOME"

# 创建settings文件,写入NVM_HOME(root)和NVM_SYMLINK(path)
echo "root: $NVM_HOME" | sudo tee "$NVM_HOME/settings.txt"
echo "symlink: $NVM_SYMLINK" | sudo tee -a "$NVM_HOME/settings.txt"

# 确保 nvm-win-rust 可以以管理员身份运行
# 判断/usr/local/bin/nvm-win-rust 是否存在，存在则删除
if [ ! -L "/usr/local/bin/nvm-win-rust" ]; then
    sudo ln -s "$NVM_HOME/nvm-win-rust" /usr/local/bin/nvm-win-rust
fi
echo "NVM has been installed successfully."
