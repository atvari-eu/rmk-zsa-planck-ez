//! STM32 system-memory (ROM) DFU bootloader entry, QMK `stm32_dfu.c` style:
//! `jump_to_dfu()` stashes a magic word just below the top of SRAM (RAM
//! survives a system reset), resets the chip, and
//! [`stm32_dfu_entry_check`] — called from `#[cortex_m_rt::pre_init]` —
//! jumps into the ROM vector table when the reset comes back.
//!
//! Addresses per ST AN2606 for STM32F303x8/xB/xC: system memory (ROM DFU)
//! at `0x1FFF_D800`; magic word at `0x2000_9FFC` (first word below the
//! 40 KB main SRAM of an F303CC).
//!
//! This mirrors what QMK provides on ZSA stock firmware; written locally so
//! no fork of rmk is needed (rmk itself comes from crates.io).

/// Jump to the ROM DFU bootloader: stash the magic and reset.
pub fn jump_to_dfu() {
    #[cfg(all(target_arch = "arm", target_os = "none", any(target_abi = "eabi", target_abi = "eabihf")))]
    unsafe {
        core::ptr::write_volatile(0x2000_9FFC as *mut u32, 0xDEAD_BEEF);
        defmt::warn!("Jumping to STM32 ROM DFU!");
        cortex_m::peripheral::SCB::sys_reset();
    }

    #[cfg(not(all(target_arch = "arm", target_os = "none", any(target_abi = "eabi", target_abi = "eabihf"))))]
    defmt::warn!("Jump to DFU not supported on this target");
}

/// Hand control into the ROM DFU when [`jump_to_dfu`] left the magic word.
/// Must be called from `#[cortex_m_rt::pre_init]`, before anything touches
/// SRAM or clocks.
pub unsafe fn stm32_dfu_entry_check() {
    const MAGIC_ADDR: *mut u32 = 0x2000_9FFC as *mut u32;
    const ROM_ADDR: *const u32 = 0x1FFF_D800 as *const u32;

    if unsafe { core::ptr::read_volatile(MAGIC_ADDR) } != 0xDEAD_BEEF {
        return;
    }
    // Hand the magic back so an aborted jump can't wedge us into the ROM.
    unsafe { core::ptr::write_volatile(MAGIC_ADDR, 0) };

    // Mask interrupts for the jump; after the reset everything is idle anyway.
    unsafe { core::arch::asm!("cpsid i") };

    // Reads the initial MSP and reset vector from the ROM table, then jumps.
    unsafe { cortex_m::asm::bootload(ROM_ADDR) };
}
