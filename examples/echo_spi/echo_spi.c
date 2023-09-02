#include <avr/io.h>
#include <util/delay.h>

// Initialize SPI as slave
void SPI_init_slave(void) {
    // Set MISO as output
    DDRB |= (1 << DDB4);

    // Enable SPI, set as slave
    SPCR = (1 << SPE);
}

// Function to send and receive data
unsigned char SPI_tranceiver(unsigned char data) {
    // Load data into the buffer
    SPDR = data;

    // Wait until transmission complete
    while (!(SPSR & (1 << SPIF)));

    // Return received data
    return SPDR;
}

int main(void) {
    // Initialize SPI as slave
    SPI_init_slave();

    // Temporary data variable
    unsigned char data;

    while (1) {
        // Receive data
        data = SPI_tranceiver(0x00); // 0x00 or any data to send during receive

        // Echo back received data
        SPI_tranceiver(data);
    }

    return 0;
}
