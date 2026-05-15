import { decryptMessage } from "../crypto/crypto";
import { loadPrivateKey } from "../crypto/keyStore";
import type { WayveEncryptedBody } from "./types";

const WAYVE_SECURE_PREFIX = "WAYVE_SECURE_V1";

export function normalizeEmailBody(body: string) {
  if (!/[<&][a-zA-Z#/!]/.test(body)) {
    return body;
  }

  const doc = new DOMParser().parseFromString(body, "text/html");

  doc
    .querySelectorAll("script, style, noscript, svg")
    .forEach((node) => node.remove());

  doc
    .querySelectorAll("br")
    .forEach((node) => node.replaceWith(doc.createTextNode("\n")));

  doc
    .querySelectorAll("p, div, section, article, header, footer, tr, table")
    .forEach((node) => node.append(doc.createTextNode("\n")));

  doc
    .querySelectorAll("li")
    .forEach((node) => node.prepend(doc.createTextNode("\n- ")));

  const text = doc.body.textContent || body;

  return text
    .replace(/\u00a0/g, " ")
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n[ \t]+/g, "\n")
    .replace(/[ \t]{2,}/g, " ")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function parseWayveEncryptedBody(body: string): WayveEncryptedBody | null {
  const trimmed = normalizeEmailBody(body).trim();

  if (!trimmed.startsWith(WAYVE_SECURE_PREFIX)) {
    return null;
  }

  const jsonStart = trimmed.indexOf("{");
  if (jsonStart === -1) {
    throw new Error("Encrypted Wayve email is missing its payload");
  }

  const jsonEnd = trimmed.lastIndexOf("}");
  if (jsonEnd < jsonStart) {
    throw new Error("Encrypted Wayve email payload is incomplete");
  }

  const parsed = JSON.parse(trimmed.slice(jsonStart, jsonEnd + 1));

  if (
    parsed?.type !== "wayve_encrypted" ||
    !Array.isArray(parsed.data) ||
    !Array.isArray(parsed.key) ||
    !Array.isArray(parsed.iv)
  ) {
    throw new Error("Encrypted Wayve email payload is invalid");
  }

  return parsed;
}

export function emailBodyErrorMessage(err: unknown) {
  const message = err instanceof Error ? err.message : "";

  if (
    message.includes("private key") ||
    message.includes("decrypt") ||
    message.includes("operation failed")
  ) {
    return "Unable to decrypt this fully encrypted email on this device. Sign out and back in to refresh your Wayve encryption key, then ask the sender to resend it.";
  }

  if (message) {
    return message;
  }

  return "Failed to load email body. Try again.";
}

export async function decryptWayveBodyIfNeeded(
  body: string,
  userId?: number | null
): Promise<string> {
  const encrypted = parseWayveEncryptedBody(body);

  if (!encrypted) {
    return normalizeEmailBody(body);
  }

  const privateKeys: CryptoKey[] = [];
  const scopedPrivateKey = await loadPrivateKey(userId);

  if (scopedPrivateKey) {
    privateKeys.push(scopedPrivateKey);
  }

  if (userId) {
    const legacyPrivateKey = await loadPrivateKey();
    if (legacyPrivateKey && legacyPrivateKey !== scopedPrivateKey) {
      privateKeys.push(legacyPrivateKey);
    }
  }

  if (privateKeys.length === 0) {
    throw new Error("This device does not have your Wayve private key");
  }

  let lastError: unknown = null;

  for (const privateKey of privateKeys) {
    try {
      return await decryptMessage(
        new Uint8Array(encrypted.data),
        new Uint8Array(encrypted.key),
        new Uint8Array(encrypted.iv),
        privateKey
      ).then(normalizeEmailBody);
    } catch (err) {
      lastError = err;
    }
  }

  throw lastError || new Error("Unable to decrypt Wayve email");
}
