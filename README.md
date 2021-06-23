# rcmd
Runs a command in another process through CreateRemoteThread(&WinExec, "<command ...>"). 

Good for when you are SYSTEM and want to inject commands into another user.

```
rcmd.exe <pid> <command ...>
```
