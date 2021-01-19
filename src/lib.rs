//! Create anonymous POSIX shared memory objects.
//!
//! This crate is only works on `unix` targets and is `no_std` compatible.
#![cfg(unix)]
#![no_std]
// Inspired by https://github.com/lassik/shm_open_anon (ISC license, Copyright 2019 Lassi Kortela)

use libc::c_int;

#[cfg(not(any(target_os = "freebsd", target_os = "android")))]
fn errno() -> c_int {
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    use libc::___errno as errno_location;
    #[cfg(any(target_os = "android", target_os = "netbsd", target_os = "openbsd"))]
    use libc::__errno as errno_location;
    #[cfg(any(target_os = "linux", target_os = "redox", target_os = "dragonfly"))]
    use libc::__errno_location as errno_location;
    #[cfg(any(target_os = "freebsd", target_os = "ios", target_os = "macos"))]
    use libc::__error as errno_location;

    unsafe { *errno_location() as c_int }
}

#[cfg(not(any(target_os = "freebsd", target_os = "android")))]
fn shm_open_anonymous_posix() -> c_int {
    use libc::c_char;

    let mut filename = *b"/shm_open_anonymous-XXXX\0";
    const OFFSET: usize = 20;
    assert_eq!(&filename[OFFSET..], b"XXXX\0");

    for i in (0..10000u16).cycle() {
        let path = {
            // replace the last four characters with the value `i`
            let digits = [
                b'0' + (i / 1000) as u8,
                b'0' + (i / 100 % 10) as u8,
                b'0' + (i / 10 % 10) as u8,
                b'0' + (i % 10) as u8,
            ];
            debug_assert!(digits.iter().all(|x| *x >= b'0' && *x <= b'9'));
            filename[OFFSET..OFFSET + 4].copy_from_slice(&digits);
            filename.as_ptr() as *const c_char
        };

        debug_assert!(filename.starts_with(b"/shm_open_anonymous-"));
        debug_assert!(filename.ends_with(b"\0"));

        // Try creating shared memory with the provided path.
        // If creation fails with EEXIST, try another filename until it works.

        // Safety: path points to a null-terminated string
        let fd = unsafe {
            libc::shm_open(
                path,
                libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW,
                0o600,
            )
        };
        if fd == -1 && errno() != libc::EEXIST {
            return -1;
        } else if fd != -1 {
            // Safety: path points to a null-terminated string and fd is valid
            unsafe {
                if libc::shm_unlink(path) == -1 {
                    libc::close(fd);
                    return -1;
                }
            }
            return fd;
        }
    }
    core::unreachable!()
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn memfd_create() -> c_int {
    static PATH: &'static str = "shm_open_anonymous\0";
    // PATH is a valid string
    let fd = unsafe {
        libc::syscall(
            libc::SYS_memfd_create,
            PATH.as_ptr() as *const libc::c_char,
            libc::MFD_CLOEXEC,
        )
    };
    fd as c_int
}

/// Creates an anonymous POSIX shared memory object.
///
/// On success, returns a new file descriptor as if by `shm_open`.
/// The file descriptor is unnamed and cannot be unlinked.
///
/// On failure, returns -1 and sets `errno`.
///
/// Depending on operating system, this function may use an OS-specific system call for creating
/// the memory object, or it may use a generic POSIX implementation.
pub fn shm_open_anonymous() -> c_int {
    #[cfg(target_os = "linux")]
    {
        // Try opening with memfd_create.
        // If that fails (because of an older kernel) use the generic POSIX method.
        let fd = memfd_create();
        if fd == -1 {
            if errno() == libc::ENOSYS {
                shm_open_anonymous_posix()
            } else {
                -1
            }
        } else {
            fd
        }
    }

    #[cfg(target_os = "android")]
    {
        memfd_create()
    }

    #[cfg(target_os = "freebsd")]
    {
        // no invariants to uphold
        unsafe { libc::shm_open(libc::SHM_ANON, libc::O_RDWR, 0) }
    }

    #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "freebsd")))]
    shm_open_anonymous_posix()
}

#[cfg(test)]
mod test {
    #[test]
    fn shm_open_anonymous() {
        let fd = super::shm_open_anonymous();
        assert!(fd != -1);
        assert!(unsafe { libc::close(fd) } != -1);
    }
}
