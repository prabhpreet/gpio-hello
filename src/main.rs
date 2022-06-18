#![no_std]
#![no_main]

// pick a panicking behavior
//use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;
use msp432e401y_rs::Peripherals;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    let mut run_once = true;

    //Follow steps as per MSP432E401Y slau723a.pdf, page 1199, section 17.4: Initialization & Configuration
    //1. Set RCGCGPIO (Run Mode Clock Gating Control) R12 (Port N) to enable GPIO N peripheral
    peripherals
        .SYSCTL
        .rcgcgpio
        .modify(|_, w| w.sysctl_rcgcgpio_r12().set_bit());

    //2. Set GPIODIR pin bits to configure as output
    peripherals
        .GPION
        .dir
        .modify(|r, w| unsafe { w.bits(r.bits() | 0x0000_0003) });

    //3. Clear GPIOAFSEL bits for GPIO Alternate Function to be controlled by GPIO (not alternative peripherals)
    peripherals
        .GPION
        .afsel
        .modify(|r, w| unsafe { w.bits(r.bits() & !(0x0000_0003)) });

    //4. Clear EDMn pin in GPIOPC register (extended drive enable) for only 2mA drive
    //GPIOPC[EDMn] = 0, GPIODR2R=1 (on reset) for 2mA Drive
    peripherals.GPION.pc.write(|w| unsafe {
        let w = w.gpio_pc_edm0().gpio_pc_edm0_disable();
        let w = w.gpio_pc_edm1().bits(0x0);
        w
    });

    //5. Clear the GPIODR4R register bits
    peripherals
        .GPION
        .dr4r
        .modify(|r, w| unsafe { w.bits(r.bits() & !((1 << 0) | (1 << 1))) });

    //6. Clear the GPIODR8R register bits
    peripherals
        .GPION
        .dr8r
        .modify(|r, w| unsafe { w.bits(r.bits() & !((1 << 0) | (1 << 1))) });

    //7. Clear the GPIODR12R register bits
    peripherals
        .GPION
        .dr12r
        .modify(|r, w| unsafe { w.bits(r.bits() & !((1 << 0) | (1 << 1))) });

    //8. Clear GPIOODR, PUR, PDR = 0
    peripherals.GPION.odr.reset();
    peripherals.GPION.pur.reset();
    peripherals.GPION.pdr.reset();

    //9. To enable GPIO pins as digital I/Os, set the appropriate DEN bit in the GPIODEN register

    peripherals
        .GPION
        .den
        .modify(|r, w| unsafe { w.bits(r.bits() | ((1 << 0) | (1 << 1))) });

    //Write to run on both LEDs
    peripherals
        .GPION
        .data
        .modify(|r, w| unsafe { w.bits(r.bits() | 1 << 0) });
    peripherals
        .GPION
        .data
        .modify(|r, w| unsafe { w.bits(r.bits() | 1 << 1) });

    loop {
        // your code goes here
        if (run_once) {
            run_once = false;
        }
        asm::nop(); // To not have main optimize to abort in release mode, remove when you add code
    }
}
