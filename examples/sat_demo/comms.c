#include <string.h>
#include <stdbool.h>
#include <stdint.h>

#include <avr/interrupt.h>
#include <avr/io.h>

#include "comms.h"

#ifndef COMMS_ADDRESS
#error "COMMS_ADDRESS must be defined"
#endif

struct receiver_state state1 = {0};
struct receiver_state state2 = {0};

// Very simple back/front buffering
volatile struct receiver_state *active_state = &state1;
volatile struct receiver_state *last_state = &state2;

volatile bool message_available = false;

void reset_state(struct receiver_state *state) {
    state->index = 0;
    memset((void *)state->data, 0, COMMS_BUFFER_SIZE);
}

void comms_init() {
    reset_state(&state1);
    reset_state(&state2);
}

void comms_send(uint16_t address, uint16_t length, uint8_t *data) {
    // Address
    uart_tx(address >> 8);
    uart_tx(address & 0xFF);

    // Length
    uart_tx(length >> 8);
    uart_tx(length & 0xFF);

    // Data
    for (int i = 0; i < length; i++) {
        uart_tx(data[i]);
    }
}

ISR(USART_RX_vect) {
    uint8_t c = UDR0;

    switch (active_state->mode) {
        case RECEIVER_MODE_ADDRESS_MSB:
            active_state->address = c << 8; // Set high byte
            active_state->mode = RECEIVER_MODE_ADDRESS_LSB;
            break;
        case RECEIVER_MODE_ADDRESS_LSB:
            active_state->address |= c; // Set low byte
            active_state->mode = RECEIVER_MODE_LENGTH_MSB;
            break;
        case RECEIVER_MODE_LENGTH_MSB:
            active_state->length = c << 8; // Set high byte
            active_state->mode = RECEIVER_MODE_LENGTH_LSB;
            break;
        case RECEIVER_MODE_LENGTH_LSB:
            active_state->length |= c; // Set low byte
            active_state->mode = RECEIVER_MODE_DATA;
            break;
        case RECEIVER_MODE_DATA:
            active_state->data[active_state->index++] = c;

            if (active_state->index == active_state->length) {
                active_state->mode = RECEIVER_MODE_ADDRESS_MSB;

                if (active_state->address == COMMS_ADDRESS) {
                    // Switch to the back buffer
                    last_state = active_state;
                    active_state = active_state == &state1 ? &state2 : &state1;
                    message_available = true;
                }

                reset_state(active_state);
            }
            break;
    }
}

void uart_init(unsigned int ubrr) {
    UBRR0H = (unsigned char)(ubrr>>8);
    UBRR0L = (unsigned char)ubrr;
    UCSR0B = (1<<RXEN0) | (1<<TXEN0) | (1<<RXCIE0); // Enable RX, TX, and RX interrupt
    UCSR0C = (3<<UCSZ00);
    sei();  // Enable global interrupts
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