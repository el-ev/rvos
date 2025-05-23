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
QEMU_ARGS += -cpu max
QEMU_ARGS += -smp 4
QEMU_ARGS += -m 2G
QEMU_ARGS += -machine $(BOARD)
QEMU_ARGS += -drive file=$(ROOT)/dev0,format=raw,if=none,id=hd0
QEMU_ARGS += -device virtio-blk-device,drive=hd0
QEMU_ARGS += -nographic
QEMU_ARGS += -bios $(SBI)
QEMU_ARGS += -device loader,file=$(KERNEL),addr=0x80200001
QEMU_ARGS += -kernel $(KERNEL)

build:
	cd kernel && cargo build $(BUILDARGS) --bin kernel

run: build kill
	$(QEMU) $(QEMU_ARGS)

debug: build kill
	$(QEMU) $(QEMU_ARGS) -s -S &
	@if [ "$(shell uname)" = "Darwin" ]; then \
		lldb -o "target create $(KERNEL)" -o "gdb-remote 1234"; \
	else \
		gdb -ex "target remote tcp::1234" -ex "symbol-file $(KERNEL)"; \
	fi

debug_qemu: build
	$(QEMU) $(QEMU_ARGS) -s -S

objdump: build
	$(OBJDUMP) -DS $(KERNEL) > kernel.asm

kill:
	killall $(QEMU) > /dev/null || true

fslib:
	cd crates/fs && RUSTFLAGS="--crate-type=staticlib" cargo build $(BUILDARGS) && cbindgen --config cbindgen.toml --output c/libfs.h
	cp $(TARGET_DIR)/libfs.a crates/fs/c/
	
user:
	cd user_c && make
	cd user_rust && cargo build $(BUILDARGS) --bin *
	
clean:
	cargo clean
	cd user && make clean

.PHONY: run debug objdump kill clean user fslib