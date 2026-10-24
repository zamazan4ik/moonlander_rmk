/* Moonlander Mark I rev A (conservative 128 KB flash target)
 *
 * Application starts at 0x08000000, including its vector table.
 * STM32 DFU is in system ROM at 0x1FFFD800, not application flash.
 * Use 128K flash and 32K main RAM on the STM32F303CB target.
 *
 * Last 12 KB reserved: one 2 KB layout-version page at 0x0801D000,
 * then five 2 KB journal pages at 0x0801D800..0x08020000.
 * Keep in sync with src/storage_layout.rs.
 */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 128K - 12K
    RAM   : ORIGIN = 0x20000000, LENGTH = 32K
    /* CCMRAM : ORIGIN = 0x10000000, LENGTH = 8K */
}
