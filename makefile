TARGET := riscv64gc-unknown-none-elf
PROFILE ?= debug
BOARD   ?= virt

ROOT			:= $(shell pwd)
TARGET_DIR 	:= $(ROOT)/target/$(TARGET)/$(PROFILE)

SBI     ?= $(ROOT)/sbi/opensbi.bin

KERNEL  := $(TARGET_DIR)/kernel

OBJDUMP := rust-objdump --arch-name=riscv64

QEMU    := qemu-system-riscv64

BUILDARGS := --target $(TARGET)

ifeq ($(PROFILE),release)
BUILDARGS += --release
endif

QEMU_ARGS :=
QEMU_ARGS += -smp 8
QEMU_ARGS += -m 2G
QEMU_ARGS += -machine $(BOARD)
QEMU_ARGS += -nographic
QEMU_ARGS += -bios $(SBI)
QEMU_ARGS += -device loader,file=$(KERNEL),addr=0x80200000
QEMU_ARGS += -kernel $(KERNEL)

build:
	cd kernel && cargo build $(BUILDARGS)

run: build
	$(QEMU) $(QEMU_ARGS)

debug: build
	$(QEMU) $(QEMU_ARGS) -s -S & 
	gdb-multiarch -ex "target remote tcp::1234" -ex "symbol-file $(KERNEL)" && killall $(QEMU)

objdump: build
	$(OBJDUMP) -d $(KERNEL) > kernel.asm

clean:
	cargo clean

.PHONY: run debug objdump clean