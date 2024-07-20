#!/bin/zsh

echo "Installing NVM (Node Version Manager)..."

$APP_NAME="nvm-win-rust"

# 设置NVM_HOME的默认值
NVM_HOME="$HOME/.nvm"

# 让用户输入安装目录，如果用户未输入，则使用默认值
read "input?Please enter the installation directory of NVM [$NVM_HOME]: "
if [ -n "$input" ]; then
    NVM_HOME="$input"
fi

# 设置NVM_SYMLINK的默认值
NVM_SYMLINK="$HOME/.nvm/node"

# 让用户输入软链接目录，如果用户未输入，则使用默认值
read "input?Please enter the symlink directory of NVM [$NVM_SYMLINK]: "
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
# 移动target/release/nvm-win-rust到NVM_HOME目录
if [ -f "./$APP_NAME" ]; then
    sudo mv "./$APP_NAME" "$NVM_HOME"
elif [ -f "../../target/release/$APP_NAME" ]; then
    sudo cp ../../target/release/$APP_NAME "$NVM_HOME"
else
    curl -L https://github.com/youlingdada/nvm-win-rust/releases/download/test-0.1/nvm-win-rust -O nvm-win-rust
    sudo mv ./$APP_NAME "$NVM_HOME"
fi
sudo chmod +x "$NVM_HOME/$APP_NAME"

# 创建settings文件，写入NVM_HOME(root)和NVM_SYMLINK(path)
echo "root: $NVM_HOME" | sudo tee "$NVM_HOME/settings.txt"
echo "symlink: $NVM_SYMLINK" | sudo tee -a "$NVM_HOME/settings.txt"

# 设置环境变量脚本
NVM_PROFILE="$NVM_HOME/nvm_profile"
if [ -f "$NVM_PROFILE" ]; then
    sudo rm -f "$NVM_PROFILE"
fi
echo "export PATH=\"$NVM_SYMLINK/bin:\$PATH\"" | sudo tee "$NVM_PROFILE"

# 写入到.zprofile中
echo "source $NVM_PROFILE" | sudo tee -a "$HOME/.zprofile"

# 配置命令脚本
NVM="$NVM_HOME/nvm"
if [ -f "$NVM" ]; then
    sudo rm -f "$NVM"
fi

echo "#!/bin/zsh" | sudo tee "$NVM"
echo "$NVM_HOME/nvm-win-rust \"\$@\"" | sudo tee -a "$NVM"
sudo chmod +x "$NVM"

# 判断/usr/local/bin/nvm 是否存在，存在则删除
if [ -L "/usr/local/bin/nvm" ]; then
    sudo rm -r /usr/local/bin/nvm
fi
sudo ln -s "$NVM" /usr/local/bin/nvm

# 卸载脚本
NVM_UNINSTALL="$NVM_HOME/nvm-uninstall"
# 判断是否存在，存在先删除
if [ -f "$NVM_UNINSTALL" ]; then
    sudo rm -f "$NVM_UNINSTALL"
fi

echo "#!/bin/zsh" | sudo tee "$NVM_UNINSTALL"
#1、 删除NVM_HOME 目录
# 判断是否存在，先删除再创建
echo "if [ -d \"$NVM_HOME\" ]; then" | sudo tee -a "$NVM_UNINSTALL"
echo "    sudo rm -rf \"$NVM_HOME\"" | sudo tee -a "$NVM_UNINSTALL"
echo "fi" | sudo tee -a "$NVM_UNINSTALL"
echo "echo delete NVM_HOME successful" | sudo tee -a "$NVM_UNINSTALL"

#2、删除对应的链接 NVM
# 判断/usr/local/bin/nvm 是否存在，存在则删除
echo "if [ -L \"/usr/local/bin/nvm\" ]; then" | sudo tee -a "$NVM_UNINSTALL"
echo "    sudo rm -f /usr/local/bin/nvm" | sudo tee -a "$NVM_UNINSTALL"
echo "fi" | sudo tee -a "$NVM_UNINSTALL"
echo "echo delete NVM LINK successful" | sudo tee -a "$NVM_UNINSTALL"

#3、删除对应的链接 NVM_UNINSTALL
# 判断/usr/local/bin/nvm 是否存在，存在则删除
echo "if [ -L \"/usr/local/bin/nvm-uninstall\" ]; then" | sudo tee -a "$NVM_UNINSTALL"
echo "    sudo rm -f /usr/local/bin/nvm-uninstall" | sudo tee -a "$NVM_UNINSTALL"
echo "fi" | sudo tee -a "$NVM_UNINSTALL"

echo "echo Please delete \"source $NVM_PROFILE\" in $HOME/.zprofile" | sudo tee -a "$NVM_UNINSTALL"
echo "echo delete NVM_UNINSTALL LINK successful" | sudo tee -a "$NVM_UNINSTALL"
sudo chmod +x "$NVM_UNINSTALL"

# 链接NVM_UNINSTALL
if [ ! -L "/usr/local/bin/nvm-uninstall" ]; then
    sudo ln -s "$NVM_UNINSTALL" /usr/local/bin/nvm-uninstall
fi

echo "Please source ~/.zprofile or restart terminal "