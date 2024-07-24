# NPUcore-重生之我是菜狗

## 团队介绍

西北工业大学 NPUcore-重生之我是菜狗

- 组长：郭睆
- 组员：化运涛，刘伟业
- 指导老师：张羽

## 成绩介绍

操作系统内核赛龙芯赛道**第一支**初赛满分队伍。

![](/初赛测例全过排行榜.png)

基于[NPUcore+LA](https://gitlab.eduxiji.net/202310699111039/project1466467-172876)项目，做了如下工作：

- 移植到龙芯2K1000的Qemu模拟器及开发板上
- 满分通过初赛测例
- 实现龙芯2K1000开发板上的SATA驱动
- 增加网络模块，支持网络测例

## [初赛文档](/初赛文档.pdf)

## 环境准备

### 下载qemu
```bash
make qemu-download
```

### 编译第二阶段测例（可选，初赛测例无需编译）

进入测例目录并创建sdcard目录（编译好的测例会放在这个目录）
```bash
cd comp-2 && mkdir sdcard
```
编译所需测例（可选：busybox、iperf、netperf）

例：
```bash
make netperf
```

## 运行内核

直接运行：
```bash
make run
```

开启日志，设置LOG选项（可选：trace、debug、info、warn、error）
示例：
```bash
make run LOG=info
```
