/**
 * What an FTP account refusal says, in the operator's language.
 *
 * The backend refuses with a dictionary key — `ftp.error.missingHost` — and,
 * where there is one, the detail after `": "`: a file path, the credential
 * store's own reason. Anything else is passed through as it came.
 */
import type { MessageKey } from "./i18n";

const REFUSAL = /^(ftp\.error\.[A-Za-z]+)(?::\s*([\s\S]*))?$/;

export function ftpErrorText(
  error: unknown,
  translate: (key: MessageKey) => string,
): string {
  const text = String(error);
  const match = REFUSAL.exec(text);
  if (!match) return text;
  const message = translate(match[1] as MessageKey);
  // An unknown key comes back as itself from the fallback, which is worse
  // than the backend's own words.
  if (message === undefined || message === match[1]) return text;
  return match[2] ? `${message}: ${match[2]}` : message;
}
