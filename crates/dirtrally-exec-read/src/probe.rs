//! Test probe for the exec-read library: maps one page with `PROT_EXEC`
//! only, the way DiRT Rally's Linux port does, and reads it back. On a CPU
//! with memory protection keys the read faults unless the library is
//! preloaded. Exit 0: the read worked.

#[cfg(target_os = "linux")]
fn main() {
    // SAFETY: a fresh anonymous private page, unmapped before exit; the
    // read is the behaviour under test.
    unsafe {
        let page = libc::mmap(
            std::ptr::null_mut(),
            4096,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        );
        assert_ne!(page, libc::MAP_FAILED, "mmap failed");
        *(page as *mut u8) = 0xc3;
        assert_eq!(
            libc::mprotect(page, 4096, libc::PROT_EXEC),
            0,
            "mprotect failed"
        );
        let byte = std::ptr::read_volatile(page as *const u8);
        libc::munmap(page, 4096);
        std::process::exit(if byte == 0xc3 { 0 } else { 1 });
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {}
