import { encryptMessage } from "../crypto/crypto";
import { logger } from "../utils/logger";

import { getWayveRecipientByEmail } from "../api/email";

export type EmailEncryptionMode = "fully_encrypted" | "standard";

export async function buildEncryptedBody(
  to: string,
  body: string,
  token: string,
  encryptionMode: EmailEncryptionMode
) {
  // =====================================
  // LOG ORIGINAL BODY
  // =====================================
  logger.warn("📨 ORIGINAL BODY:");
  logger.warn(body);

  if (encryptionMode === "standard") {
    logger.warn("Standard encryption selected → sending Gmail-readable body");
    return body;
  }

  // =====================================
  // CHECK USER
  // =====================================
  const users = await getWayveRecipientByEmail(to, token);

  const user = Array.isArray(users)
    ? users[0]
    : users;

  // no encryption
  if (!user?.public_key) {
    throw new Error(
      "Fully encrypted email requires the recipient to have Wayve encryption enabled. Choose Standard encryption for Gmail-readable email."
    );
  }

  // =====================================
  // IMPORT PUBLIC KEY
  // =====================================
  const parsedKey =
    typeof user.public_key === "string"
      ? JSON.parse(user.public_key)
      : user.public_key;

  const publicKey =
    await crypto.subtle.importKey(
      "spki",
      new Uint8Array(parsedKey),
      {
        name: "RSA-OAEP",
        hash: "SHA-256",
      },
      true,
      ["encrypt"]
    );

  // =====================================
  // ENCRYPT
  // =====================================
  const {
    encryptedMessage,
    encryptedKey,
    iv,
  } = await encryptMessage(
    body,
    publicKey
  );

  const encryptedBody =
    "WAYVE_SECURE_V1\n" +
    JSON.stringify({
      type: "wayve_encrypted",
      data: Array.from(
        new Uint8Array(
          encryptedMessage
        )
      ),
      key: Array.from(
        new Uint8Array(
          encryptedKey
        )
      ),
      iv: Array.from(iv),
    });

  // =====================================
  // LOG ENCRYPTED BODY
  // =====================================
  logger.warn(
    "🔐 ENCRYPTED BODY:"
  );

  logger.warn(encryptedBody);

  return encryptedBody;
}
