//! Makes executable memory readable for DiRT Rally's Linux port.
//!
//! The 2017 Feral build maps memory with `PROT_EXEC` alone and then reads
//! it. Old kernels made such memory readable anyway. On CPUs with memory
//! protection keys (PKU: AMD Zen 3 and later, Intel since Skylake-SP), Linux
//! makes `PROT_EXEC`-only memory really execute-only, so that read faults
//! (`SEGV_PKUERR`) and the game crashes right after "SDL2 initialised"
//! (docs/RESEARCH.md #5, docs/RUNTIME_NOTES.md).
//!
//! Preloaded into the game (`LD_PRELOAD`), this library replaces `mmap`,
//! `mmap64` and `mprotect`: a request that includes `PROT_EXEC` also gets
//! `PROT_READ`, which is what those older kernels did. Nothing else changes.
//! The calls go straight to the kernel, so there is no lookup of the next
//! definition and no recursion. The game's files are not touched.

/// The protection the kernel is asked for: `prot`, plus read access when
/// it asks for execute access.
pub fn with_read(prot: i32) -> i32 {
    #[cfg(target_os = "linux")]
    {
        if prot & libc::PROT_EXEC != 0 {
            return prot | libc::PROT_READ;
        }
    }
    prot
}

#[cfg(target_os = "linux")]
mod hooks {
    use super::with_read;
    use libc::{c_int, c_long, c_void, off_t, size_t};

    /// # Safety
    /// Same contract as libc's `mmap`.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn mmap(
        addr: *mut c_void,
        len: size_t,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: off_t,
    ) -> *mut c_void {
        // SAFETY: the caller's arguments, passed through; only `prot` gains
        // PROT_READ. On failure the raw call returns -1 and sets errno,
        // which is MAP_FAILED.
        unsafe {
            libc::syscall(
                libc::SYS_mmap,
                addr,
                len,
                with_read(prot) as c_long,
                flags as c_long,
                fd as c_long,
                offset as c_long,
            ) as *mut c_void
        }
    }

    /// # Safety
    /// Same contract as libc's `mmap64` (the same call on 64-bit Linux).
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn mmap64(
        addr: *mut c_void,
        len: size_t,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: off_t,
    ) -> *mut c_void {
        // SAFETY: as `mmap`.
        unsafe { mmap(addr, len, prot, flags, fd, offset) }
    }

    /// # Safety
    /// Same contract as libc's `mprotect`.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn mprotect(addr: *mut c_void, len: size_t, prot: c_int) -> c_int {
        // SAFETY: the caller's arguments, passed through; only `prot` gains
        // PROT_READ.
        unsafe { libc::syscall(libc::SYS_mprotect, addr, len, with_read(prot) as c_long) as c_int }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::with_read;

    #[test]
    fn exec_gains_read_and_nothing_else_changes() {
        use libc::{PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE};
        assert_eq!(with_read(PROT_EXEC), PROT_EXEC | PROT_READ);
        assert_eq!(
            with_read(PROT_EXEC | PROT_WRITE),
            PROT_EXEC | PROT_WRITE | PROT_READ
        );
        assert_eq!(with_read(PROT_EXEC | PROT_READ), PROT_EXEC | PROT_READ);
        for prot in [PROT_NONE, PROT_READ, PROT_WRITE, PROT_READ | PROT_WRITE] {
            assert_eq!(with_read(prot), prot);
        }
    }
}
