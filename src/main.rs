#![no_std]
#![no_main]

use cortex_m::{iprintln, Peripherals};
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f3::stm32f303::interrupt;

// Reset & Clock Control
const RCC_ADDR: u32 = 0x4002_1000;
const RCC_AHBENR_OFFSET: u32 = 0x14; // Advanced High Performance Bus Enable Register
const RCC_AHBENR: u32 = RCC_ADDR + RCC_AHBENR_OFFSET;
const RCC_AHB2ENR_OFFSET: u32 = 0x18;
const RCC_AHB2ENR: u32 = RCC_ADDR + RCC_AHB2ENR_OFFSET;

// General Purpose IO Port A
const GPIOA_ADDR: u32 = 0x4800_0000;
const GPIOA_IDR_OFFSET: u32 = 0x10; // Input Data Register
const GPIOA_IDR_ADDR: u32 = GPIOA_ADDR + GPIOA_IDR_OFFSET;

// General Purpose IO Port E
const GPIOE_ADDR: u32 = 0x4800_1000;
const GPIO_MODER_OFFSET: u32 = 0x00; // Mode Register
const GPIO_BSRR_OFFSET: u32 = 0x18; // Bit Set/Reset Register
const GPIOE_MODER_ADDR: u32 = GPIOE_ADDR + GPIO_MODER_OFFSET;
const GPIOE_BSRR_ADDR: u32 = GPIOE_ADDR + GPIO_BSRR_OFFSET;

// Interrupts
const SYSCFG_ADDR: u32 = 0x4001_0000;
const SYSCFG_EXTICR1_OFFSET: u32 = 0x8;
const SYSCFG_EXTICR1_ADDR: u32 = SYSCFG_ADDR + SYSCFG_EXTICR1_OFFSET;

const EXTI_ADDR: u32 = 0x4001_0400;
const EXTI_IMR1_OFFSET: u32 = 0x00; // Input mask
const EXTI_RTSR1_OFFSET: u32 = 0x08; // Rising trigger select register
const EXTI_FTSR1_OFFSET: u32 = 0x0C; // Falling trigger select register
const EXTI_PR1_OFFSET: u32 = 0x14; // Pending

// Nested Vector Inierrupt Controller
const NVIC_ADDR: u32 = 0xe000_e100;
const NVIC_ISER0_OFFSET: u32 = 0x0; // Interrupt Set Enable

const OUTPUT_PIN: i32 = 15; // On the STM32F3Discovery the West LED of the compass (green) is Port E.13

fn setup_gpioe_pin_as_output(pin: i32) {
    unsafe {
        // Enable the GPIOE peripheral
        let rcc_ahbenr = &*(RCC_AHBENR as *mut volatile_register::RW<u32>);
        rcc_ahbenr.modify(|r| r | (1 << 21)); // Bit 21 is the I/O port E clock enable

        // Set desired pin as output, controlled by MODER
        let gpioe_moder = &*(GPIOE_MODER_ADDR as *mut volatile_register::RW<u32>);
        let pin_shift = pin * 2; // Calculate the bit position based on pin number
        let mask = 0b11 << pin_shift; // Create a mask for the pin bits in the register (2 bits per pin)
        let mode = 0b01; // General purpose output mode
        let set_mode = mode << pin_shift; // Shift the mode to the correct position

        gpioe_moder.modify(|r| (r & !mask) | set_mode); // First clear the two bits of this pins mode, then OR with the new (bit-shifted) value
    }
}

fn setup_gpioa_pin_as_input() {
    // On the STM32F3Discovery the USER pushbutton is connected to Port A.0

    unsafe {
        // Enable the GPIOA peripheral
        let ahbenr = &*(RCC_AHBENR as *mut volatile_register::RW<u32>);
        ahbenr.modify(|r| r | (1 << 17)); // Bit 17 is the I/O port A clock enable

        // By default the pin is set to be an input, so no further config is needed
    }
}

fn setup_input_interrupt() {
    unsafe {
        // Enable the clock to the SYSCFG peripheral
        let ahb2enr = &*(RCC_AHB2ENR as *mut volatile_register::RW<u32>);
        ahb2enr.modify(|r| r | (1 << 0)); //Set Bit 0 (SYSCFGEN) to enable System Config clock

        // Connect EXTI0 line to Port A.0
        let exticr1 = &*(SYSCFG_EXTICR1_ADDR as *mut volatile_register::RW<u32>);
        exticr1.modify(|r| r & !0b111); //Set lowest 3 bits 000 of SYSCFG_EXTICR1 to enable select Port A.0

        // The pin needs to be set to an input, but this is the default after reset so no action needed
        // This is controlled by GPIOA_MODER

        // Configure the interrupt source
        // This is controlled by SYSCFG_EXTICR1
        // On reset Port A.0 is already configured so no action is necessary

        // Unmask interrupt for Port A.0
        // This is controlled in EXTI_IMR1
        let exti_imr1 = &*((EXTI_ADDR + EXTI_IMR1_OFFSET) as *mut volatile_register::RW<u32>);
        exti_imr1.modify(|r| r | (1 << 0));

        // Configure the trigger to use a rising trigger
        // This is controlled by EXTI_RTSR1
        let exti_rtsr1 = &*((EXTI_ADDR + EXTI_RTSR1_OFFSET) as *mut volatile_register::RW<u32>);
        exti_rtsr1.modify(|r| r | (1 << 0));

        // Configure the trigger to use a rising trigger
        // This is controlled by EXTI_FTSR1
        let exti_ftsr1 = &*((EXTI_ADDR + EXTI_FTSR1_OFFSET) as *mut volatile_register::RW<u32>);
        exti_ftsr1.modify(|r| r | (1 << 0));

        // Enable the interrupt
        // This is controlled by NVIC_ISER0 (Interrupt Set Enable)
        let nvic_iser0 = &*((NVIC_ADDR + NVIC_ISER0_OFFSET) as *mut volatile_register::RW<u32>);
        nvic_iser0.modify(|r| r | (1 << 6));
    }
}

fn set_led_on(pin: i32) {
    unsafe {
        // BSRR is the register used to set or clear individual pins
        let bsrr = &*(GPIOE_BSRR_ADDR as *mut volatile_register::RW<u32>);
        bsrr.write(1 << pin); // A pin is set by setting the bit in the lower 16 bits of the BSRR
    }
}

fn set_led_off(pin: i32) {
    unsafe {
        // BSRR is the register used to set or clear individual pins
        let bsrr = &*(GPIOE_BSRR_ADDR as *mut volatile_register::RW<u32>);
        bsrr.write(1 << (16 + pin)); // A pin is cleared by setting the bit in the top 16 bits of the BSRR
    }
}

fn read_input() -> bool {
    unsafe {
        // IDR is the value at the input of the port
        let idr = &*(GPIOA_IDR_ADDR as *mut volatile_register::RW<u32>);
        let value = idr.read();

        let mask = 0x1;

        value & mask > 0
    }
}

#[interrupt]
fn EXTI0() {
    unsafe {
        // PR1 is the Pending Registers, which indicates which triggers have occurred
        let exti_pr1 = &*((EXTI_ADDR + EXTI_PR1_OFFSET) as *mut volatile_register::RW<u32>);
        exti_pr1.modify(|r| (r | 0x1)); // Set the bit to clear the pending interrupt
    }

    let switch_pressed = read_input();

    if switch_pressed {
        set_led_on(OUTPUT_PIN);
    } else {
        set_led_off(OUTPUT_PIN);
    }
}

#[entry]
fn main() -> ! {
    let mut p = Peripherals::take().unwrap();
    let stim = &mut p.ITM.stim[0];

    setup_gpioe_pin_as_output(OUTPUT_PIN); // There are 8 LED outputs connected to Port E.8 --> 15. Change OUTPUT_PIN to use a different LED

    setup_gpioa_pin_as_input(); // On the STM32F3Discovery the USER pushbutton is connected to Port A.0

    setup_input_interrupt();

    let mut i = 0;
    loop {
        iprintln!(stim, "Hello, world! {}", i);
        i += 1;
    }
}
