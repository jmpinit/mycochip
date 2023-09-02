#include <avr/io.h>
#include <string.h>
#include <util/delay.h>
#include <avr/interrupt.h>
#include "comms.h"

#define BAUD 115200
#define MYUBRR F_CPU/16/BAUD-1

/*
 * Camera receives messages telling it about the world
 */

int main(void) {
    uart_init(MYUBRR);
    comms_init();

    int messages_received = 0;

    while (1) {
        if (message_available) {
            messages_received += 1;
            message_available = false;

            char *str = "hello from cam";
            int len = strlen(str);
            comms_send(0, len, (uint8_t *)str);
        }
    }

    return 0;
}

