# qclock_rs
---

Toolset for reproducing Quine Clock in Rust: https://gist.github.com/Iskaa303/49fa2ed027e4c99b4dcd244b88a52f9f

Inspired by https://gist.github.com/rexim/f582098611b2be202051ba543e21da05 who was inspired by https://x.com/aemkei/status/1795762928399880680

---

# Quick Start

Linux/MacOS
```bash
$ wget https://gist.githubusercontent.com/Iskaa303/49fa2ed027e4c99b4dcd244b88a52f9f/raw/de50f922e79fd38b5e28d7a760fee1f1acc3908f/qclock.rs -O qclock.rs
$ rustc qclock.rs -o qclock
$ ./qclock
```

Windows
```powershell
$ wget https://gist.githubusercontent.com/Iskaa303/49fa2ed027e4c99b4dcd244b88a52f9f/raw/de50f922e79fd38b5e28d7a760fee1f1acc3908f/qclock.rs -O qclock.rs
$ rustc qclock.rs -o qclock.exe
$ ./qclock.exe
```

---

# Usage

``` ./qclock -b ``` to make it bounce like that DVD logo
``` ./qclock hh:mm:ss ``` to set a timer

Press ``` q ``` to quit the program
Press ``` space ``` to pause the program
Press ``` s ``` to stop the bouncing