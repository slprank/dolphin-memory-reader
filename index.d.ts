export = DolphinMemory

declare class DolphinMemory {
  constructor();

  read(address: number, length: number): Uint8Array;
  readString(address: number, chars: number): string;
}
