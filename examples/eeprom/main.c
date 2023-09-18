#include <stdio.h>
#include <avr/io.h>
#include <avr/interrupt.h>
#include <avr/eeprom.h>
#include <util/delay.h>

#define BAUD 115200
#define MYUBRR F_CPU/16/BAUD-1

void uart_init(unsigned int ubrr) {
    UBRR0H = (unsigned char)(ubrr>>8);
    UBRR0L = (unsigned char)ubrr;
    UCSR0B = (1<<RXEN0) | (1<<TXEN0) | (1<<RXCIE0); // Enable RX, TX, and RX interrupt
    UCSR0C = (3<<UCSZ00);
    sei(); // Enable global interrupts
}

void uart_tx(char data) {
    while (!(UCSR0A & (1<<UDRE0)));
    UDR0 = data;
}

void uart_tx_str(char *str) {
    for (int i = 0; str[i]; i++) {
        uart_tx(str[i]);
    }
}

int main(void) {
    uart_init(MYUBRR);

    uint8_t id = eeprom_read_byte(0);

    char message[32] = {0};
    sprintf(message, "Hello from 0x%x\n", id);

    int i;

    while (1) {
        uart_tx_str(message);
        //_delay_ms(1000);  // delay for 1 second
    }

    return 0;
}

