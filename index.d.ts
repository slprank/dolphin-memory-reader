declare class Memory {
  constructor();

  read(address: number, length: number): number;
  readString(address: number, chars: number): string;
}

export = Memory;
