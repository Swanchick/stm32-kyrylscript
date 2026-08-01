use alloc::vec::Vec;

pub enum ProgramatorStates {
    Ready,
    LoadSize { le_bytes: [u8; 4], step: u8 },
    Loading { size: u32 },
    Loaded,
}
