## Dolphin Memory Reader

### Initialize

```ts
import DolphinMemory, { ByteSize } from "dolphin-memory-reader";
import os from "os"

readFromMemory() {
    if (os.platform() !== "win32") return;

    // Throws error if dolphin is not running
    const memory = new DolphinMemory();

    // Current stage Id address
    const address = 0x8049e6c8 + 0x88 + 0x03;

    // Throws error if not able to read memory address or dolphin is no longer active when called
    const byte = memory.read(address, ByteSize.U8);

    console.log("Byte from memory", byte);
}
```

### Info

Package is written in rust and is currently only supported for windows and tested on `Super Smash Bros Melee`

### Contribute

To be able to run this project locally you will need to have `Rust` installed on your computer with `Nightly`
