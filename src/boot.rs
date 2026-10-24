//! Moonlander rev A: enter STM32F303 ROM DFU after a peripheral reset.
use core::mem::MaybeUninit;
use core::ptr;

const ROM_VECTORS: u32 = 0x1FFF_D800;
const REQUEST: u32 = 0x4D4C_4446;

// NOLOAD RAM survives a software reset and is not initialized by cortex-m-rt.
// A tag and its complement avoid treating arbitrary power-on SRAM as a request.
#[unsafe(link_section = ".uninit.dfu_request")]
static mut DFU_REQUEST: MaybeUninit<[u32; 2]> = MaybeUninit::uninit();

pub fn request_rom_dfu() -> ! {
    cortex_m::interrupt::disable();
    unsafe {
        ptr::write_volatile(
            ptr::addr_of_mut!(DFU_REQUEST).cast::<[u32; 2]>(),
            [REQUEST, !REQUEST],
        );
    }
    cortex_m::asm::dsb();
    cortex_m::peripheral::SCB::sys_reset();
}

/// Called before .data/.bss initialization. Only accesses registers and .uninit.
unsafe fn enter_rom_if_requested() {
    unsafe {
        let flag = ptr::addr_of_mut!(DFU_REQUEST).cast::<[u32; 2]>();
        if ptr::read_volatile(flag) == [REQUEST, !REQUEST] {
            ptr::write_volatile(flag, [0, 0]);
            ptr::write_volatile(0xE000_ED08 as *mut u32, ROM_VECTORS);
            cortex_m::asm::dsb();
            cortex_m::asm::isb();
            cortex_m::interrupt::enable();
            cortex_m::asm::bootload(ROM_VECTORS as *const u32);
        }
    }
}

/// Common startup for power-on and return from STM32 ROM DFU.
/// Use the vector table address supplied by the linker.
#[cortex_m_rt::pre_init]
unsafe fn pre_init() {
    unsafe {
        unsafe extern "C" {
            static __vector_table: u32;
        }
        core::arch::asm!("cpsid i");
        // Disable SysTick and clear PendSV/SysTick pending bits before enabling IRQs.
        ptr::write_volatile(0xE000_E010 as *mut u32, 0);
        ptr::write_volatile(0xE000_ED04 as *mut u32, (1 << 27) | (1 << 25));

        for i in 0..8u32 {
            ptr::write_volatile((0xE000_E180 + i * 4) as *mut u32, 0xFFFF_FFFF);
            ptr::write_volatile((0xE000_E280 + i * 4) as *mut u32, 0xFFFF_FFFF);
        }
        core::arch::asm!("msr BASEPRI, {}", in(reg) 0u32);
        core::arch::asm!("cpsie f");
        enter_rom_if_requested();
        ptr::write_volatile(
            0xE000_ED08 as *mut u32,
            ptr::addr_of!(__vector_table) as u32,
        );
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
        core::arch::asm!("cpsie i");
    }
}
