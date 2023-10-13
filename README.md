## Dolphin Memory Reader

### Initialize

```ts
import DolphinMemory from "dolphin-memory-reader";

const memory = new DolphinMemory();

const address = 0x80000000;
const byte = memory.read(address, ByteSize.U8);

console.log("Byte from memory", byte);
```

### Info

Package is written in rust and is currently only supported for windows tested on `Super Smash Bros Melee`
