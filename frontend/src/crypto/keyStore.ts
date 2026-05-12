const DB_NAME = "wayve_keys";
const STORE_NAME = "keys";
const DB_VERSION = 1;
const LEGACY_PRIVATE_KEY_ID = "privateKey";
const LEGACY_PUBLIC_KEY_ID = "publicKey";

function privateKeyId(userId?: number | null) {
  return userId ? `privateKey:${userId}` : LEGACY_PRIVATE_KEY_ID;
}

function publicKeyId(userId?: number | null) {
  return userId ? `publicKey:${userId}` : LEGACY_PUBLIC_KEY_ID;
}

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);

    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME);
      }
    };

    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

// 🔐 Save private key
export async function savePrivateKey(key: CryptoKey, userId?: number | null) {
  const db = await openDB();

  const exported = await crypto.subtle.exportKey("pkcs8", key);

  return new Promise<void>((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);

    store.put(exported, privateKeyId(userId));

    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

// 🔐 Save public key bytes so we can re-publish the DB key after reload/login.
export async function savePublicKey(publicKey: ArrayBuffer, userId?: number | null) {
  const db = await openDB();
  const publicKeyBytes = new Uint8Array(publicKey).slice().buffer;

  return new Promise<void>((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);

    store.put(publicKeyBytes, publicKeyId(userId));

    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

// 🔓 Load private key
export async function loadPrivateKey(userId?: number | null): Promise<CryptoKey | null> {
  const db = await openDB();

  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readonly");
    const store = tx.objectStore(STORE_NAME);

    const req = store.get(privateKeyId(userId));

    req.onsuccess = async () => {
      if (!req.result) return resolve(null);

      const key = await crypto.subtle.importKey(
        "pkcs8",
        req.result,
        { name: "RSA-OAEP", hash: "SHA-256" },
        true,
        ["decrypt"]
      );

      resolve(key);
    };

    req.onerror = () => reject(req.error);
  });
}

// 🔓 Load saved public key bytes for server registration.
export async function loadPublicKey(userId?: number | null): Promise<ArrayBuffer | null> {
  const db = await openDB();

  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readonly");
    const store = tx.objectStore(STORE_NAME);

    const req = store.get(publicKeyId(userId));

    req.onsuccess = () => {
      if (!req.result) return resolve(null);

      if (req.result instanceof ArrayBuffer) {
        return resolve(req.result);
      }

      resolve(new Uint8Array(req.result).slice().buffer);
    };

    req.onerror = () => reject(req.error);
  });
}
