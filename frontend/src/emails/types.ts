export type Email = {
  id: number;
  sender: string;
  receiver: string;
  subject: string;
  preview?: string;
  body?: string;
  created_at: string;
};

export type WayveEncryptedBody = {
  type: "wayve_encrypted";
  data: number[];
  key: number[];
  iv: number[];
};
