#include <avr/io.h>
#include <util/delay.h>

#define BAUD 9600
#define MYUBRR F_CPU/16/BAUD-1

void USART_Init(unsigned int ubrr) {
    // Set baud rate
    UBRR0H = (unsigned char)(ubrr>>8);
    UBRR0L = (unsigned char)ubrr;

    // Enable receiver and transmitter
    UCSR0B = (1<<TXEN0);
    
    // Set frame format: 8data, 1stop bit
    UCSR0C = (3<<UCSZ00);
}

void USART_Transmit(char data) {
    // Wait for empty transmit buffer
    while (!(UCSR0A & (1<<UDRE0)));
    
    // Put data into buffer, sends the data
    UDR0 = data;
}

int main(void) {
    // Initialize USART
    USART_Init(MYUBRR);

    char message[] = "hello world";
    int i;

    while (1) {
        for(i = 0; message[i]; i++) {
            USART_Transmit(message[i]);
        }

        _delay_ms(1000);  // delay for 1 second
    }

    return 0;
}

