// Program metadata for `picotool info`.
// This isn't needed, but it's recommended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
  embassy_rp::binary_info::rp_program_name!(c"Smart Favag"),
  embassy_rp::binary_info::rp_program_description!(c"Control Program for the Smart Favag slave clock"),
  embassy_rp::binary_info::rp_cargo_version!(),
  embassy_rp::binary_info::rp_program_build_attribute!(),
];

// Features to enable RP2040 or RP235x support
// required since there is no_std support for reading .env file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chip {
  RP2040,
  RP235x,
}

#[cfg(feature = "rp2040")]
pub const TARGET_CHIP: Chip = Chip::RP2040;
#[cfg(not(feature = "rp2040"))]
pub const TARGET_CHIP: Chip = Chip::RP235x;
