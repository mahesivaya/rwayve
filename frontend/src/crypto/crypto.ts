function toArrayBuffer(input: ArrayBuffer | SharedArrayBuffer | Uint8Array): ArrayBuffer {
  if (input instanceof Uint8Array) {
    // copy to a new ArrayBuffer
    return input.slice().buffer;
  }
  if (input instanceof ArrayBuffer) {
    return input;
  }
  // SharedArrayBuffer → copy into a new ArrayBuffer
  const view = new Uint8Array(input);
  return view.slice().buffer;
}


// 🔐 Generate AES key
export async function generateKey(): Promise<CryptoKey> {
  return crypto.subtle.generateKey(
    { name: "AES-GCM", length: 256 },
    true,
    ["encrypt", "decrypt"]
  );
}

// 🔐 Encrypt
export async function encrypt(text: string, key: CryptoKey) {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encoded = new TextEncoder().encode(text);

  const ciphertext = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    key,
    encoded
  );

  return {
    iv: btoa(String.fromCharCode(...iv)), // base64
    data: btoa(String.fromCharCode(...new Uint8Array(ciphertext))), // base64
  };
}

// 🔐 Decrypt
export async function decrypt(
  encrypted: { iv: string; data: string },
  key: CryptoKey
) {
  const iv = Uint8Array.from(atob(encrypted.iv), c => c.charCodeAt(0));
  const data = Uint8Array.from(atob(encrypted.data), c => c.charCodeAt(0));

  const decrypted = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv },
    key,
    data
  );

  return new TextDecoder().decode(decrypted);
}


export async function decryptMessage(
  encryptedMessage: ArrayBuffer | SharedArrayBuffer | Uint8Array,
  encryptedKey: ArrayBuffer | SharedArrayBuffer | Uint8Array,
  iv: Uint8Array,
  privateKey: CryptoKey
): Promise<string> {

  const msg = toArrayBuffer(encryptedMessage);
  const key = toArrayBuffer(encryptedKey);

  const rawKey = await crypto.subtle.decrypt(
    { name: "RSA-OAEP" },
    privateKey,
    key
  );

  const aesKey = await crypto.subtle.importKey(
    "raw",
    rawKey,
    { name: "AES-GCM" },
    false,
    ["decrypt"]
  );

  const decrypted = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv },
    aesKey,
    msg
  );

  return new TextDecoder().decode(decrypted);
}



export async function encryptMessage(
  message: string,
  publicKey: CryptoKey
) {
  // 1. Generate AES key
  const aesKey = await crypto.subtle.generateKey(
    { name: "AES-GCM", length: 256 },
    true,
    ["encrypt", "decrypt"]
  );

  // 2. Create IV
  const iv = crypto.getRandomValues(new Uint8Array(12));

  // 3. Encrypt message
  const encryptedMessage = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    aesKey,
    new TextEncoder().encode(message)
  );

  // 4. Export AES key
  const rawKey = await crypto.subtle.exportKey("raw", aesKey);

  // 5. Encrypt AES key with receiver public key
  const encryptedKey = await crypto.subtle.encrypt(
    { name: "RSA-OAEP" },
    publicKey,
    rawKey
  );

  return {
    encryptedMessage,
    encryptedKey,
    iv,
  };
}