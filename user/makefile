TOOLPREFIX	= riscv64-elf-
CC = $(TOOLPREFIX)gcc
AS = $(TOOLPREFIX)gas
LD = $(TOOLPREFIX)ld
OBJCOPY = $(TOOLPREFIX)objcopy
OBJDUMP = $(TOOLPREFIX)objdump

CFLAGS = -Wall -Werror -fno-omit-frame-pointer -gdwarf-2
CFLAGS += -MD
CFLAGS += -mcmodel=medany
CFLAGS += -ffreestanding -fno-common -nostdlib -mno-relax
CFLAGS += -I.
CFLAGS += $(shell $(CC) -fno-stack-protector -E -x c /dev/null >/dev/null 2>&1 && echo -fno-stack-protector)
CFLAGS += -DNCPU=$(NCPU)
LDFLAGS = -z max-page-size=4096



INCLUDES		:= -I./include/
USER_LIB		:= 
USER_TARGET		:= loop.b

.PHONY: clean

build: $(USER_TARGET)

%.b: entry.o %.o $(USER_LIB)
	$(LD) -o $@ $(LDFLAGS) -G 0 -static -n -nostdlib -T linker.lds $^

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

%.o: %.S
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

clean:
	rm -rf *.o *~ *.d *.asm *.b
