# Unix Shell

(Project described in *Operating System Concepts, 10th edition*, pp. 204)

---

Writing this in C needs using `fork()`, `exec()`, `wait()`, `dup2()`, and `pipe()`.

In Rust we use `nix` library to do those syscalls.

## Redirection

`dup2(fd1, fd2)` means if you do something to `fd2`, you will actually do it to `fd` without changing `fd2`.
