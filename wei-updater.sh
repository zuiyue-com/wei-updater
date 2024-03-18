#!/bin/bash

# 参数
arg1=$1

# 暂停 10 秒
sleep 5

# 复制文件
cp -r "./new/$arg1/"* "../"

# 改变工作目录
cd ".."

# 运行
./wei &