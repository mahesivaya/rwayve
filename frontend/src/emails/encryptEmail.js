import { encryptMessage } from "../crypto/crypto";
import { logger } from "../utils/logger";
import { apiFetch } from "../api/client";
export async function buildEncryptedBody(to, body, token) {
    // =====================================
    // LOG ORIGINAL BODY
    // =====================================
    logger.warn("📨 ORIGINAL BODY:");
    logger.warn(body);
    // =====================================
    // CHECK USER
    // =====================================
    const checkRes = await apiFetch(`/api/users?email=${to}`, {
        headers: {
            Authorization: `Bearer ${token}`,
        },
    });
    // normal email
    if (!checkRes.ok) {
        logger.warn("⚠️ User lookup failed → sending normal email");
        return body;
    }
    const users = await checkRes.json();
    const user = Array.isArray(users)
        ? users[0]
        : users;
    // no encryption
    if (!user?.public_key) {
        logger.warn("⚠️ No public key → sending normal email");
        return body;
    }
    // =====================================
    // IMPORT PUBLIC KEY
    // =====================================
    const parsedKey = typeof user.public_key === "string"
        ? JSON.parse(user.public_key)
        : user.public_key;
    const publicKey = await crypto.subtle.importKey("spki", new Uint8Array(parsedKey), {
        name: "RSA-OAEP",
        hash: "SHA-256",
    }, true, ["encrypt"]);
    // =====================================
    // ENCRYPT
    // =====================================
    const { encryptedMessage, encryptedKey, iv, } = await encryptMessage(body, publicKey);
    const encryptedBody = "WAYVE_SECURE_V1\n" +
        JSON.stringify({
            type: "wayve_encrypted",
            data: Array.from(new Uint8Array(encryptedMessage)),
            key: Array.from(new Uint8Array(encryptedKey)),
            iv: Array.from(iv),
        });
    // =====================================
    // LOG ENCRYPTED BODY
    // =====================================
    logger.warn("🔐 ENCRYPTED BODY:");
    logger.warn(encryptedBody);
    return encryptedBody;
}
