#include <avr/io.h>
#include <string.h>
#include <util/delay.h>
#include <avr/interrupt.h>
#include <avr/sleep.h>
#include "comms.h"

#define BAUD 115200
#define MYUBRR F_CPU/16/BAUD-1

int main(void) {
    uart_init(MYUBRR);
    comms_init();

    int messages_received = 0;

    while (1) {
        if (message_available) {
            messages_received += 1;
            message_available = false;

            char response[] = "HTTP/1.1 200 OK\r\n"
                              "Content-Length: 12\r\n"
                              "Content-Type: text/plain; charset=utf-8\r\n"
                              "\r\n"
                              "Hello World!";
            int len = strlen(response);
            comms_send(0, len, (uint8_t *)response);
        }

        sleep_mode();
    }

    return 0;
}

