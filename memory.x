/* Moonlander Mark I rev A (conservative 128 KB flash target)
 *
 * Application starts at 0x08000000, including its vector table.
 * STM32 DFU is in system ROM at 0x1FFFD800, not application flash.
 * Use 128K flash and 32K main RAM on the STM32F303CB target.
 * No persistent storage: the whole flash is available to the application.
 */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 128K
    RAM   : ORIGIN = 0x20000000, LENGTH = 32K
    CCMRAM : ORIGIN = 0x10000000, LENGTH = 8K
}
