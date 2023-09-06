#include <avr/io.h>
#include <string.h>
#include <util/delay.h>
#include <avr/interrupt.h>

#define BAUD 9600
#define MYUBRR F_CPU/16/BAUD-1

#define BUFFER_SIZE 1400

volatile char buffer[BUFFER_SIZE];
volatile int buffer_idx = 0;
volatile int end_headers = 0;

void USART_Init(unsigned int ubrr) {
    UBRR0H = (unsigned char)(ubrr>>8);
    UBRR0L = (unsigned char)ubrr;
    UCSR0B = (1<<RXEN0) | (1<<TXEN0) | (1<<RXCIE0); // Enable RX, TX, and RX interrupt
    UCSR0C = (3<<UCSZ00);
    sei();  // Enable global interrupts
}

void USART_Transmit(char data) {
    while (!(UCSR0A & (1<<UDRE0)));
    UDR0 = data;
}

void send_http_response() {
    char response[] = "HTTP/1.1 200 OK\r\n"
                      "Content-Length: 18\r\n"
                      "Content-Type: text/plain; charset=utf-8\r\n"
                      "\r\n"
                      "Hello from an AVR!";
    for (int i = 0; response[i]; i++) {
        USART_Transmit(response[i]);
    }

    // End of transmission
    USART_Transmit(4);
}

// Interrupt service routine for UART Receive Complete
ISR(USART_RX_vect) {
    char received_char = UDR0; // Read received character from buffer

    if (buffer_idx == 0 && received_char != 'G') {
        return;
    }

    // Store received characters into the buffer
    if (buffer_idx < BUFFER_SIZE - 1) {
        buffer[buffer_idx] = received_char;
        buffer_idx++;
        buffer[buffer_idx] = '\0';  // Null-terminate
    }

    // Check for the end of HTTP headers
    if (buffer_idx >= 4 &&
        buffer[buffer_idx-4] == '\r' &&
        buffer[buffer_idx-3] == '\n' &&
        buffer[buffer_idx-2] == '\r' &&
        buffer[buffer_idx-1] == '\n') {
        end_headers = 1;
    }
}

int main(void) {
    USART_Init(MYUBRR);

    while (1) {
        // If headers are complete, check if it's a GET request
        if (end_headers) {
            if (strncmp((const char *)buffer, "GET", 3) == 0) {
                send_http_response();
            }

            // Reset everything for the next request
            memset((void *)buffer, 0, BUFFER_SIZE);
            buffer_idx = 0;
            end_headers = 0;
        }
    }

    return 0;
}

