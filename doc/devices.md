devices are local to a node so can't migrate. This is achieved by pinning the vm addressing.

A process accessing pinned memory would need to migrate to complete. 

A process doing io on two nodes would bounce between the two. A better design would create two threads and a buffer (or IPC) between the two.

Naming

node name
device name
va



