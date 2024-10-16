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
# 1、移动nvm-linux到NVM_HOME目录
sudo cp ./nvm "$NVM_HOME"/nvm

# 创建settings文件,写入NVM_HOME(root)和NVM_SYMLINK(path)
echo "root: $NVM_HOME" | sudo tee "$NVM_HOME/settings.txt"
echo "symlink: $NVM_SYMLINK" | sudo tee -a "$NVM_HOME/settings.txt"

# 确保 nvm 可以以管理员身份运行
# 判断/usr/local/bin/nvm 是否存在，存在则删除
if [ ! -L "/usr/local/bin/nvm" ]; then
    sudo ln -s "$NVM_HOME/nvm" /usr/local/bin/nvm
fi

# 给予卸载脚本可执行权限，并移动到NVM_HOME目录
sudo cp ./uninstall.sh "$NVM_HOME"/nvm-uninstall.sh
sudo chmod +x "$NVM_HOME"/nvm-uninstall.sh

# 导出环境变量到用户目录
echo "export NVM_HOME=$NVM_HOME" >> "$HOME/.bashrc"
echo "export NVM_SYMLINK=$NVM_SYMLINK" >> "$HOME/.bashrc"
echo "export PATH=\$PATH:\$NVM_HOME" >> "$HOME/.bashrc"
source "$HOME/.bashrc"

echo "NVM has been installed successfully."
