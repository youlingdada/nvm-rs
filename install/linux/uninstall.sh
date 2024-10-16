#!/bin/bash

echo "Uninstalling NVM (Node Version Manager)..."

# 通过环境变量判断是否安装NVM
if [ -z "$NVM_HOME" ]; then
    echo "NVM is not installed."
    exit 1
fi

# 删除/usr/local/bin/nvm 软链接
if [ -L "/usr/local/bin/nvm" ]; then
    sudo rm -f /usr/local/bin/nvm
fi

# 删除NVM_SYMLINK软链接
if [ -L "$NVM_SYMLINK" ]; then
    sudo rm -f "$NVM_SYMLINK"
fi

# 删除NVM_HOME目录
if [ -d "$NVM_HOME" ]; then
    sudo rm -rf "$NVM_HOME"
fi

echo "NVM has been uninstalled successfully."
echo "Please manually remove the environment variables NVM_HOME and NVM_SYMLINK.!!!"