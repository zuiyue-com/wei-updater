# wei-updater

- [ ] 当前目录加载udpate.dat,获取线上路径
- [ ] 读取当前系统版本，区分windows,mac,ubuntu
- [ ] 自动对比远程服务器上面的版本号
- [ ] 使用transmission下载数据
- [ ] 下载完成后，自动解压文件
- [ ] 读取编写好的version.dat并自动完成更新到线上布署

# 示例

- update.dat

url: http://updater.zuiyue.com/

- 服务器目录结构

````
http://updater.zuiyue.com/      # 服务器根目录
/windows                        # windows 文件存放
/windows/version.dat            # 版本号对比
/windows/updater.zip            # 更新文件
/mac                            # mac 文件存放
/mac/version.dat                # 版本号对比
/mac/updater.zip                # 更新文件
/ubuntu                         # ubuntu 更新文件存放
/ubuntu/version.dat             # 版本号对比
/ubuntu/updater.zip             # 更新文件
````
