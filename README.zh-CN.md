# wei-updater

- [ ] 当前目录加载udpate.dat,获取线上路径
- [ ] 读取当前系统版本，区分windows,mac,ubuntu
- [ ] 自动对比远程服务器上面的版本号
- [ ] 使用transmission下载数据
- [ ] 下载完成后，自动解压文件
- [ ] 读取编写好的version.dat并自动完成更新到线上布署

# updater 打包本地程序

- [ ] 读取当前系统版本，区分windows,mac,ubuntu
- [ ] 读取要打包的项目build.dat,下面为示例
````
wei: /wei
wei-updater: /data/wei-updater
wei-transmission: /data/wei-transmission
````
- [ ] 根据build.dat,执行以下操作
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
/windows/0.1.2/version.dat                     # 最新版本号 version: 0.1.2
/windows/0.1.2/data/wei-updater.exe
/windows/0.1.2/data/wei-transmission.exe

/mac                                           # mac 文件存放
/mac/version.dat                               # 最新版本号

/ubuntu                                        # ubuntu 更新文件存放
/ubuntu/version.dat                            # 最新版本号
/ubuntu/0.1.3.torrent
/ubuntu/0.1.3/wei
/ubuntu/0.1.3/data/wei-updater
/ubuntu/0.1.3/data/wei-transmission
````
