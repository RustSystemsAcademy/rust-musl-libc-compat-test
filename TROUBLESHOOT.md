# Troubleshoot
## Does a trivial musl binary work at all?
cat > /tmp/hello.c << 'EOF'
#include <stdio.h>
int main() { puts("hello"); return 0; }
EOF
musl-gcc -static /tmp/hello.c -o /tmp/hello-musl
file /tmp/hello-musl
/tmp/hello-musl

## Check if a core file was written
ls -la core* /tmp/core* 2>/dev/null

## If systemd-coredump is active on SUSE 12 (unlikely but possible)
coredumpctl list 2>/dev/null

## Get the crash address at minimum
dmesg | tail -20

## See exactly where it faults
ulimit -c unlimited
./musl-compat-test
ls -la core*

