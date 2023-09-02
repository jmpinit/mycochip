#ifndef H_COMMS
#define H_COMMS

#include <stdbool.h>

#define COMMS_BUFFER_SIZE 780

enum receiver_mode {
    RECEIVER_MODE_ADDRESS_MSB,
    RECEIVER_MODE_ADDRESS_LSB,
    RECEIVER_MODE_LENGTH_MSB,
    RECEIVER_MODE_LENGTH_LSB,
    RECEIVER_MODE_DATA,
};

struct receiver_state {
    uint16_t address;
    uint16_t length;
    int index;
    uint8_t data[256];
    enum receiver_mode mode;
};

void comms_init();
void comms_send(uint16_t address, uint16_t length, uint8_t *data);

extern volatile bool message_available;
extern volatile struct receiver_state *last_state;

void uart_init(unsigned int ubrr);
void uart_tx(char data);
void uart_tx_str(char *str);

#endif
