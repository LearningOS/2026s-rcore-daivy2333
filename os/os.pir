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
u17: src/trap/context.rs type=Rust role=lib module=trap
u18: src/trap/mod.rs type=Rust role=lib module=trap
u19: src/trap/trap.S type=S role=lib module=trap
u20: src/boards/qemu.rs type=Rust role=lib module=boards
u21: src/sync/up.rs type=Rust role=lib module=sync
u22: src/sync/mod.rs type=Rust role=lib module=sync
u23: src/task/switch.S type=S role=lib module=task
u24: src/task/context.rs type=Rust role=lib module=task
u25: src/task/task.rs type=Rust role=lib module=task
u26: src/task/switch.rs type=Rust role=lib module=task
u27: src/task/mod.rs type=Rust role=lib module=task
</units>
<dependency-pool>
d0: call:u13#rust_main
d1: call:u18#trap_handler
d2: use:[buddy_system_allocator::LockedHeap]
d3: use:[core::arch::asm]
d4: use:[core::arch::global_asm]
d5: use:[core::cell::{RefCell, RefMut}]
d6: use:[core::fmt::{self, Write}]
d7: use:[core::panic::PanicInfo]
d8: use:[crate::board::QEMUExit]
d9: use:[crate::config::*]
d10: use:[crate::config::CLOCK_FREQ]
d11: use:[crate::config::KERNEL_HEAP_SIZE]
d12: use:[crate::config::MAX_APP_NUM]
d13: use:[crate::loader::{get_num_app, init_app_cx}]
d14: use:[crate::sbi::console_putchar]
d15: use:[crate::sbi::set_timer]
d16: use:[crate::sbi::shutdown]
d17: use:[crate::sync::UPSafeCell]
d18: use:[crate::syscall::syscall]
d19: use:[crate::task::{exit_current_and_run_next, suspend_current_and_run_next}]
d20: use:[crate::timer::set_next_trigger]
d21: use:[crate::trap::TrapContext]
d22: use:[crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
}]
d23: use:[fs::*]
d24: use:[lazy_static::*]
d25: use:[log::{Level, LevelFilter, Log, Metadata, Record}]
d26: use:[process::*]
d27: use:[riscv::register::sstatus::{self, Sstatus, SPP}]
d28: use:[riscv::register::time]
d29: use:[riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
}]
d30: use:[stdlib:rust]
d31: use:[super::TaskContext]
d32: use:[switch::__switch]
</dependency-pool>
<dependencies>
u0->refs:[d30]
u2->refs:[d28 d10 d15]
u3->refs:[d16 d7]
u5->refs:[d3 d21 d9]
u6->refs:[d25]
u7->refs:[d17 d21 d24 d3 d8]
u9->refs:[d3]
u10->refs:[d0]
u11->refs:[d11 d2]
u12->refs:[d14 d6]
u14->refs:[d22]
u15->refs:[d26 d23]
u17->refs:[d27]
u18->refs:[d19 d18 d20 d4 d29]
u19->refs:[d1]
u20->refs:[d3]
u21->refs:[d5]
u25->refs:[d31]
u26->refs:[d4 d31]
u27->refs:[d17 d13 d32 d24 d12]
</dependencies>
<symbols>
main:u0 func entry=true
insert_app_data:u0 func
_start:u1 ld_entry
skernel:u1 ld_symbol
stext:u1 ld_symbol
etext:u1 ld_symbol
srodata:u1 ld_symbol
erodata:u1 ld_symbol
sdata:u1 ld_symbol
edata:u1 ld_symbol
sbss:u1 ld_symbol
ebss:u1 ld_symbol
ekernel:u1 ld_symbol
get_time:u2 func
get_time_ms:u2 func
get_time_us:u2 func
set_next_trigger:u2 func
panic:u3 func
get_sp:u5 func
push_context:u5 func
get_sp:u5 func
get_base_i:u5 func
get_num_app:u5 func
load_apps:u5 func
init_app_cx:u5 func
KernelStack:u5 struct
UserStack:u5 struct
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
_num_app:u8 label
app_0_start:u8 label
app_0_end:u8 label
app_1_start:u8 label
app_1_end:u8 label
app_2_start:u8 label
app_2_end:u8 label
app_3_start:u8 label
app_3_end:u8 label
app_4_start:u8 label
app_4_end:u8 label
app_5_start:u8 label
app_5_end:u8 label
app_6_start:u8 label
app_6_end:u8 label
app_7_start:u8 label
app_7_end:u8 label
app_8_start:u8 label
app_8_end:u8 label
app_9_start:u8 label
app_9_end:u8 label
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
TimeVal:u14 struct
sys_write:u16 func
set_sp:u17 func
app_init_context:u17 func
TrapContext:u17 struct
init:u18 func
enable_timer_interrupt:u18 func
trap_handler:u18 func
__alltraps:u19 label
__restore:u19 label
exit:u20 func
exit_success:u20 func
exit_failure:u20 func
QEMUExit:u20 trait
RISCV64:u20 struct
exclusive_access:u21 func
UPSafeCell:u21 struct
__switch:u23 label
zero_init:u24 func
goto_restore:u24 func
TaskContext:u24 struct
TaskControlBlock:u25 struct
TaskStatus:u25 enum
run_first_task:u27 func
mark_current_suspended:u27 func
mark_current_exited:u27 func
find_next_task:u27 func
run_next_task:u27 func
run_first_task:u27 func
run_next_task:u27 func
mark_current_suspended:u27 func
mark_current_exited:u27 func
suspend_current_and_run_next:u27 func
exit_current_and_run_next:u27 func
TaskManager:u27 struct
TaskManagerInner:u27 struct
</symbols>
<layout>
ENTRY=_start
SECTIONS=.bss,.data,.rodata,.text
SECTIONS_FROM=u1
</layout>
</pir>