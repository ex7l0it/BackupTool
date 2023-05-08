
# Config Backup Tool

## Introduction

用于备份配置文件的工具 (Linux/MacOS)


例如：

- .bashrc
- .vimrc
- .tmux.conf

## Usage

```shell
Usage: cfgbkc [OPTIONS] <MODE>

Arguments:
  <MODE>  Mode of operation [possible values: backup, restore]

Options:
  -c, --config <CONFIG>  Path to config file [default: ./config.yaml]
  -o, --output <OUTPUT>  Path to backup file [default: ./bkup/]
  -f, --bkfile <BKFILE>  Path to backup file
  -h, --help             Print help
```

配置文件：

```yaml
- name: bash     # 配置文件组名
  path:          # 当前组中包含的配置文件路径
    - ~/.bashrc
- name: vim
  path: 
    - ~/.vimrc
    - ~/.vimrc.custom.config
    - ~/.vimrc.custom.plugins
- name: tmux
  path: 
    - ~/.tmux.conf
```

执行命令以下命令后将按照配置文件中的信息进行备份打包配置文件操作：

```shell
cfgbkc backup -c ./config.yaml -o ./bkup/
```

打包后生成的压缩包内容:

```shell
./bkup_1683447915
├── bash
│   └── .bashrc
├── config.yaml
├── tmux
│   └── .tmux.conf
└── vim
    ├── .vimrc
    ├── .vimrc.custom.config
    └── .vimrc.custom.plugins
```

执行以下命令还原归档的配置文件：

```shell
cfgbkc restore -f ./bkup/bkup_1683447915.tar.gz
```

## Process

**Backup Process Steps:**

1. 访问默认配置中的工具的配置文件路径
2. 复制到指定备份路径打包归档

**Restore Process Steps:**

1. 读取备份归档文件，解压到临时目录
2. 读取配置文件，按照指定的文件路径进行还原

## TODO

- [ ] 把临时文件夹全部挪到/tmp
- [x] ~~添加还原功能~~
- [ ] 自定义打包后的文件名
- [ ] 增加对重名目标的处理
- [ ] 增加多线程处理
- [ ] 优化错误处理
- [ ] 搞一个服务器端用来收集备份的数据
- [ ] 搞一个版本控制？（整合git）
- [ ] 搞一个GUI
- [ ] 增加对 Windows 的支持

## Log

- 2023.5.7: v0.0.1 备份打包基本功能完成
- 2023.5.7: v0.0.2 还原归档基本功能完成
