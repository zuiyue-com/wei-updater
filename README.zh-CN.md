# wei-updater

- [x] 当前目录加载./version.dat,对比线上版本
- [x] 读取当前系统版本，区分windows,mac,ubuntu
- [x] 使用qbittorrent下载数据
- [x] 处理qbittorrent状态
- [x] 下载完成后，写入 .wei/status.dat 0 重启所有daemon
- [ ] 清理旧版本的种子文件

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
