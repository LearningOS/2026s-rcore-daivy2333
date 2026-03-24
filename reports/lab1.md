## 简单总结

本次实验实现了 `sys_trace` 系统调用，用于追踪当前任务系统调用的历史信息。主要修改包括：
1. 在 `TaskControlBlock` 中添加了系统调用计数器数组
2. 在 `TaskManager` 中添加了访问和修改计数器的方法
3. 在 `syscall` 函数中添加了计数逻辑
4. 实现了 `sys_trace` 的三种功能：读取用户态内存、写入用户态内存、查询系统调用次数
不会真有人看吧，祝你天天开心
---

## trap.S 问题回答

### L40：刚进入 __restore 时，sp 代表了什么值？请指出 __restore 的两种使用情景。

**sp 的值**：刚进入 `__restore` 时，`sp` 指向内核栈上已分配的 `TrapContext` 结构的起始地址。

**__restore 的两种使用情景**：
1. **从 trap_handler 返回**：当 trap 处理完成后（如系统调用处理完毕、时钟中断处理完毕），通过 `__restore` 恢复用户态上下文继续执行。
2. **启动第一个应用程序**：在 `run_first_task` 中，通过 `TaskContext::goto_restore` 构造初始上下文，最终跳转到 `__restore` 启动第一个用户程序。

### L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的值对于进入用户态有何意义？

```assembly
ld t0, 32*8(sp)    # 加载 sstatus
ld t1, 33*8(sp)    # 加载 sepc
ld t2, 2*8(sp)     # 加载用户栈指针
csrw sstatus, t0   # 恢复 sstatus
csrw sepc, t1      # 恢复 sepc
csrw sscratch, t2  # 恢复 sscratch
```

**处理的寄存器及其意义**：

| 寄存器 | 意义 |
|--------|------|
| `sstatus` | 处理器状态寄存器。其中 SPP 位决定了 `sret` 后的特权级（SPP=0 返回 U 态，SPP=1 返回 S 态），SPIE 位决定了返回后是否启用中断。 |
| `sepc` | 异常程序计数器，保存了发生异常时的指令地址。`sret` 会跳转到该地址继续执行。对于系统调用，`sepc` 已被加 4，指向下一条指令。 |
| `sscratch` | 暂存寄存器，在这里保存用户栈指针。在用户态运行时，`sscratch` 保存内核栈指针，用于 trap 发生时切换栈。 |
另外t0 t1 t2临时寄存器，
### L50-L56：为何跳过了 x2 和 x4？

```assembly
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
   LOAD_GP %n
   .set n, n+1
.endr
```

**跳过的寄存器及原因**：

| 寄存器 | 名称 | 跳过原因 |
|--------|------|----------|
| x2 | sp (stack pointer) | 栈指针不能直接恢复，需要在最后通过 `csrrw sp, sscratch, sp` 指令与 `sscratch` 交换来恢复用户栈指针。 |
| x4 | tp (thread pointer) | 线程指针寄存器，用于线程本地存储（TLS）。在当前的简单实现中，应用程序不使用该寄存器，因此无需恢复。 |

### L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？

```assembly
csrrw sp, sscratch, sp
```

**执行后**：
- `sp`：用户栈指针，指向用户程序的栈顶
- `sscratch`：内核栈指针，指向内核栈，下次 trap 发生时用于切换到内核栈

### __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

**状态切换指令**：`sret`（第 61 行）

**为何进入用户态**：
1. `sret` 指令会根据 `sstatus.SPP`（Supervisor Previous Privilege）位的值来决定返回后的特权级
2. 在 `app_init_context` 中，`sstatus` 被初始化为 `Sstatus::from_bits(1 << 8)`，即 SPP=0
3. SPP=0 表示返回到 U 态（User mode）
4. 同时 `sret` 会将 PC 设置为 `sepc` 的值，跳转到用户程序继续执行

### L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？

```assembly
csrrw sp, sscratch, sp
```

**执行后**：
- `sp`：内核栈指针，用于在内核态保存 trap 上下文
- `sscratch`：用户栈指针，保存在这里以便 trap 处理完成后恢复

### 从 U 态进入 S 态是哪一条指令发生的？

**进入 S 态的指令**：`ecall`（环境调用指令）

**触发过程**：
1. 用户程序执行 `ecall` 指令
2. CPU 触发 "Environment call from U-mode" 异常
3. 硬件自动完成：
   - 将当前特权级保存到 `sstatus.SPP`
   - 将 `sepc` 设置为 `ecall` 指令的地址
   - 将 `stvec` 的值写入 `pc`
   - 切换到 S 态
4. 跳转到 `__alltraps` 开始执行 trap 处理代码

---

## 荣誉准则
在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    gpt

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    rCore-Tutorial-Book v3 写的真不错，这里点赞
    rCore-Tutorial-Code
    RISC-V Privileged Architecture
    mit6004 这个最好玩
    mit6.S081 这个也不错，学理论好用

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按"-100"分计。
---