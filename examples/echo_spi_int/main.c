#include <avr/io.h>
#include <avr/interrupt.h>

// Initialize SPI as slave
void SPI_init_slave(void) {
    // Set MISO as output
    DDRB |= (1 << DDB4);

    // Enable SPI, set as slave, enable SPI interrupt
    SPCR = (1 << SPE) | (1 << SPIE);
}

// Function to send data
void SPI_transmit(unsigned char data) {
    // Load data into the buffer
    SPDR = data;

    // Wait until transmission complete
    while (!(SPSR & (1 << SPIF)));
}

// SPI serial transfer interrupt
ISR(SPI_STC_vect) {
        // Fetch received data
//        unsigned char data = SPDR;
//
//        // Echo back the data
//        SPI_transmit(data);
}

int main(void) {
    // Initialize SPI as slave
    SPI_init_slave();

    // Enable global interrupts
//    sei();

    while (1) {
        __asm__ volatile ("nop");
    }

    return 0;
}
