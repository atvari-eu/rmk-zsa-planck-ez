/* ZSA Planck EZ memory layout (STM32F303CC / GD32F303CC)
 *
 * The firmware is flashed at 0x08000000 by the STM32 system-memory DFU
 * bootloader (no resident bootloader), so the application starts at the
 * beginning of flash.
 *
 * RMK keeps its keymap/config storage in the last two 2 KB flash sectors
 * (StorageConfig::default()), i.e. the last 4 KB. Shrink the linker-visible
 * FLASH region so firmware never links into that range; growing into it makes
 * storage init erase the running firmware.
 */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 256K - 4K
    RAM   : ORIGIN = 0x20000000, LENGTH = 40K
    /* CCMRAM : ORIGIN = 0x10000000, LENGTH = 8K */
}
