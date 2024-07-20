#!/bin/bash

echo "Uninstalling NVM (Node Version Manager)..."

# 删除链接
if [ -L "$NVM_SYMLINK" ]; then
    sudo rm -f "$NVM_SYMLINK"
fi

# 删除NVM_HOME目录
if [ -d "$NVM_HOME" ]; then
    sudo rm -rf "$NVM_HOME"
fi

# 判断/usr/local/bin/nvm-win-rust 是否存在，存在则删除
if [ -L "/usr/local/bin/nvm-win-rust" ]; then
    sudo rm -f "/usr/local/bin/nvm-win-rust"
fi

echo "NVM has been uninstalled successfully."