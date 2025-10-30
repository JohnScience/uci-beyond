use crate::engine_commands::{IdBlock, UciOptionBlock};

pub struct UciCommandResponse {
    id_block: IdBlock,
    option_block: UciOptionBlock,
}
