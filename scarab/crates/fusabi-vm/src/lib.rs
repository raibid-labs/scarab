// fusabi-vm: The AOT runtime for the Daemon
pub struct VirtualMachine;

impl VirtualMachine {
    pub fn new() -> Self { Self }
    pub fn exec_binary(&self, _bytes: &[u8]) {
        // Execute.fzb bytecode
    }
}
