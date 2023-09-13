#include <avr/io.h>
#include <avr/interrupt.h>

#define BAUD 115200

ISR(USART_RX_vect) {
    char c = UDR0;

    // Wait for empty transmit buffer
    while (!(UCSR0A & (1<<UDRE0)));
    
    // Put data into buffer, sends the data
    UDR0 = c;
}

int main(void) {
    uint16_t ubrr = 0;

    UBRR0H = (unsigned char)(ubrr>>8);
    UBRR0L = (unsigned char)ubrr;
    UCSR0B = (1<<RXEN0) | (1<<TXEN0) | (1<<RXCIE0); // Enable RX, TX, and RX interrupt
    UCSR0C = (3<<UCSZ00);
    sei();  // Enable global interrupts

    while (1) {
        __asm__ volatile ("nop");
    }

    return 0;
}

