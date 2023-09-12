# wei-updater

# 更新版本指南

- [ ] 打包完的项目要自动提交给微软：https://www.microsoft.com/en-us/wdsi/filesubmission
- 如果有新增daemon非wei-相关的项目，需要在kill.dat里面添加，格式为yml，key为项目名称，value为进程名称
- 需要修改下面几个文件，程序会根据以下文件进行自动打包到../wei-release/os/version/目录
- 修改git设置：git config --global core.autocrlf false
- version.dat：版本号，对应 wei 项目的版本号
- build.dat：需要打包的项目，使用yml格式，key值为项目名称，value为打包路径
```
wei: /wei
wei-ui: /data/wei-ui
wei-tray: /data/wei-tray
wei-updater: /data/wei-updater
wei-task: /data/wei-task
wei-qbittorrent: /data/wei-qbittorrent
wei-sd: /data/wei-sd
```
- daemon.dat：需要自动守护的进程名字，使用yml格式，key值为项目名称，value对应key值即可
```
wei-ui: 1
wei-tray: 1
wei-updater: 1
wei-task: 1
wei-qbittorrent: 1
```

# 功能开发

- [x] 当前目录加载./version.dat,对比线上版本
- [x] 读取当前系统版本，区分windows,mac,ubuntu
- [x] 使用qbittorrent下载数据
- [x] 处理qbittorrent状态
- [x] 下载完成后，写入 .wei/status.dat 0 重启所有daemon
- [ ] 清理旧版本的种子文件
- [ ] 混合下载，使用qbittorrent和http同时下载更新文件

# updater 打包本地程序

- [x] 读取当前系统版本，区分windows,mac,ubuntu
- [x] 读取要打包的项目build.dat,下面为示例
````
wei: /wei
wei-updater: /data/wei-updater
wei-transmission: /data/wei-transmission
````
- [x] 根据build.dat,执行以下操作
````
#create version.dat, write version: 0.1.2
mv version.dat ~/work/wei-dist/linux/0.1.2/version.dat

cd ~/work/wei-dist
git pull

cd ~/work/wei
git pull
docker run -it --rm -v ~/work/wei:/data -w zuiyue-com/rust:ubuntu cargo build --release
cp -rfp ~/work/wei/target/release/wei ~/work/wei-dist/linux/0.1.2/wei

cd ~/work/wei-updater
git pull
docker run -it --rm -v ~/work/wei-updater:/data -w zuiyue-com/rust:ubuntu cargo build --release
cp -rfp ~/work/wei-updater/target/release/wei-updater ~/work/wei-dist/linux/0.1.2/data/wei-updater

# 读取 https://cf.trackerslist.com/best.txt 使用 -t http://tracker.skyts.net:6969/announce -t http://tracker.tfile.co:80/announce -t http://v6-tracker.0g.cx:6969/announce -t http://www.all4nothin.net:80/announce.php 加入多个 tracker

transmission-create -o ~/work/wei-dist/0.1.2.torrent -t http://tracker.skyts.net:6969/announce -t http://tracker.tfile.co:80/announce -t http://v6-tracker.0g.cx:6969/announce -t http://www.all4nothin.net:80/announce.php -s 2048 ~/work/wei-dist/linux/0.1.2 -c wei_0.1.2
````

# 示例

- update.dat

````
url: http://updater.zuiyue.com/
````

- 服务器目录结构

````
http://updater.zuiyue.com/                     # 服务器根目录

/windows/                                      # windows 文件存放
/windows/version.dat                           # 最新版本号 version: 0.1.2
/windows/0.1.2.torrent
/windows/0.1.2/wei.exe
/windows/0.1.2/data/version.dat                # 最新版本号 version: 0.1.2
/windows/0.1.2/data/wei-updater.exe
/windows/0.1.2/data/wei-transmission.exe

/mac                                           # mac 文件存放
/mac/version.dat                               # 最新版本号

/ubuntu                                        # ubuntu 更新文件存放
/ubuntu/version.dat                            # 最新版本号
/ubuntu/0.1.3.torrent
/ubuntu/0.1.3/wei
/ubuntu/0.1.3/data/version.dat
/ubuntu/0.1.3/data/wei-updater
/ubuntu/0.1.3/data/wei-transmission
````
