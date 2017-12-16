# path-shadows

Find shadowed programs in $PATH, that is programs for which a program of the same name exists in a folder that occurs earlier in $PATH.
Program paths that point to the same file (like symlinks) are skipped

# Example

The format is `/path/to/shadowed/bin:/path/to/shadowing/bin`

```
$ path-shadows
/usr/bin/rust-gdb:/home/someuser/.cargo/bin/rust-gdb
/usr/bin/rust-lldb:/home/someuser/.cargo/bin/rust-lldb
```