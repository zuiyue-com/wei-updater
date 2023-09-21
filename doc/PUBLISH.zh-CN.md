https://bbs.aardio.com/forum.php/?mod=viewthread&tid=8299

首先说重点：
------------------------------------------------------------------------------------------------------------
1、发布目录应添加为杀毒软件信任目录
aardio 工程首次发布时会自动打开 Windows Defender 信任目录设置工具，
如果不这样做，生成 EXE 可能非常慢，或者生成的 EXE 文件可能无法正常运行。

2、不要有侥幸心理
软件发布前你自己用杀毒软件扫描无任何误报 —— 这通常没用，
也不要以为改几句代码，或者换个编程语言没有误报就安全了 —— 发布出去过几天可能就出现误报了。

3、误报不难解决
只要软件确实没病毒，误报并不难解决，用户量越大的杀毒厂商，清除误报就越积极，提交误报的流程也越简单。

4、不是所有报毒都是误报
请先自查软件是否有不安全、潜在不安全、不受欢迎的行为，例如不要自动设置开机启动 —— 改为让用户自主选择是否开机启动。一些杀毒软件对于流行度高的软件会相对宽容（ 流行度不是单纯指用户量 ），对新的未知软件相对严格，所以新软件尤其要注意自查。

5、发布时提供压缩包而不是原文件
下载可运行的 EXE 文件通常被认为具有潜在风险，建议发布压缩包而不是原文件，基本所有电脑都有解压软件都能支持流行的压缩格式（例如 7z ）。


系统自带 Windows Defender 误报提交方法
------------------------------------------------------------------------------------------------------------

时移势易，现在除了系统自带的 Windows Defender 已经很少有人使用第三方杀毒软件。
一般软件只要解决了  Windows Defender 的误报就可以了。

Windows Defender 对于新的未知 EXE 容易产生误报，对于流行度高已建立信誉的软件相对宽容，无论你使用什么编程语言—— 都没有特权。请随手网上搜一下，你会找到更多其他编程语言编写软件的误报反馈，例如我今天就看到著名的开源界面组件 Sciter 有人反馈下载这个开源项目时也会误报 https://github.com/c-smile/sciter-js-sdk/issues/271 。实际上 GitHub 上搜索各种编程语言实现的开源软件误报反馈非常多，所以请不要道听途说就真以为换个编程语言就能解决问题。

微软Windows Defender 误报提交
https://www.microsoft.com/en-us/wdsi/filesubmission
1、点击「Software developer」图标，然后点「Continue」按钮，然后用微软账号登录，也可以用github账号邮箱登录。
2、打开「Submit a file for malware analysis」网页以后，在「Select the Microsoft security product used to scan the file *」下拉框选择产品，这里一般选「Microsoft Defender Antivirus (Windows 10)」就行了。
3、在「Select the file *」项点「Select」上传软件。
4、「Should this file be removed from our database at a certain date?」这里可以选「Yes」,日期默认是5年后。
5、「What do you believe this file is?」这里选「Incorrectly detected as malware/malicious」
6、「Detection name *」这里写病毒名称，如果本机没误报，换台机器一般就有了，这个可以在发布软件之前提交，并不需要等到发布以后，实在不确定可以随便写一个病毒名，例如 Trojan: Win32/Wacatac.D!ml
7、Additional information * 这里随便写几句，也可以附上软件官网网址。例如
There is a false positive for my software. Please re-check it and remove it from your virus list.Thanks very much!

现在 defender 基本不认识的EXE就会误报，而且系统自带，市场份额较大，建议大家重视，在软件发布前就处理下。

查看提交历史： https://www.microsoft.com/en-us/wdsi/submissionhistory

在清除误报以前 —— 请始终在设置为信任开发目录的工程目录下操作该文件（不要对外发布）。
Windows Defender 检测通过并清除误报以后，注意看检测报告中显示的病毒库版本，以及误报是由本地病毒库还是云查杀导致。如果在客户端测试下载无误报（ 1、要通过网站下载软件做测试 2、最好不要在开发该软件的电脑上测试 ）就可以对外发布。

在 aardio 里运行以下代码更新 Windows Defender 病毒库:
//RUNAS//
import console;
import process.mpCmdRun;
process.mpCmdRun.updateDefinitions();
console.pause(true);

检测通过后 —— 应当立即对外公布下载链接，正常增加流行度。
如果再次误报，这时候可以到上面 submissionhistory 里面去点击一下 Rescan submission 重新检测一次试试。如果在官网检测显示无误报，但客户端在更新到相同或更新的病毒库后仍然出现误报，请在 aardio 中运行下面的代码收集 Windows Defender 日志：
//RUNAS//
import console;
import process.mpCmdRun;
import win.clip.file;

console.showLoading(" 正在获取 Windows Defender 日志")
var logPath = process.mpCmdRun.getDiagnosticFile()
if(logPath){
    //为尽量仅收集与问题有关的数据，建议在干净的电脑或虚拟机上执行此操作。
    win.clip.file.write(logPath)
    console.log("已复制 Windows Defender 日志文件到剪贴板")
}

console.pause(true);

请将收集到的日志文件与误报文件一起按前述的方法重新提交到 https://www.microsoft.com/en-us/wdsi/filesubmission 请求重新检测。

再强调一下重点：
1、如果某几个版本一直误报不要着急，改一些代码重新提交，只要软件没问题一般都能过。
2、提交检测的是什么压缩包，对外发布下载的也应该是同一压缩包，一个字节也不要改动，过检测前不要对外发布，过检测后要立即对外发布增加流行度。


安全、杀毒软件厂商误报提交方法大全
------------------------------------------------------------------------------------------------------------
如果生成EXE失败、或生成的EXE文件不正常，文件莫名其妙消失，不能正常运行等等，请首先检查任何可能干扰EXE生成的因素：

1、自己的代码中有没有敏感的，可能被误判为威胁的操作。
2、杀毒安全类软件，及这些软件创建的后台服务。
3、检查某些软件“安全模块”。
4、不要在 U盘 上编写发布软件，U盘被误杀或干扰的机率更高
5、不要在虚拟加密分区发布软件，这些软件可能会影响 EXE 文件生成（可在工程中将生成EXE的发布目录设为普通硬盘分区）
6、首次发布工程时，请使用 aardio 自带信任目录设置工具，将发布目录添加为 Windows Defender 信任目录，不然生成 EXE 会非常慢。
7、.........其他任何可能干扰文件读写的监控软件。

一、生成EXE文件中被安全软件干扰，导致无法生成EXE文件或生成的EXE文件不正常
首先，如果在开发环境中运行正常、发布程序后如果运行 EXE 却出现错误，很有可能是安全软件在监控导致生成 EXE 过程中出现错乱，这时候请暂时关闭杀毒或安全类软件，并退出 aardio 开发环境然后重新打开再试一次就行了。

二、生成EXE成功，但是EXE文件被误报，或双击无法运行（没有任何提示）
目前很多基于服务端白名单进行云查杀的安全软件（或某些非安全软件的在后台运行的安全模块）会误报未知EXE、或阻止您的软件启动(可能无任何提示)，也有可能您生成EXE时一切正常，但发布给别人使用一段时间后被误报或拦截。解决这个问题并不难，只要你的软件没有恶意代码纯属于误报的情况：仅仅需要简单的提交你的被误报的软件给相关杀毒厂商基本都会比较快的过白、清除误报。

据用户反馈，QQ 自带的安全模块 QQProtect.exe 进程可能阻止未被QQ电脑管家过白的EXE文件启动（可能无任何提示，这是很多年前的事了，现在好像很少听到这类问题了 ）。

三、软件在浏览器中下载时被提示为恶意软件
这通常没关系，过一段时间就会自己好了，很多知名软件都有类似问题。如果你是一个全新的软件发布到互联网上，浏览器在下载该软件时可能会拦截或报警，这没有关系，这是基于社区信任评估机制，一个新的下载地址、新的文件被下载出于安全的顾虑会无条件的阻止或警报（即使软件加了数字签名、没有过界行为等等都可能被无条件警报 ）。一般放几天以后就不会再提示了。我们也建议大家尽可能以压缩包格式发布软件（ 直接提供EXE文件下载容易被拦截警告 ），强烈建议大家不要直接提供EXE格式的文件下载，改成7z等压缩包格式。

四、为软件添加签名以避免误报或被拦截。
软件生成以后，有需要的推荐到权威可信的证书服务商购买软件签名证书对软件进行签名，基本可以避免安全杀毒软件误报，在软件的行为上也会增加一些容忍度。注意： 请勿相信网络上他人免费提供给你的来历不明的不用花钱的签名工具（这种证书毫无价值，更不可轻信他人将来历不明的根证书添加为系统信任根证书 )。

五、安全杀毒厂商提交软件过白名单如果是360、金山、百度杀毒等软件，可以到官网的安全认证平台申请账号（ 都允许企业、或个人工作室注册）以后直接提交过白就不会再被相应的安全软件误报：
360安全认证平台:
http://open.soft.360.cn/

金山毒霸安全认证平台:
http://rz.ijinshan.com/

其他误报一般可以写邮件或到网站上去提交。
下面是英文邮件范例( 下面的VBA32换成杀毒软件产品名称即可 ):
邮件主题：
False Positive Submission

邮件内容：
Dear VBA32:
There is a false positive for my software.
The sample is in a password protected zip file，The password for the attachment is infected
Please re-check it and remove it from your virus list.
Thanks very much!

邮件附件：
提交zip压缩文件，加密码"infected"

注意杀毒软件提交的误报样本，一般会要求打包为zip文件，并设置密码为 'infected' 或 'virus' （ 具体参考误报提交页面的说明，不设密码可能导致你的提交的样本被自动误删）



360安全卫士误报提交:
http://open.soft.360.cn/report.htm

火绒误报提交:
误报提交邮箱：seclab@huorong.cn

VBA32
feedback@anti-virus.by
邮件主题：
False Positive Submission

邮件内容：
Dear VBA32:
There is a false positive for my software.
The sample is in a password protected zip file，The password for the attachment is infected
Please re-check it and remove it from your virus list.
Thanks very much!

邮件附件：
提交zip压缩文件，加密码"infected"

360杀毒误报提交:
http://sampleup.sd.360.cn/index.php

东方微点：
http://service.micropoint.com.cn/mail.php   

江民 误报反馈邮箱：
virus@jiangmin.com
到江民官网论坛提交

赛门铁克误报提交
https://symsubmit.symantec.com/false_positive

卡巴斯基
误报提交网址：https://support.kaspersky.com/1870
newvirus@kaspersky.com
附件把被报毒的文件用zip压缩一下加进去，ZIP设置一个密码virus，避免被邮件服务器拦截

Avira(小红伞)
http://analysis.avira.com/samples/ 选”可疑的误报”
可到论坛申请深层分析 http://forum.avira.com/wbb/index.php?page=Board&boardID=140

NOD32
安装NOD32试用版，打开NOD32点工具，然后可以提交误报文件。
samples@eset.com
压缩zip发送

NANO-Antivirus
误报提交网址：https://www.nanoav.pro/index.php ... =15&Itemid=&lang=en
theme 下拉框选 False detection
这个误报提交点「Send」以后页面可能长时间没有任何反应，要耐心的等等等，最后会提示提交成功。

Cyren
zip打包，密码为 infected
发送到邮箱 support@cyren.com

BitDefender 方法一： 到这里提交 https://www.bitdefender.com/submit/
方法二：BitDefender病毒样本上报邮箱： sample@bitdefender-cn.com
附件一定记得把被报毒的文件用zip压缩一下加进去，ZIP文件可以加密为 infected

赛门铁克（诺顿）
https://submit.symantec.com/security_risks/dispute/
https://submit.symantec.com/false_positive/standard/

Dr.Web
误报文件提交网址：https://vms.drweb.cn/sendvirus/
选择上传文件后，Submission category下拉选框中选择：False detection
Comments中输入：Dear Dr.Web: There is a false positive for my software. The file is in attachment. Please re-check it and remove it from your virus list.
Thanks very much!

AVG
误报文件提交网址：https://www.avg.com/en-us/false-positive-file-form
virus@avg.com
zip或rar压缩发送

AVAST
误报提交网址： https://www.avast.com/false-positive-file-form.php
也可以试试提交到邮箱：virus@avast.com

麦咖啡

在线提交 zip 或 rar 压缩包： https://www.mcafee.com/en-us/con ... n-allowlisting.html


sophos
https://support.sophos.com/support/s/filesubmission
提交 zip 或 7z 压缩文件

F-PROT
http://www.f-prot.com/virusinfo/false_positive_form.html
提交zip 如设置加密 请在Zip archive password写上压缩包密码

F-Secure
http://www.f-secure.com/samples/index.html
需要注册，注册成功后直接在Submit a new sample那儿点submit
提交zip压缩文件

Panda
误报问题可以通过提交zip压缩文件并发信到Virus@pandasecurity.com来解决

Prevx
误报问题可以通过提交（需要加密：infected）rar压缩文件并发信report@prevxresearch.com来解决 （zip文件将会被拒绝）

emsisoft
误报问题可以通过提交（需要加密，密码在信件正文中提示出来）zip压缩文件并发信fp@emsisoft.com来解决

Comodo
误报问题可以通过直接到网站页面提交zip压缩文件（推荐） http://internetsecurity.comodo.com/submit.php

ikarus
误报问题可以通过提交zip压缩文件并发信到 false-positive@ikarus.at来解决

Sunbelt
误报问题可以通过直接到网站页面提交zip压缩文件http://www.sunbeltsecurity.com/falsepositive/

esafe
误报问题可以通过提交zip压缩文件并发信到esafe.virus@eAladdin.com 来解决
或填表，联系客服，但要填的内容相当多：
http://www.aladdin.com/forms/send-email/form.aspx

AhnLab-V3
误报问题可以通过提交zip压缩文件并发信到v3sos@ahnlab.com来解决

nProtect
误报问题可以通过提交（需要加密：infected）zip压缩文件并发信isarc@inca.co.kr来解决

GData
在线提交误报地址：https://su.gdatasoftware.com/us/sample-submission/
误报样本可上报给：china@gdatasoftware.com 压缩密码：virus
也可通过邮箱上报给Bitdefender或者Avastvirus@avast.com 压缩密码：virus
sample@bitdefender-cn.com 压缩密码：infecte

quickheal
误报提交( 要开代理才能打开 )：
https://techsupport.quickheal.com/report-an-issue/false-positive

Immunet Protect，ClamAV ，Moon Secure, Untangle误报提交:
https://www.immunet.com/false_positive 这个网站我试了下，不用国外代理打不开。

Trend Micro误报提交:
https://www.trendmicro.com/en_us/about/legal/detection-reevaluation.html

ALYac 误报提交:
https://en.estsecurity.com/support/report
下载一个软件提交,不开代理可能会提交失败.

Ad-Aware 误报提交:
https://www.adaware.com/report-false-positives

Bkav 误报提交:
发邮件到 bkav@bkav.com.vn 附件打包为7z并加密码infected (邮件里要说明密码)

eScan 误报提交:
http://support.mwti.net/support/index.php?/Tickets/Submit/RenderForm

AegisLab  误报提交:
发邮件到 support@aegislab.com

Malwarebytes:
https://support.malwarebytes.com/hc/en-us/requests/new
不要点推荐答案，提交以后弹出页面上点 「No, I need Help」

Fortinet
https://www.fortiguard.com/faq/onlinescanner
选择上传文件，输入邮箱后弹出下拉选项，点选：This is a clean file and advise not to be detected (False Positive)
然后点「Submit」提交。



