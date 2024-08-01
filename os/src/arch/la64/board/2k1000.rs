use crate::config::HIGH_BASE_EIGHT;

pub const MMIO: &[(usize, usize)] = &[
    (0x400E_0000, 0x1_0000)
];

pub const BLOCK_SZ: usize = 4096;
// warning: 不能移除“ + HIGH_BASE_EIGHT”，会导致开发板上地址错误
pub const UART_BASE: usize = 0x1FE2_0000 + HIGH_BASE_EIGHT;
pub const ACPI_BASE: usize = 0x1FE2_7000 + HIGH_BASE_EIGHT;
