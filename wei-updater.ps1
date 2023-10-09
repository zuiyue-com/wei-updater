param (
    [Parameter(Mandatory=$true)]
    [string]$arg1
)

# 暂停 10 秒
Start-Sleep -s 10

# 复制文件
Copy-Item -Path ".\new\$arg1\*" -Destination "..\" -Recurse

# 改变工作目录
Set-Location "..\"

# 运行 wei.exe
Start-Process ".\wei.exe"