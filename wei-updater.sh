#!/bin/bash

# 参数
arg1=$1

killall wei

# 暂停 10 秒
sleep 5

# 复制文件
cp -r "$2/data/new/$1/"* "$2/"

# 运行
systemctl restart wei
