#![no_std]
#![no_main]

// pick a panicking behavior
//use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use msp432e401y_rs::Peripherals;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    let mut prev_input = false;
    let mut run_task = false;

    //Follow steps as per MSP432E401Y slau723a.pdf, page 1199, section 17.4: Initialization & Configuration
    //1. Set RCGCGPIO (Run Mode Clock Gating Control)
    //R12 (Port N) to enable GPIO N peripheral
    //R8  (Port J) to enable GPIO J peripheral
    let rcgcgpio = &peripherals.SYSCTL.rcgcgpio;
    rcgcgpio.modify(|_, w| w.sysctl_rcgcgpio_r12().set_bit());
    rcgcgpio.modify(|_, w| w.sysctl_rcgcgpio_r8().set_bit());

    let portn = peripherals.GPION;
    let portj = peripherals.GPIOJ;

    //2. Set GPIODIR pin bits to configure as output for Port N, Input for Port J

    portn
        .dir
        .modify(|r, w| unsafe { w.bits(r.bits() | 0x0000_0003) });

    portj
        .dir
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 0)) });

    //3. Clear GPIOAFSEL bits for GPIO Alternate Function to be controlled by GPIO (not alternative peripherals)
    portn
        .afsel
        .modify(|r, w| unsafe { w.bits(r.bits() & !(0x0000_0003)) });

    portj
        .afsel
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 0)) });

    //4. Clear EDMn pin in GPIOPC register (extended drive enable) for only 2mA drive
    //GPIOPC[EDMn] = 0, GPIODR2R=1 (on reset) for 2mA Drive
    portn.pc.write(|w| unsafe {
        let w = w.gpio_pc_edm0().gpio_pc_edm0_disable();
        let w = w.gpio_pc_edm1().bits(0x0);
        w
    });

    portj.pc.write(|w| unsafe {
        let w = w.gpio_pc_edm0().gpio_pc_edm0_disable();
        let w = w.gpio_pc_edm1().bits(0x0);
        w
    });

    //5. Clear the GPIODR4R register bits
    portn
        .dr4r
        .modify(|r, w| unsafe { w.bits(r.bits() & !((1 << 0) | (1 << 1))) });
    portj
        .dr4r
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 0)) });

    //6. Clear the GPIODR8R register bits
    portn
        .dr8r
        .modify(|r, w| unsafe { w.bits(r.bits() & !((1 << 0) | (1 << 1))) });
    portj
        .dr8r
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 0)) });

    //7. Clear the GPIODR12R register bits
    portn
        .dr12r
        .modify(|r, w| unsafe { w.bits(r.bits() & !((1 << 0) | (1 << 1))) });
    portj
        .dr12r
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 0)) });

    //8. Clear GPIOODR, PUR, PDR = 0 for Port n, set PUR for Port J
    portn.odr.reset();
    portn.pur.reset();
    portn.pdr.reset();

    portj.odr.reset();
    portj.pur.write(|w| unsafe { w.bits(1 << 0) });
    portj.pdr.reset();

    //9. To enable GPIO pins as digital I/Os, set the appropriate DEN bit in the GPIODEN register

    portn
        .den
        .modify(|r, w| unsafe { w.bits(r.bits() | ((1 << 0) | (1 << 1))) });
    portj
        .den
        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 0)) });

    //Write to run on both LEDs
    portn
        .data
        .modify(|r, w| unsafe { w.bits(r.bits() | 1 << 0) });

    loop {
        let bits = portj.data.read().bits() & (1 << 0);
        if (bits != 0) {
            if (prev_input == false) {
                run_task = true;
            }
            prev_input = true;
        } else {
            prev_input = false;
        }
        if (run_task) {
            portn
                .data
                .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 1)) });
            run_task = false;
        }
        asm::nop(); // To not have main optimize to abort in release mode, remove when you add code
    }
}
