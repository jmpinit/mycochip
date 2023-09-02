use criterion::{black_box, criterion_group, criterion_main, Criterion};

use mycochip::avr_simulator::AvrSimulator;

fn sim_step(avr: &mut AvrSimulator) {
    black_box(avr.step());
}

fn sim_communicate_uart(avr: &mut AvrSimulator) {
    let msg_bytes: &[u8] = "hello world".as_bytes();

    // Ensure the AVR has had time to initialize
    for _ in 0..100 {
        avr.step();
    }

    for b in msg_bytes {
        avr.write_uart('0', *b);
    }

    let mut cycles = 0;
    let mut num_received = 0;
    while num_received < msg_bytes.len() {
        avr.step();
        cycles += 1;

        let b = avr.read_uart('0');

        if b.is_some() {
            num_received += 1;
        }
    }

    // println!("It took {} cycles to receive {} bytes", cycles, msg_bytes.len());
    // println!("That's {} cycles per byte", cycles / msg_bytes.len());
}

fn sim_communicate_spi(avr: &mut AvrSimulator) {
    let msg_bytes: &[u8] = "hello world".as_bytes();

    // Ensure the AVR has had time to initialize
    for _ in 0..100 {
        avr.step();
    }

    for b in msg_bytes {
        avr.write_spi(0, *b);
    }

    let mut cycles = 0;
    let mut num_received = 0;
    while num_received < msg_bytes.len() {
        avr.step();
        cycles += 1;

        let b = avr.read_spi(0);

        if b.is_some() {
            // println!("Received byte: {:x}", b.unwrap());
            num_received += 1;
        }
    }

    // println!("It took {} cycles to receive {} bytes", cycles, msg_bytes.len());
    // println!("That's {} cycles per byte", cycles / msg_bytes.len());
}

fn criterion_benchmark(c: &mut Criterion) {
    // {
    //     let mut avr = AvrSimulator::new(
    //         "atmega328p",
    //         u32::MAX,
    //         "examples/uart_hello_world/build/uart_hello_world.elf",
    //     );
    //
    //     c.bench_function("sim_step", |b| b.iter(|| sim_step(&mut avr)));
    // }
    //
    // {
    //     let mut avr = AvrSimulator::new(
    //         "atmega328p",
    //         u32::MAX,
    //         "examples/echo/build/echo.elf",
    //     );
    //
    //     c.bench_function("sim_communicate_uart blocking", |b| b.iter(|| sim_communicate_uart(&mut avr)));
    // }
    //
    // {
    //     let mut avr = AvrSimulator::new(
    //         "atmega328p",
    //         u32::MAX,
    //         "examples/echo_int/build/echo_int.elf",
    //     );
    //
    //     c.bench_function("sim_communicate_uart interrupt", |b| b.iter(|| sim_communicate_uart(&mut avr)));
    // }

    // {
    //     let mut avr = AvrSimulator::new(
    //         "atmega328p",
    //         u32::MAX,
    //         "examples/echo_spi/build/echo_spi.elf",
    //     );
    //
    //     c.bench_function("sim_communicate_spi blocking", |b| b.iter(|| sim_communicate_spi(&mut avr)));
    // }

    {
        let mut avr = AvrSimulator::new(
            "atmega328p",
            u32::MAX,
            "examples/echo_spi_int/build/echo_spi_int.elf",
        );

        c.bench_function("sim_communicate_spi interrupt", |b| b.iter(|| sim_communicate_spi(&mut avr)));
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
