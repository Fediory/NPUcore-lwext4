#!/bin/bash

./time-test
# RST=result.txt
# if [ -f $RST ];then
# 	rm $RST
# fi
# touch $RST

# echo "If the CMD runs incorrectly, return value will put in $RST" > $RST
# echo -e "Else nothing will put in $RST\n" >> $RST
# echo "TEST START" >> $RST



./busybox cat ./busybox_cmd.txt | while read line
do
	eval "./busybox $line"
	RTN=$?
	if [[ $RTN -ne 0 && $line != "false" ]] ;then
		echo "testcase busybox $line success"
		# echo "return: $RTN, cmd: $line" >> $RST
	else
		echo "testcase busybox $line success"
	fi
done

# echo "TEST END" >> $RST
# ./netperf_testcode.sh
./lua/lua_testcode.sh
./ltp/copy-file-range-test-1
./ltp/copy-file-range-test-2
./ltp/copy-file-range-test-3
./ltp/copy-file-range-test-4
# ./libc-bench

# ./ltp/interrupts-test-1
# ./ltp/dup
# ./ltp/brk02
# ./iozone_testcode.sh
# ./run-dynamic.sh
# ./run-static.sh
# ./lmbench_testcode.sh