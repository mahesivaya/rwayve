const DB_NAME = "wayve_keys";
const STORE_NAME = "keys";
const DB_VERSION = 1;
function openDB() {
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
export async function savePrivateKey(key) {
    const db = await openDB();
    const exported = await crypto.subtle.exportKey("pkcs8", key);
    return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, "readwrite");
        const store = tx.objectStore(STORE_NAME);
        store.put(exported, "privateKey");
        tx.oncomplete = () => resolve();
        tx.onerror = () => reject(tx.error);
    });
}
// 🔓 Load private key
export async function loadPrivateKey() {
    const db = await openDB();
    return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, "readonly");
        const store = tx.objectStore(STORE_NAME);
        const req = store.get("privateKey");
        req.onsuccess = async () => {
            if (!req.result)
                return resolve(null);
            const key = await crypto.subtle.importKey("pkcs8", req.result, { name: "RSA-OAEP", hash: "SHA-256" }, true, ["decrypt"]);
            resolve(key);
        };
        req.onerror = () => reject(req.error);
    });
}
