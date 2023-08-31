CC=avr-gcc
OBJCOPY=avr-objcopy
MCU=atmega328p
CFLAGS=-Os -mmcu=$(MCU) -DF_CPU=1000000
TARGET=uart_hello_world

# Build Rules
all: build/$(TARGET).hex

build/$(TARGET).hex: build/$(TARGET).elf
	mkdir -p build
	$(OBJCOPY) -O ihex -R .eeprom $< $@

build/$(TARGET).elf: $(TARGET).c
	mkdir -p build
	$(CC) $(CFLAGS) -o $@ $<

clean:
	rm -f build/$(TARGET).elf build/$(TARGET).hex
	rmdir build

.PHONY: all flash clean

