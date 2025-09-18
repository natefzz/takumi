export interface FontDetails {
  name?: string;
  data: Buffer | ArrayBuffer;
  weight?: number;
  style?: "normal" | "italic" | "oblique";
}

export type Font = FontDetails | Buffer | ArrayBuffer;

export interface AnyNode {
  type: string;
}
