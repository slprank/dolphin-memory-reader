## Dolphin Memory Reader

### Initialize

```ts
import DolphinMemory, { ByteSize } from "dolphin-memory-reader";
import os from "os"

readFromMemory() {
    if (os.platform() !== "win32") return;

    const memory = new DolphinMemory();

    // P1 Selected Character In CSS
    const address = 0x8043208b;
    const byte = memory.read(address, ByteSize.U8);

    console.log("Byte from memory", byte);
}
```

### Info

Package is written in rust and is currently only supported for windows and tested on `Super Smash Bros Melee`

### Contribute?

To be able to run this project locally you will need to have `Rust` installed on your computer with `Nightly`
