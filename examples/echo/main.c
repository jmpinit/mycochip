#include <avr/io.h>
#include <util/delay.h>

#define BAUD 9600
#define MYUBRR F_CPU/16/BAUD-1

void USART_Init(unsigned int ubrr) {
    // Set baud rate
    UBRR0H = (unsigned char)(ubrr>>8);
    UBRR0L = (unsigned char)ubrr;

    // Enable receiver and transmitter
    UCSR0B = (1<<RXEN0) | (1<<TXEN0);

    // Set frame format: 8data, 1stop bit
    UCSR0C = (3<<UCSZ00);
}

void USART_Transmit(char data) {
    // Wait for empty transmit buffer
    while (!(UCSR0A & (1<<UDRE0)));
    
    // Put data into buffer, sends the data
    UDR0 = data;
}

char USART_Receive(void) {
    // Wait for data to be received
    while (!(UCSR0A & (1<<RXC0)));
    
    // Get and return received data from buffer
    return UDR0;
}

int main(void) {
    // Initialize USART
    USART_Init(MYUBRR);

    while (1) {
        char c = USART_Receive();
        USART_Transmit(c);
    }

    return 0;
}

