## 简单总结

本次实验实现了进程创建的新方式 `spawn` 系统调用以及带优先级的 `stride` 调度算法。主要修改包括：
1. 实现了 `sys_spawn` 系统调用，可以直接创建新进程并执行指定程序，无需像 fork 那样复制父进程地址空间
2. 在 TCB 中添加了 `priority` 和 `stride` 字段，实现了 stride 调度算法
3. 实现了 `sys_set_priority` 系统调用，允许进程动态调整自身优先级
spawn 确实比 fork+exec 更直观，stride 调度也很有意思

---

## 问答作业

### 1. stride 算法溢出问题

**问题描述**：例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride，p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。实际情况是轮到 p1 执行吗？为什么？

**回答**：

实际情况**不一定**是 p1 执行。

当 p2 执行一个时间片后，p2.stride = 250 + 10 = 260。但由于使用 8bit 无符号整型存储，260 会溢出变成 260 - 256 = 4。

此时：
- p1.stride = 255
- p2.stride = 4

如果直接比较，4 < 255，会错误地认为 p2 的 stride 更小，继续选择 p2 执行，这显然是错误的。

这就是 stride 算法的**溢出问题**：当 stride 值超出存储范围后，直接比较会得到错误的结果。

---

### 2. STRIDE_MAX – STRIDE_MIN <= BigStride / 2 的说明

**前提**：所有进程优先级 >= 2，不考虑溢出。

**说明**：

1. 设 BigStride 为常量 B，进程优先级为 p，则 pass = B / p

2. 由于 p >= 2，所以 pass <= B / 2

3. 假设当前 STRIDE_MAX - STRIDE_MIN > B / 2

4. 考虑 stride 最小的进程（STRIDE_MIN），它被调度后 stride 增加 pass：
   - 新 stride = STRIDE_MIN + pass <= STRIDE_MIN + B/2

5. 由于 STRIDE_MAX - STRIDE_MIN > B/2，即 STRIDE_MAX > STRIDE_MIN + B/2

6. 所以新 stride 仍然小于 STRIDE_MAX，该进程仍然可能是 stride 最小的

7. 这意味着 stride 最大值和最小值的差距不会继续扩大

8. 实际上，由于每次调度都选择 stride 最小的进程，stride 值会趋于"聚集"，最大差距不会超过最大的 pass 值，即 B/2

因此，在优先级 >= 2 的条件下，STRIDE_MAX – STRIDE_MIN <= BigStride / 2 成立。

---

### 3. Stride 比较器实现

已知 STRIDE_MAX – STRIDE_MIN <= BigStride / 2，可以利用这个性质设计比较器：

当两个 stride 的差值超过 BigStride / 2 时，说明发生了溢出，较小的值实际上是较大的。

```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 假设 BigStride = 1 << 32 (示例值)
        const BIG_STRIDE: u64 = 1 << 32;
        const HALF_BIG_STRIDE: u64 = BIG_STRIDE / 2;
        
        // 计算差值，考虑溢出
        let diff = self.0.wrapping_sub(other.0);
        
        if diff == 0 {
            // 题目说明两个 Stride 永远不会相等，但为了完整性保留
            Some(Ordering::Equal)
        } else if diff < HALF_BIG_STRIDE {
            // 正常情况：self > other
            Some(Ordering::Greater)
        } else {
            // 溢出情况：self < other（因为差值超过了半程，说明 other "绕了一圈"）
            Some(Ordering::Less)
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false  // 题目说明两个 Stride 永远不会相等
    }
}
```

**原理解释**：

假设使用 8 bits 存储 stride，BigStride = 255：

| self | other | wrapping_sub | 结果 | 解释 |
|------|-------|--------------|------|------|
| 125 | 255 | 125 - 255 = -130 → 126 | 126 < 127.5 | self < other（正确） |
| 129 | 255 | 129 - 255 = -126 → 130 | 130 > 127.5 | self > other（正确，因为 129 实际上是溢出后的值，比 255 小） |

当 `wrapping_sub` 的结果小于 `BIG_STRIDE / 2` 时，说明 self 确实比 other 大（没有溢出）；反之，说明发生了溢出，self 实际上比 other 小。

---

## 荣誉准则

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    gpt

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    rCore-Tutorial-Book v3
    rCore-Tutorial-Code
    RISC-V Privileged Architecture

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按"-100"分计。

---