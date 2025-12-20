//! Foreign Function Interface (FFI) bindings to LibRaw C API
//!
//! This module provides safe Rust bindings to the LibRaw C library functions.
//! LibRaw is a library for reading RAW files from digital cameras.
//!
//! The bindings include:
//! - Core initialization and cleanup functions
//! - File opening and processing operations
//! - Thumbnail extraction functionality
//! - Memory management helpers
//! - Error code constants and utilities

use libc::{c_char, c_int, c_ushort, c_uchar};

// Windows 平台需要使用宽字符 API
#[cfg(windows)]
use std::os::raw::c_ushort as wchar_t;

// libraw_data_t 是一个不透明指针类型，用于表示 LibRaw 数据结构
// 我们使用空枚举来创建类型安全的指针，而不暴露内部结构
pub enum libraw_data_t {}

// LibRaw 处理后的图像数据结构
// 这个结构体表示 LibRaw 解码后的图像数据，包括缩略图和完整图像
#[repr(C)]  // 确保 C 内存布局兼容性
pub struct libraw_processed_image_t {
    /// 图像格式类型 (JPEG = 1, Bitmap = 2)
    pub image_type: c_int,
    /// 图像高度（像素）
    pub height: c_ushort,
    /// 图像宽度（像素）
    pub width: c_ushort,
    /// 颜色通道数
    pub colors: c_ushort,
    /// 每像素位数
    pub bits: c_ushort,
    /// 图像数据大小（字节）
    pub data_size: u32,
    /// 图像数据（柔性数组成员，实际大小由 data_size 决定）
    /// 注意：在 Rust 中我们使用长度为 1 的数组来表示柔性数组成员
    pub data: [c_uchar; 1],
}

// 链接配置：根据目标平台选择不同的库名称
// Windows MSVC 使用 libraw_static 静态库
#[cfg_attr(all(target_os = "windows", target_env = "msvc"), link(name = "libraw_static", kind = "static"))]
// 其他平台使用 raw 静态库（如果可用）或动态库
#[cfg_attr(not(all(target_os = "windows", target_env = "msvc")), link(name = "raw", kind = "static"))]
extern "C" {
    // === 库版本信息 ===
    /// 获取 LibRaw 版本字符串
    pub fn libraw_version() -> *const c_char;

    /// 获取 LibRaw 版本号（整数格式）
    pub fn libraw_versionNumber() -> c_int;

    // === 构造函数和析构函数 ===
    /// 初始化 LibRaw 实例
    /// flags: 初始化标志，通常使用 LIBRAW_OPTIONS_NONE
    /// 返回: 指向 libraw_data_t 的指针，失败时返回 NULL
    pub fn libraw_init(flags: c_int) -> *mut libraw_data_t;

    /// 关闭 LibRaw 实例并释放所有资源
    pub fn libraw_close(data: *mut libraw_data_t);

    // === 文件操作 ===
    /// 打开 RAW 文件
    /// data: LibRaw 实例指针
    /// filename: 文件名（UTF-8 字符串）
    /// 返回: LIBRAW_SUCCESS 表示成功，其他值表示错误
    pub fn libraw_open_file(data: *mut libraw_data_t, filename: *const c_char) -> c_int;

    /// Windows 平台：打开宽字符文件名
    #[cfg(windows)]
    pub fn libraw_open_wfile(data: *mut libraw_data_t, filename: *const wchar_t) -> c_int;

    /// 解包 RAW 文件数据（解析文件头和基本信息）
    pub fn libraw_unpack(data: *mut libraw_data_t) -> c_int;

    /// 处理 RAW 数据（去马赛克、色彩转换等）
    pub fn libraw_dcraw_process(data: *mut libraw_data_t) -> c_int;

    // === 缩略图操作 ===
    /// 解包缩略图数据
    pub fn libraw_unpack_thumb(data: *mut libraw_data_t) -> c_int;

    /// 从缩略图数据创建内存中的图像
    /// errc: 输出参数，接收错误代码
    /// 返回: 指向处理后图像的指针，失败时返回 NULL
    pub fn libraw_dcraw_make_mem_thumb(data: *mut libraw_data_t, errc: *mut c_int) -> *mut libraw_processed_image_t;

    /// 释放由 libraw_dcraw_make_mem_* 分配的内存
    pub fn libraw_dcraw_clear_mem(img: *mut libraw_processed_image_t);

    // === 错误处理 ===
    /// 获取错误代码的描述字符串
    pub fn libraw_strerror(error_code: c_int) -> *const c_char;

    // === 内存管理 ===
    /// 回收 LibRaw 实例的数据流，准备处理新文件
    /// 这比 libraw_close 更轻量级，不会释放所有内存
    pub fn libraw_recycle(data: *mut libraw_data_t);
}

// === LibRaw 初始化标志常量 ===
/// 无特殊选项
pub const LIBRAW_OPTIONS_NONE: c_int = 0;

/// 禁用内存错误回调
pub const LIBRAW_OPIONS_NO_MEMERR_CALLBACK: c_int = 1;

/// 禁用数据错误回调
pub const LIBRAW_OPIONS_NO_DATAERR_CALLBACK: c_int = 1 << 1;

// === LibRaw 返回代码常量 ===
/// 操作成功
pub const LIBRAW_SUCCESS: c_int = 0;

/// 未指定错误
pub const LIBRAW_UNSPECIFIED_ERROR: c_int = -1;

/// 不支持的文件格式
pub const LIBRAW_FILE_UNSUPPORTED: c_int = -2;

/// 请求不存在的图像
pub const LIBRAW_REQUEST_FOR_NONEXISTENT_IMAGE: c_int = -3;

/// 函数调用顺序错误
pub const LIBRAW_OUT_OF_ORDER_CALL: c_int = -4;

/// 没有缩略图
pub const LIBRAW_NO_THUMBNAIL: c_int = -5;

/// 不支持的缩略图格式
pub const LIBRAW_UNSUPPORTED_THUMBNAIL: c_int = -6;

/// 输入已关闭
pub const LIBRAW_INPUT_CLOSED: c_int = -7;

/// 内存不足
pub const LIBRAW_INSUFFICIENT_MEMORY: c_int = -100;

/// 数据错误（损坏的文件）
pub const LIBRAW_DATA_ERROR: c_int = -101;

/// I/O 错误（读写失败）
pub const LIBRAW_IO_ERROR: c_int = -102;

/// 操作被回调取消
pub const LIBRAW_CANCELLED_BY_CALLBACK: c_int = -103;

/// 错误的裁剪参数
pub const LIBRAW_BAD_CROP: c_int = -104;

/// 图像太大
pub const LIBRAW_TOO_BIG: c_int = -105;

/// 内存池溢出
pub const LIBRAW_MEMPOOL_OVERFLOW: c_int = -106;

// === 图像格式常量 ===
/// JPEG 格式图像
pub const LIBRAW_IMAGE_JPEG: c_int = 1;

/// 位图格式图像（未压缩的 RGB 数据）
pub const LIBRAW_IMAGE_BITMAP: c_int = 2;
