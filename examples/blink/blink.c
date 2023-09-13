#include <avr/io.h>
#include <util/delay.h>

int main(void) {
    // Set PB0 as an output
    DDRB |= _BV(0);

    while (1) {
        PORTB |= _BV(0);
        _delay_ms(1000);

        PORTB &= ~_BV(0);
        _delay_ms(1000);
    }

    return 0;
}

