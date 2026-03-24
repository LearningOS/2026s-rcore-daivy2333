<pir>
<meta>
name: os
root: /home/daivy/projects/2026s-rcore-daivy2333/os
profile: generic
lang: ASM,LD,Rust,S
</meta>
<units>
u0: build.rs type=Rust role=lib module=os
u1: src/linker.ld type=LD role=lib module=src
u2: src/timer.rs type=Rust role=lib module=src
u3: src/lang_items.rs type=Rust role=lib module=src
u4: src/config.rs type=Rust role=lib module=src
u5: src/loader.rs type=Rust role=lib module=src
u6: src/logging.rs type=Rust role=lib module=src
u7: src/batch.rs type=Rust role=lib module=src
u8: src/link_app.S type=S role=lib module=src
u9: src/sbi.rs type=Rust role=lib module=src
u10: src/entry.asm type=ASM role=lib module=src
u11: src/heap_alloc.rs type=Rust role=lib module=src
u12: src/console.rs type=Rust role=lib module=src
u13: src/main.rs type=Rust role=lib module=src
u14: src/syscall/process.rs type=Rust role=lib module=syscall
u15: src/syscall/mod.rs type=Rust role=lib module=syscall
u16: src/syscall/fs.rs type=Rust role=lib module=syscall
u17: src/mm/frame_allocator.rs type=Rust role=lib module=mm
u18: src/mm/memory_set.rs type=Rust role=lib module=mm
u19: src/mm/heap_allocator.rs type=Rust role=lib module=mm
u20: src/mm/page_table.rs type=Rust role=lib module=mm
u21: src/mm/address.rs type=Rust role=lib module=mm
u22: src/mm/mod.rs type=Rust role=lib module=mm
u23: src/trap/context.rs type=Rust role=lib module=trap
u24: src/trap/mod.rs type=Rust role=lib module=trap
u25: src/trap/trap.S type=S role=lib module=trap
u26: src/boards/qemu.rs type=Rust role=lib module=boards
u27: src/sync/up.rs type=Rust role=lib module=sync
u28: src/sync/mod.rs type=Rust role=lib module=sync
u29: src/task/switch.S type=S role=lib module=task
u30: src/task/context.rs type=Rust role=lib module=task
u31: src/task/task.rs type=Rust role=lib module=task
u32: src/task/switch.rs type=Rust role=lib module=task
u33: src/task/mod.rs type=Rust role=lib module=task
</units>
<dependency-pool>
d0: call:u13#rust_main
d1: call:u24#trap_from_kernel
d2: use:[address::{StepByOne, VPNRange}]
d3: use:[alloc::boxed::Box]
d4: use:[alloc::collections::BTreeMap]
d5: use:[alloc::sync::Arc]
d6: use:[alloc::vec::Vec]
d7: use:[alloc::vec]
d8: use:[bitflags::*]
d9: use:[buddy_system_allocator::LockedHeap]
d10: use:[core::arch::asm]
d11: use:[core::arch::{asm, global_asm}]
d12: use:[core::cell::{RefCell, RefMut}]
d13: use:[core::fmt::{self, Debug, Formatter}]
d14: use:[core::fmt::{self, Write}]
d15: use:[core::panic::PanicInfo]
d16: use:[crate::board::QEMUExit]
d17: use:[crate::config::CLOCK_FREQ]
d18: use:[crate::config::KERNEL_HEAP_SIZE]
d19: use:[crate::config::MEMORY_END]
d20: use:[crate::config::PAGE_SIZE]
d21: use:[crate::config::TRAP_CONTEXT_BASE]
d22: use:[crate::config::{
    KERNEL_STACK_SIZE, MEMORY_END, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT_BASE, USER_STACK_SIZE,
}]
d23: use:[crate::config::{PAGE_SIZE, PAGE_SIZE_BITS}]
d24: use:[crate::config::{TRAMPOLINE, TRAP_CONTEXT_BASE}]
d25: use:[crate::loader::{get_app_data, get_num_app}]
d26: use:[crate::mm::translated_byte_buffer]
d27: use:[crate::mm::{
    kernel_stack_position, MapPermission, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE,
}]
d28: use:[crate::mm::{MapPermission, VirtAddr}]
d29: use:[crate::mm::{translated_refmut, MapPermission}]
d30: use:[crate::sbi::console_putchar]
d31: use:[crate::sbi::set_timer]
d32: use:[crate::sbi::shutdown]
d33: use:[crate::sync::UPSafeCell]
d34: use:[crate::syscall::syscall]
d35: use:[crate::task::current_user_token]
d36: use:[crate::task::increment_syscall_count]
d37: use:[crate::task::{
    current_trap_cx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
}]
d38: use:[crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, current_user_token, current_mmap, current_munmap, get_syscall_count}]
d39: use:[crate::timer::get_time_us]
d40: use:[crate::timer::set_next_trigger]
d41: use:[crate::trap::TrapContext]
d42: use:[crate::trap::trap_return]
d43: use:[crate::trap::{trap_handler, TrapContext}]
d44: use:[fs::*]
d45: use:[lazy_static::*]
d46: use:[log::{Level, LevelFilter, Log, Metadata, Record}]
d47: use:[process::*]
d48: use:[riscv::register::satp]
d49: use:[riscv::register::sepc]
d50: use:[riscv::register::sstatus::{self, Sstatus, SPP}]
d51: use:[riscv::register::time]
d52: use:[riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
}]
d53: use:[stdlib:rust]
d54: use:[super::PageTableEntry]
d55: use:[super::TaskContext]
d56: use:[super::{PTEFlags, PageTable, PageTableEntry}]
d57: use:[super::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum}]
d58: use:[super::{PhysAddr, PhysPageNum}]
d59: use:[super::{StepByOne, VPNRange}]
d60: use:[super::{frame_alloc, FrameTracker, PhysPageNum, PhysAddr, StepByOne, VirtAddr, VirtPageNum}]
d61: use:[super::{frame_alloc, FrameTracker}]
d62: use:[switch::__switch]
</dependency-pool>
<dependencies>
u0->refs:[d53]
u2->refs:[d17 d31 d51]
u3->refs:[d15 d32]
u6->refs:[d46]
u7->refs:[d10 d33 d16 d45 d41]
u9->refs:[d10]
u10->refs:[d0]
u11->refs:[d18 d9]
u12->refs:[d14 d30]
u14->refs:[d29 d38 d20 d39]
u15->refs:[d47 d44 d36]
u16->refs:[d35 d26]
u17->refs:[d45 d6 d19 d33 d58 d13]
u18->refs:[d10 d48 d6 d59 d61 d56 d33 d22 d45 d5 d4 d57]
u19->refs:[d18 d3 d9 d6]
u20->refs:[d6 d8 d20 d60 d7]
u21->refs:[d13 d54 d23]
u22->refs:[d2]
u23->refs:[d50]
u24->refs:[d11 d34 d24 d52 d37 d40 d49]
u25->refs:[d1]
u26->refs:[d10]
u27->refs:[d12]
u30->refs:[d42]
u31->refs:[d27 d55 d43 d21]
u32->refs:[d55]
u33->refs:[d25 d6 d62 d33 d28 d45 d41]
</dependencies>
<symbols>
main:u0 func entry=true
insert_app_data:u0 func
_start:u1 ld_entry
skernel:u1 ld_symbol
stext:u1 ld_symbol
strampoline:u1 ld_symbol
etext:u1 ld_symbol
srodata:u1 ld_symbol
erodata:u1 ld_symbol
sdata:u1 ld_symbol
edata:u1 ld_symbol
sbss_with_stack:u1 ld_symbol
sbss:u1 ld_symbol
ebss:u1 ld_symbol
ekernel:u1 ld_symbol
get_time:u2 func
get_time_ms:u2 func
get_time_us:u2 func
set_next_trigger:u2 func
panic:u3 func
get_num_app:u5 func
get_app_data:u5 func
enabled:u6 func
log:u6 func
flush:u6 func
init:u6 func
SimpleLogger:u6 struct
get_sp:u7 func
push_context:u7 func
get_sp:u7 func
print_app_info:u7 func
get_current_app:u7 func
move_to_next_app:u7 func
init:u7 func
print_app_info:u7 func
run_next_app:u7 func
KernelStack:u7 struct
UserStack:u7 struct
AppManager:u7 struct
sbi_call:u9 func
set_timer:u9 func
console_putchar:u9 func
shutdown:u9 func
_start:u10 label
boot_stack_lower_bound:u10 label
boot_stack_top:u10 label
init_heap:u11 func
handle_alloc_error:u11 func
write_str:u12 func
print:u12 func
Stdout:u12 struct
clear_bss:u13 func
kernel_log_info:u13 func
rust_main:u13 func
sys_exit:u14 func
sys_yield:u14 func
sys_get_time:u14 func
sys_trace:u14 func
sys_mmap:u14 func
sys_munmap:u14 func
sys_sbrk:u14 func
prot_to_perm:u14 func
TimeVal:u14 struct
sys_write:u16 func
new:u17 func
fmt:u17 func
drop:u17 func
init:u17 func
new:u17 func
alloc:u17 func
dealloc:u17 func
init_frame_allocator:u17 func
frame_alloc:u17 func
frame_dealloc:u17 func
frame_allocator_test:u17 func
FrameTracker:u17 struct
FrameAllocator:u17 trait
StackFrameAllocator:u17 struct
new_bare:u18 func
token:u18 func
insert_framed_area:u18 func
push:u18 func
map_trampoline:u18 func
new_kernel:u18 func
from_elf:u18 func
activate:u18 func
translate:u18 func
shrink_to:u18 func
append_to:u18 func
check_range:u18 func
mmap:u18 func
munmap:u18 func
new:u18 func
map_one:u18 func
unmap_one:u18 func
map:u18 func
unmap:u18 func
shrink_to:u18 func
append_to:u18 func
copy_data:u18 func
kernel_stack_position:u18 func
remap_test:u18 func
MemorySet:u18 struct
MapArea:u18 struct
MapType:u18 enum
MapPermission:u18 struct
handle_alloc_error:u19 func
init_heap:u19 func
heap_test:u19 func
new:u20 func
empty:u20 func
ppn:u20 func
flags:u20 func
is_valid:u20 func
readable:u20 func
writable:u20 func
executable:u20 func
new:u20 func
from_token:u20 func
find_pte_create:u20 func
find_pte:u20 func
map:u20 func
unmap:u20 func
translate:u20 func
token:u20 func
translated_byte_buffer:u20 func
translated_refmut:u20 func
check_permission:u20 func
PTEFlags:u20 struct
PageTableEntry:u20 struct
PageTable:u20 struct
fmt:u21 func
fmt:u21 func
fmt:u21 func
fmt:u21 func
from:u21 func
from:u21 func
from:u21 func
from:u21 func
from:u21 func
from:u21 func
from:u21 func
from:u21 func
floor:u21 func
ceil:u21 func
page_offset:u21 func
aligned:u21 func
from:u21 func
from:u21 func
floor:u21 func
ceil:u21 func
page_offset:u21 func
aligned:u21 func
from:u21 func
from:u21 func
get_mut:u21 func
get_pte_array:u21 func
get_bytes_array:u21 func
get_mut:u21 func
step:u21 func
new:u21 func
get_start:u21 func
get_end:u21 func
into_iter:u21 func
new:u21 func
next:u21 func
PhysAddr:u21 struct
VirtAddr:u21 struct
PhysPageNum:u21 struct
VirtPageNum:u21 struct
StepByOne:u21 trait
SimpleRange:u21 struct
SimpleRangeIterator:u21 struct
init:u22 func
set_sp:u23 func
app_init_context:u23 func
TrapContext:u23 struct
init:u24 func
set_kernel_trap_entry:u24 func
set_user_trap_entry:u24 func
enable_timer_interrupt:u24 func
trap_handler:u24 func
trap_return:u24 func
trap_from_kernel:u24 func
__alltraps:u25 label
__restore:u25 label
__emergency:u25 label
__emergency_end:u25 label
__trap_from_kernel:u25 label
exit:u26 func
exit_success:u26 func
exit_failure:u26 func
QEMUExit:u26 trait
RISCV64:u26 struct
exclusive_access:u27 func
UPSafeCell:u27 struct
__switch:u29 label
zero_init:u30 func
goto_trap_return:u30 func
TaskContext:u30 struct
get_trap_cx:u31 func
get_user_token:u31 func
new:u31 func
change_program_brk:u31 func
TaskControlBlock:u31 struct
TaskStatus:u31 enum
run_first_task:u33 func
mark_current_suspended:u33 func
mark_current_exited:u33 func
find_next_task:u33 func
get_current_token:u33 func
get_current_trap_cx:u33 func
change_current_program_brk:u33 func
run_next_task:u33 func
run_first_task:u33 func
run_next_task:u33 func
mark_current_suspended:u33 func
mark_current_exited:u33 func
suspend_current_and_run_next:u33 func
exit_current_and_run_next:u33 func
current_user_token:u33 func
current_trap_cx:u33 func
change_program_brk:u33 func
current_mmap:u33 func
current_munmap:u33 func
get_syscall_count:u33 func
increment_syscall_count:u33 func
TaskManager:u33 struct
TaskManagerInner:u33 struct
</symbols>
</pir>