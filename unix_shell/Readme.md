# Unix Shell

(Project described in *Operating System Concepts, 10th edition*, pp. 204)

---

Writing this in C needs using `fork()`, `exec()`, `wait()`, `dup2()`, and `pipe()`.

In Rust we use `nix` library to do those syscalls.

## Redirection

`dup2(fd1, fd2)` means if you do something to `fd2`, you will actually do it to `fd` without changing `fd2`.

## Internal commands support

- `exit`: exit with a given exit code
- `cd`: change working directory

## Syntax

There are no semicolon, keywords. You are only allowed to do pipes and redirections.

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
a |
ls | cat < a
```

## Multiple pipes

```
echo hello | head -c 1 | cat

head /dev/urandom | tr -dc a-z | head -c 10

ps -ef | awk "{print $1}" | sort | uniq -c | sort -n
```

## Known Issues

- No post-processors, so if a program outputs something without `\n`, you might not see it

## Reference

- https://stackoverflow.com/questions/35007063/c-pipe-and-fork-closing-nothing-gets-printed
