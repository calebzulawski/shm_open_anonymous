//! Create anonymous POSIX shared memory objects.
#![cfg(unix)]
// Inspired by https://github.com/lassik/shm_open_anon (ISC license, Copyright 2019 Lassi Kortela)

use libc::c_int;
use std::ffi::CStr;
use std::os::unix::io::RawFd;

#[cfg(any(target_os = "android", target_os = "netbsd", target_os = "openbsd"))]
use libc::__errno as errno_location;
#[cfg(any(target_os = "linux", target_os = "redox", target_os = "dragonfly"))]
use libc::__errno_location as errno_location;
#[cfg(any(target_os = "freebsd", target_os = "ios", target_os = "macos"))]
use libc::__error as errno_location;

fn errno() -> c_int {
    unsafe { *errno_location() as c_int }
}

fn shm_unlink(path: &CStr) -> c_int {
    // CStr ensures `path` is safe.
    unsafe { libc::shm_unlink(path.as_ptr()) }
}

#[cfg(not(target_os = "freebsd"))]
fn shm_open_anonymous_posix() -> RawFd {
    fn shm_open_path(path: &CStr) -> c_int {
        // CStr ensures `path` is safe.
        unsafe {
            libc::shm_open(
                path.as_ptr(),
                libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW,
                0o600,
            )
        }
    }

    fn pseudorandom() -> Option<i64> {
        // `time` is initialized if clock_gettime doesn't return -1
        unsafe {
            let mut time: std::mem::MaybeUninit<libc::timespec> = std::mem::MaybeUninit::uninit();
            if libc::clock_gettime(libc::CLOCK_REALTIME, time.as_mut_ptr()) == -1 {
                return None;
            }
            Some(time.assume_init().tv_nsec)
        }
    }

    let mut filename = *b"/shm_open_anonymous-XXXX\0";
    loop {
        let path = {
            use std::io::Write;

            // replace the last four characters with "random" digits
            if let Some(random) = pseudorandom() {
                write!(&mut filename[20..], "{:4}", random % 10000).unwrap();
                std::ffi::CStr::from_bytes_with_nul(filename.as_ref()).unwrap()
            } else {
                return -1;
            }
        };

        debug_assert!(path.to_str().unwrap().starts_with("/shm_open_anonymous-"));

        // Try creating shared memory with the provided path.
        // If creation fails with EEXIST, try another filename until it works.
        let fd = shm_open_path(&path);
        if fd == -1 && errno() != libc::EEXIST {
            return -1;
        } else if fd != -1 {
            if shm_unlink(path) == -1 {
                // fd is valid
                unsafe {
                    libc::close(fd);
                }
                return -1;
            }
            return fd;
        }
    }
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
pub fn shm_open_anonymous() -> RawFd {
    #[cfg(target_os = "linux")]
    {
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

        // Try opening with memfd_create.
        // If that fails, use the generic POSIX method.
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

    #[cfg(target_os = "freebsd")]
    {
        // no invariants to uphold
        unsafe { libc::shm_open(libc::SHM_ANON, libc::O_RDWR, 0) }
    }

    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
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
