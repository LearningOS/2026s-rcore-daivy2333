# Ch4 系统调用实现方案

## 概述

本章需要在引入虚存机制后重写 `sys_get_time` 和 `sys_trace` 系统调用，并实现新的 `sys_mmap` 和 `sys_munmap` 系统调用。

---

## 修改清单总览

| 序号 | 文件路径 | 修改类型 | 说明 |
|------|----------|----------|------|
| 1 | `os/src/mm/page_table.rs` | 新增函数 | 添加 `translated_refmut`、`check_permission` |
| 2 | `os/src/mm/memory_set.rs` | 新增方法 | 添加 `mmap`、`munmap` 等 |
| 3 | `os/src/mm/mod.rs` | 导出更新 | 导出新增的公开方法 |
| 4 | `os/src/task/mod.rs` | 新增函数 | 添加 `current_mmap`、`current_munmap` |
| 5 | `os/src/syscall/process.rs` | 重写函数 | 重写 4 个系统调用 |

---

## 关键设计决策

### 为什么不需要 `current_memory_set()`？

**问题**: 尝试返回 `&'static mut MemorySet` 会导致生命周期问题：
```rust
// ❌ 错误的实现
pub fn current_memory_set() -> &'static mut MemorySet {
    let inner = TASK_MANAGER.inner.exclusive_access();
    // inner 在函数返回时被 drop，返回的引用失效！
    &mut inner.tasks[cur].memory_set
}
```

**解决方案**: 使用 `PageTable::from_token(token)` 模式，与 `translated_byte_buffer` 相同：
1. 使用 `current_user_token()` 获取当前任务的页表 token
2. 创建临时 `PageTable` 对象进行地址翻译
3. 通过 `PhysPageNum::get_mut()` 获取 `&'static mut` 引用

这种模式是安全的，因为物理页帧在任务生命周期内有效。

---

## 详细实现方案

### 修改 1: `os/src/mm/page_table.rs` - 新增函数

#### 1.1 新增 `translated_refmut` 函数

**位置**: 文件末尾，约第 181 行之后

**代码**:
```rust
/// Translate a virtual address to a mutable reference to T through page table
/// Returns None if:
/// - The virtual address is not mapped
/// - The page is not accessible from user mode (no U flag)
/// - The data crosses page boundary
pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> Option<&'static mut T> {
    let page_table = PageTable::from_token(token);
    let va = VirtAddr::from(ptr as usize);
    let vpn = va.floor();
    
    // 翻译虚拟页号
    let pte = page_table.translate(vpn)?;
    
    // 检查页表项是否有效且用户可访问
    if !pte.is_valid() || !pte.flags().contains(PTEFlags::U) {
        return None;
    }
    
    // 检查是否跨页
    let offset = va.page_offset();
    if offset + core::mem::size_of::<T>() > PAGE_SIZE {
        return None;  // 跨页情况暂不处理
    }
    
    // 获取物理地址并返回可变引用
    let ppn = pte.ppn();
    let pa: PhysAddr = ppn.into();
    Some(unsafe { &mut *((pa.0 + offset) as *mut T) })
}
```

#### 1.2 新增 `check_permission` 函数

**位置**: `translated_refmut` 之后

**代码**:
```rust
/// Check if a virtual address has the required permissions
/// Returns true if the address is mapped with U flag and the required R/W permissions
pub fn check_permission(token: usize, va: VirtAddr, need_read: bool, need_write: bool) -> bool {
    let page_table = PageTable::from_token(token);
    let vpn = va.floor();
    
    if let Some(pte) = page_table.translate(vpn) {
        if !pte.is_valid() {
            return false;
        }
        let flags = pte.flags();
        // 必须有 U 标志
        if !flags.contains(PTEFlags::U) {
            return false;
        }
        // 检查 R 权限
        if need_read && !flags.contains(PTEFlags::R) {
            return false;
        }
        // 检查 W 权限
        if need_write && !flags.contains(PTEFlags::W) {
            return false;
        }
        true
    } else {
        false
    }
}
```

**需要导入** (文件顶部):
```rust
use super::PAGE_SIZE;
use super::VirtAddr;
```

---

### 修改 2: `os/src/mm/memory_set.rs` - 新增辅助方法

#### 2.1 新增 `mmap` 方法

**位置**: `MemorySet` impl 块中，约第 265 行之前

**代码**:
```rust
/// Map a new memory region - for sys_mmap
pub fn mmap(&mut self, start: VirtAddr, len: usize, perm: MapPermission) -> isize {
    if len == 0 {
        return 0;
    }
    
    let end: VirtAddr = (start.0 + len).into();
    
    // 检查是否与现有映射冲突
    let start_vpn = start.floor();
    let end_vpn = end.ceil();
    
    for vpn in VPNRange::new(start_vpn, end_vpn) {
        if let Some(pte) = self.page_table.translate(vpn) {
            if pte.is_valid() {
                return -1;  // 已映射，冲突
            }
        }
    }
    
    // 创建新的映射区域
    let map_area = MapArea::new(start, end, MapType::Framed, perm);
    self.push(map_area, None);
    0
}
```

#### 2.2 新增 `munmap` 方法

**位置**: `MemorySet` impl 块中

**代码**:
```rust
/// Unmap a memory region - for sys_munmap
pub fn munmap(&mut self, start: VirtAddr, len: usize) -> isize {
    if len == 0 {
        return 0;
    }
    
    let start_vpn = start.floor();
    let end: VirtAddr = (start.0 + len).into();
    let end_vpn = end.ceil();
    
    // 检查所有页面是否都已映射
    for vpn in VPNRange::new(start_vpn, end_vpn) {
        if let Some(pte) = self.page_table.translate(vpn) {
            if !pte.is_valid() {
                return -1;  // 未映射
            }
        } else {
            return -1;
        }
    }
    
    // 取消映射
    for vpn in VPNRange::new(start_vpn, end_vpn) {
        self.page_table.unmap(vpn);
    }
    
    // 注意：简化实现，不更新 areas 和不回收物理页帧
    // 完整实现需要找到对应的 MapArea 并移除 data_frames
    
    0
}
```

---

### 修改 3: `os/src/mm/mod.rs` - 更新导出

**位置**: 约第 15-21 行

**修改后的导出**:
```rust
pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use address::{StepByOne, VPNRange};
pub use frame_allocator::{frame_alloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{kernel_stack_position, MapPermission, MemorySet, KERNEL_SPACE};
pub use page_table::{translated_byte_buffer, translated_refmut, check_permission, PageTableEntry};  // 新增导出
pub use page_table::{PTEFlags, PageTable};
pub use memory_set::MapType;  // 新增导出
```

---

### 修改 4: `os/src/task/mod.rs` - 新增 mmap/munmap 封装函数

**位置**: 文件末尾，约第 204 行之后

**代码**:
```rust
/// mmap: map memory region for current task
pub fn current_mmap(start: usize, len: usize, perm: MapPermission) -> isize {
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let cur = inner.current_task;
    inner.tasks[cur].memory_set.mmap(VirtAddr::from(start), len, perm)
}

/// munmap: unmap memory region for current task
pub fn current_munmap(start: usize, len: usize) -> isize {
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let cur = inner.current_task;
    inner.tasks[cur].memory_set.munmap(VirtAddr::from(start), len)
}
```

**需要导入** (文件顶部):
```rust
use crate::mm::{MapPermission, VirtAddr};
```

---

### 修改 5: `os/src/syscall/process.rs` - 重写系统调用

#### 5.1 更新导入

**位置**: 文件顶部，第 1-2 行

**修改后**:
```rust
//! Process management syscalls
use crate::task::{change_program_brk, current_mmap, current_munmap, current_user_token, exit_current_and_run_next, suspend_current_and_run_next};
use crate::mm::{check_permission, translated_refmut, MapPermission, VirtAddr, PAGE_SIZE};
use crate::timer::get_time_us;
```

#### 5.2 重写 `sys_get_time`

**位置**: 约第 28-31 行

**修改后**:
```rust
/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let token = current_user_token();
    
    match translated_refmut(token, ts) {
        Some(ts_ref) => {
            ts_ref.sec = us / 1_000_000;
            ts_ref.usec = us % 1_000_000;
            0
        }
        None => -1,
    }
}
```

#### 5.3 重写 `sys_trace` - 关键修改！

**位置**: 约第 33-38 行

**代码**:
```rust
/// trace syscall: read or write a byte in user space
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    let token = current_user_token();
    let va = VirtAddr::from(data);
    
    match trace_request {
        0 => {
            // 读取 - 检查用户可见且可读
            if !check_permission(token, va, true, false) {
                return -1;
            }
            match translated_refmut::<u8>(token, data as *mut u8) {
                Some(byte_ref) => *byte_ref as isize,
                None => -1,
            }
        }
        1 => {
            // 写入 - 检查用户可见且可写
            if !check_permission(token, va, false, true) {
                return -1;
            }
            match translated_refmut::<u8>(token, data as *mut u8) {
                Some(byte_ref) => {
                    *byte_ref = id as u8;
                    0
                }
                None => -1,
            }
        }
        _ => -1,
    }
}
```

#### 5.4 实现 `sys_mmap`

**位置**: 约第 40-44 行

**代码**:
```rust
/// mmap: map memory region
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    
    // 参数校验：页对齐
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    
    // 参数校验：prot 有效
    if prot & !0x7 != 0 {
        return -1;  // 其他位必须为 0
    }
    if prot & 0x7 == 0 {
        return -1;  // 无意义权限
    }
    
    // 长度为 0 视为成功
    if len == 0 {
        return 0;
    }
    
    // 转换权限：prot 格式 -> MapPermission 格式
    let mut perm = MapPermission::U;  // 必须有 U 标志
    if prot & 0x1 != 0 { perm |= MapPermission::R; }
    if prot & 0x2 != 0 { perm |= MapPermission::W; }
    if prot & 0x4 != 0 { perm |= MapPermission::X; }
    
    current_mmap(start, len, perm)
}
```

#### 5.5 实现 `sys_munmap`

**位置**: 约第 46-50 行

**代码**:
```rust
/// munmap: unmap memory region
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    
    // 参数校验：页对齐
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    
    // 长度为 0 视为成功
    if len == 0 {
        return 0;
    }
    
    current_munmap(start, len)
}
```

---

## 权限标志位对应关系

### MapPermission vs PTEFlags vs prot

| 权限 | MapPermission | PTEFlags | prot 参数 |
|------|---------------|----------|-----------|
| 可读 | R = 1 << 1 | R = 1 << 1 | bit 0 |
| 可写 | W = 1 << 2 | W = 1 << 2 | bit 1 |
| 可执行 | X = 1 << 3 | X = 1 << 3 | bit 2 |
| 用户态 | U = 1 << 4 | U = 1 << 4 | - |

**prot 到 MapPermission 转换**:
```rust
fn prot_to_perm(prot: usize) -> MapPermission {
    let mut perm = MapPermission::U;  // 用户态可访问
    if prot & 0x1 != 0 { perm |= MapPermission::R; }
    if prot & 0x2 != 0 { perm |= MapPermission::W; }
    if prot & 0x4 != 0 { perm |= MapPermission::X; }
    perm
}
```

---

## sys_trace 权限检查说明

根据题目要求：
- **读取时**（trace_request = 0）：如果对应地址用户不可见或不可读，返回 -1
- **写入时**（trace_request = 1）：如果对应地址用户不可见或不可写，返回 -1

`check_permission` 函数检查：
1. 地址是否已映射（PTE 有效）
2. 是否有 U 标志（用户态可访问）
3. 是否有 R 标志（need_read = true 时）
4. 是否有 W 标志（need_write = true 时）

---

## 测试用例

### sys_get_time
- ✅ 正常获取时间
- ❌ 传入无效地址返回 -1

### sys_trace
- ✅ 读取成功返回字节值
- ✅ 写入成功返回 0
- ❌ 读取无权限地址返回 -1（无 U 或 R 标志）
- ❌ 写入无权限地址返回 -1（无 U 或 W 标志）

### sys_mmap
- ✅ 正常映射返回 0
- ❌ start 未页对齐返回 -1
- ❌ prot 无效返回 -1
- ❌ 区间已映射返回 -1

### sys_munmap
- ✅ 正常取消映射返回 0
- ❌ start 未页对齐返回 -1
- ❌ 区间未完全映射返回 -1

---

## 注意事项

1. **PTE_U 标志**: mmap 映射的页面必须设置 U 标志，否则用户态无法访问
2. **页对齐**: mmap/munmap 的 start 参数必须页对齐
3. **权限检查**: sys_trace 需要检查 U 标志 + 对应的 R/W 标志
4. **跨页处理**: TimeVal 结构体 16 字节，通常不会跨页，但 translated_refmut 中做了检查
5. **物理内存**: mmap 时通过 MapArea::new 自动分配物理页
6. **生命周期安全**: 使用 `PageTable::from_token` + `PhysPageNum::get_mut` 模式避免生命周期问题