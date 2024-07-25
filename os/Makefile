# run: 清除编译结果，重新编译，运行
# all: 直接编译，并把.bin内核拷贝到根目录（适配大赛要求）
# gdb: 只运行gdb（需要先通过make run来编译）
# clean: 清除编译结果

TARGET := loongarch64-unknown-linux-gnu
MODE := debug

KERNEL_ELF = target/$(TARGET)/$(MODE)/os
KERNEL_BIN = $(KERNEL_ELF).bin
KERNEL_UIMG = $(KERNEL_ELF).ui

BOARD ?= laqemu
LDBOARD = la2k1000

# 大写K转小写
ifeq ($(BOARD), 2K1000)
	BOARD = 2k1000
endif

BLOCK ?= sata

# Binutils
OBJCOPY := loongarch64-linux-gnu-objcopy
OBJDUMP := loongarch64-linux-gnu-objdump
READELF := loongarch64-linux-gnu-readelf

ifndef LOG
	LOG_OPTION := "log_off"
endif

ifeq ($(MODE), debug)
	LA_2k1000_DISABLE_EH_FRAME := -D EH_ENABLED
endif

IMG_DIR := ../easy-fs-fuse
IMG_NAME = rootfs-ubifs-ze.img
IMG := ${IMG_DIR}/$(IMG_NAME)
IMG_LN = $(shell readlink -f $(IMG_DIR))/$(IMG_NAME)

QEMU_2k1000_DIR=../util/qemu-2k1000/gz
QEMU_2k1000=$(QEMU_2k1000_DIR)/runqemu2k1000
U_IMG=$(IMG_DIR)/uImage

LA_DEBUGGER_SERIAL_PORT = $$(python3 -m serial.tools.list_ports 1A86:7523 -q | head -n 1)
LA_DEBUGGER_PORT_FREQ = $(LA_DEBUGGER_SERIAL_PORT) 115200
LA_2k1000_SERIAL_PORT = $$(python3 -m serial.tools.list_ports 067B:2303 -q | head -n 1)
LA_2k1000_PORT_FREQ = $(LA_2k1000_SERIAL_PORT) 115200
MINITERM_START_CMD=python3 -m serial.tools.miniterm --dtr 0 --rts 0 --filter direct 

LA_ENTRY_POINT = 0x9000000090000000
LA_LOAD_ADDR = 0x9000000090000000

run: clean env update-usr run-inner 

update-usr:user fat32

user: env
	@cd ../user && make rust-user BOARD=$(BOARD) MODE=$(MODE)

fat32:
ifeq ($(BOARD),laqemu)
	./buildfs.sh "$(IMG)" "laqemu" $(MODE)
else
	./buildfs.sh "$(IMG)" 2k1000 $(MODE)
endif

run-inner: build uimage do-run

build: env $(KERNEL_BIN)

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) $@ --strip-all -O binary &
	@$(OBJDUMP) $(KERNEL_ELF) -SC > target/$(TARGET)/$(MODE)/asm_all.txt 
	@$(READELF) -ash $(KERNEL_ELF) > target/$(TARGET)/$(MODE)/sec.txt &

kernel:
	@echo Platform: $(BOARD)
    ifeq ($(MODE), debug)
		@cargo build --no-default-features --features "board_$(BOARD) block_$(BLOCK) $(LOG_OPTION)" --target $(TARGET)
    else
		@cargo build --no-default-features --release --features "board_$(BOARD) block_$(BLOCK) $(LOG_OPTION)"  --target $(TARGET)
    endif

uimage: $(KERNEL_BIN)
	../util/mkimage -A loongarch -O linux -T kernel -C none -a $(LA_LOAD_ADDR) -e $(LA_ENTRY_POINT) -n NPUcore+ -d $(KERNEL_BIN) $(KERNEL_UIMG)
	-@rm $(U_IMG)
	@cp -f $$(pwd)/target/$(TARGET)/$(MODE)/os.ui $(U_IMG)

do-run:
ifeq ($(BOARD), laqemu)
#First, link the image into the directory.
	-ln -s $(IMG_LN) $(QEMU_2k1000_DIR)/$(IMG_NAME)
	@echo "========WARNING!========"
	@echo "The next command is expecting a modified runqemu2k1000 script where any potential and implicit \"current working directory\" has been replaced by a generated script storage path."
	@./run_script $(QEMU_2k1000)
else ifeq ($(BOARD), 2k1000)
	@./run_script $(MINITERM_START_CMD) $(LA_2k1000_PORT_FREQ)
endif

all: build mv
mv:
	mv $(KERNEL_BIN) ../kernel.bin

gdb:
ifeq ($(BOARD),laqemu)
	./run_script $(QEMU_2k1000) "-S"
else ifeq ($(BOARD), 2k1000)
	@./la_gdbserver minicom -D $(LA_DEBUGGER_PORT_FREQ)
endif

env: # switch-toolchain
	-(rustup target list | grep "$(TARGET) (installed)") || rustup target add $(TARGET)
	if [ "$(dpkg --list|grep "ii[[:space:]]*expect")"="" ];then true;else sudo apt install expect;fi

clean:
	@cargo clean
	-@rm ../easy-fs-fuse/uImage
	-@rm ../easy-fs-fuse/rootfs-ubifs-ze.img
	-@cd ../user && make clean

.PHONY: user update gdb new-gdb monitor .FORCE
