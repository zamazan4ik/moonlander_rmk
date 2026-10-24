//! Internal flash partition, shared with the host regression tests.
pub const FLASH_BYTES: u32 = 128 * 1024;
pub const PAGE_BYTES: u32 = 2048;
pub const STORAGE_SECTORS: u8 = 5;
pub const STORAGE_START: u32 = FLASH_BYTES - STORAGE_SECTORS as u32 * PAGE_BYTES;
pub const MIGRATION_START: u32 = STORAGE_START - PAGE_BYTES;
pub const LAYOUT_TAG: [u8; 8] = *b"MNLD0004";

pub fn needs_migration(stored_tag: &[u8; 8]) -> bool {
    stored_tag != &LAYOUT_TAG
}

/// Prepare the versioned journal; the marker and journal use separate pages.
#[cfg(target_arch = "arm")]
pub fn prepare(
    flash: &mut embassy_stm32::flash::Flash<'_, embassy_stm32::flash::Blocking>,
) -> Result<rmk::config::StorageConfig, embassy_stm32::flash::Error> {
    let mut stored_tag = [0; 8];
    flash.blocking_read(MIGRATION_START, &mut stored_tag)?;
    if needs_migration(&stored_tag) {
        // Erase before committing the marker. After an interrupted migration,
        // either migration retries or RMK initializes an empty/invalid journal.
        flash.blocking_erase(MIGRATION_START, FLASH_BYTES)?;
        flash.blocking_write(MIGRATION_START, &LAYOUT_TAG)?;
    }
    Ok(rmk::config::StorageConfig {
        start_addr: STORAGE_START as usize,
        num_sectors: STORAGE_SECTORS,
        clear_layout: cfg!(feature = "reset-layout"),
        ..Default::default()
    })
}
