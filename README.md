# nvm-rs

* 采用rust编写的nvm，完全兼容nvm-windows 完全实现nvm的功能，且扩充对linux和mac的支持。扩充了命令，优化了list打印

### 安装
* rust安装请自行百度
* 拉取代码，并编译
```shell
git clone git@github.com:youlingdada/nvm-rs.git
cd nvm-rs
cargo build --release
```

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

