# Unix Shell

(Project described in *Operating System Concepts, 10th edition*, pp. 204)

---

Writing this in C needs using `fork()`, `exec()`, `wait()`, `dup2()`, `chdir()` and `pipe()`.

In Rust we use `nix` library to do those syscalls.

## Redirection

`dup2(fd1, fd2)` means if you do something to `fd2`, you will actually do it to `fd` without changing `fd2`.

## Internal commands support

- `exit`: exit with a given exit code
- `cd`: change working directory

## Syntax

There are no semicolon, keywords. You are only allowed to use pipe and redirection operator.

You cannot have 2 stdin for one command.

The rules of those operators are:

- `>` expects only **one** file location (might not exist), and there should be no other commands afterwards
- `<` also expects **one** file location and cannot be used after a pipe, but it can be followed by **other** two operators
- `|` expects any commands, but not avoiding the rules above

Invalid examples:

```
ls > a > a
ls > a < a
ls > a | cat
ls < a < s
ls | cat > b | m
ls | cat < a
```

## Multiple pipes

```
echo hello | head -c 1 | cat | ./post

head /dev/urandom | tr -dc [:graph:] | head -c 10 | ./post

ps -ef | awk "{print $1}" | sort | uniq -c | sort -n
```

## Known Issues

- No internal post-processors, so if a program outputs something without `\n`, you might not see it

## Warnings

> It is important to notice that both the parent process and the child process initially close their unused ends of the pipe.
>
> It is an important step to ensure that a process reading from the pipe can detect end-of-file (`read()` returns 0) when the writer has closed its end of the pipe.

## Reference

- https://stackoverflow.com/questions/35007063/c-pipe-and-fork-closing-nothing-gets-printed
- https://stackoverflow.com/questions/11599462/what-happens-if-a-child-process-wont-close-the-pipe-from-writing-while-reading
