# nvm-rs

* 采用rust编写的nvm，完全兼容nvm-windows 完全实现nvm的功能，且扩充对linux和mac的支持。扩充了命令，优化了list打印

## 编译
### rust环境与代码编译
* [rust官网下载环境](https://www.rust-lang.org/zh-CN/tools/install),
* [字节跳动提供的镜像下载](https://rsproxy.cn/#getStarted)

* 拉取代码，并编译
```shell
git clone git@github.com:youlingdada/nvm-rs.git
cd nvm-rs
cargo build --release
```

## 直接安装
* 对于linux 和 mac 系统,如安装到/usr/local/nvm文件下
    * 移动可执行文件到对应的目录
        ```shell
        sudo mv target/release/nvm /usr/local/nvm/
        ```
    * 写入setting.txt配置
        ```shell
        echo "root: /usr/local/nvm" | sudo tee "/usr/local/nvm/settings.txt"
        echo "symlink: /usr/local/nvm/node" | sudo tee -a "/usr/local/nvm/settings.txt"
        ```
    * 设置环境变量
        ```shell
        # zsh
        echo "export NVM_HOME=/usr/local/nvm" >> ~/.zshrc
        echo "export NVM_SYMLINK=/usr/local/nvm/node" >> ~/.zshrc
        source ~/.zshrc

        # bash
        echo "export NVM_HOME=/usr/local/nvm" >> ~/.bash_profile
        echo "export NVM_SYMLINK=/usr/local/nvm/node" >> ~/.bash_profile
        source ~/.bash_profile

        # fish
        echo "set -x NVM_HOME /usr/local/nvm" >> ~/.config/fish/config.fish
        echo "set -x NVM_SYMLINK /usr/local/nvm/node" >> ~/.config/fish/config.fish
        source ~/.config/fish/config.fish
        ```
* windows


## 预写脚本安装
### windows 安装

* 安装Inno Setup Compilter，并编译install/win/install.iss
* 运行生产的安装执行文件setup/nvm-rs-setup.exe
* 可以直接打开Inno Setup Compilter编译，也可配置好Inno Setup Compilter环境变量后运行install/win/build.cmd
```cmd
install/win/build.cmd
```

### linux 安装
* 运行install/linux/install.sh脚本安装
```shell
chmod +x install/linux/install.sh
sudo sh install/linux/install.sh
```
### mac 安装
* 运行 install/mac/install.zsh
```shell
chmod +x install/mac/install.zsh
sudo zsh install/mac/install.zsh
```

