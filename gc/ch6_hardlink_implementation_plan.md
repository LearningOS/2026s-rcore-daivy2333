# Ch6 硬链接实现计划

## 背景

当前 ch6 分支已经有了 easy-fs 文件系统，但需要实现以下三个系统调用来支持硬链接：

1. `sys_linkat` - 创建硬链接
2. `sys_unlinkat` - 删除硬链接/文件
3. `sys_fstat` - 获取文件状态

## 硬链接原理

硬链接要求两个不同的目录项指向同一个文件（同一个 inode）。在 easy-fs 中需要：

1. **nlink 字段**：在 DiskInode 中添加链接计数
2. **link 操作**：在目录中创建新的目录项指向已存在的 inode
3. **unlink 操作**：删除目录项，当 nlink 为 0 时才真正删除文件

## 需要修改的文件

### 1. easy-fs/src/layout.rs

修改 `DiskInode` 结构体，添加 `nlink` 字段：

```rust
#[repr(C)]
pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect1: u32,
    pub indirect2: u32,
    type_: DiskInodeType,
    pub nlink: u32,  // 新增：硬链接计数
}
```

修改 `initialize` 方法，初始化 `nlink = 1`。

### 2. easy-fs/src/vfs.rs

添加以下方法到 `Inode`：

```rust
/// 创建硬链接
pub fn link(&self, old_name: &str, new_name: &str) -> isize;

/// 删除目录项
pub fn unlink(&self, name: &str) -> isize;

/// 获取 inode 信息
pub fn get_stat(&self) -> Option<Stat>;
```

### 3. os/src/fs/inode.rs

添加对硬链接的支持，可能需要修改 `OSInode` 来存储 inode ID。

### 4. os/src/syscall/fs.rs

实现三个系统调用：

```rust
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize;
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize;
pub fn sys_unlinkat(name: *const u8) -> isize;
```

### 5. os/src/syscall/mod.rs

添加系统调用的分发。

## 实现步骤

### 步骤 1：修改 DiskInode 结构体

在 `easy-fs/src/layout.rs` 中：
- 添加 `nlink` 字段
- 修改 `initialize` 方法设置 `nlink = 1`

### 步骤 2：修改 DirEntry

确保 `DirEntry` 可以获取和设置 inode ID。

### 步骤 3：实现 Inode 的 link 方法

在 `easy-fs/src/vfs.rs` 中：
1. 查找 old_name 对应的 inode
2. 在目录中创建新的目录项指向同一个 inode
3. 增加 nlink 计数

### 步骤 4：实现 Inode 的 unlink 方法

在 `easy-fs/src/vfs.rs` 中：
1. 查找 name 对应的 inode
2. 从目录中删除该目录项
3. 减少 nlink 计数
4. 如果 nlink == 0，回收 inode 和数据块

### 步骤 5：实现 sys_fstat

在 `os/src/syscall/fs.rs` 中：
1. 根据 fd 获取文件
2. 获取 inode 信息
3. 填充 Stat 结构体

### 步骤 6：实现 sys_linkat

在 `os/src/syscall/fs.rs` 中：
1. 解析 old_name 和 new_name
2. 调用 ROOT_INODE.link()

### 步骤 7：实现 sys_unlinkat

在 `os/src/syscall/fs.rs` 中：
1. 解析 name
2. 调用 ROOT_INODE.unlink()

## 注意事项

1. **磁盘格式兼容性**：添加 nlink 字段会改变 DiskInode 的大小，可能需要重新创建文件系统镜像
2. **并发安全**：需要正确处理多进程并发访问
3. **错误处理**：需要正确处理文件不存在、同名文件等情况

## 测试

使用 `make run BASE=2` 运行测试，确保通过所有测例。

## 关于 Git Cherry-pick

**不需要从 ch5 cherry-pick 代码**。ch5 分支没有文件系统代码，ch6 已经有了完整的 easy-fs 文件系统。你需要的是在 ch6 分支上实现硬链接功能。

如果你需要参考其他实现，可以：
1. 查看 rCore-Tutorial 的官方实现
2. 查看同学的实现
3. 参考 Linux 的硬链接实现原理