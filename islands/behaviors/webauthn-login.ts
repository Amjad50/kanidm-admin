import { defineBehavior } from './index';
import { base64urlToBytes, bytesToBase64url } from '../lib/base64url';

function showError(form: HTMLElement, message: string): void {
  const errorEl = form.querySelector<HTMLElement>('[data-webauthn-error]');
  if (!errorEl) return;
  const inner = errorEl.querySelector<HTMLElement>('.flex-1');
  if (inner) inner.textContent = message;
  errorEl.hidden = false;
}

async function runWebAuthnFlow(form: HTMLElement): Promise<void> {
  const raw = (form as HTMLFormElement).dataset.webauthnChallenge;
  if (!raw) return;

  const credInput = form.querySelector<HTMLInputElement>('#webauthn-cred');
  if (!credInput) return;

  let cro: any;
  try {
    // Outer wrapper is standard base64 (atob-decodable); parse the JSON inside.
    cro = JSON.parse(atob(raw));
  } catch {
    showError(form, 'Your device returned an unexpected response.');
    return;
  }

  try {
    cro.publicKey.challenge = base64urlToBytes(cro.publicKey.challenge);
    for (const c of cro.publicKey.allowCredentials ?? []) {
      c.id = base64urlToBytes(c.id);
    }

    const assertion = await navigator.credentials.get({ publicKey: cro.publicKey }) as PublicKeyCredential | null;
    if (!assertion) {
      showError(form, 'No passkey was used. Click to try again.');
      return;
    }

    const resp = assertion.response as AuthenticatorAssertionResponse;
    const envelope = {
      id: assertion.id,
      rawId: bytesToBase64url(assertion.rawId),
      type: assertion.type,
      response: {
        authenticatorData: bytesToBase64url(resp.authenticatorData),
        clientDataJSON: bytesToBase64url(resp.clientDataJSON),
        signature: bytesToBase64url(resp.signature),
        userHandle: resp.userHandle !== null ? bytesToBase64url(resp.userHandle) : null,
      },
    };

    credInput.value = JSON.stringify(envelope);
    (form as HTMLFormElement).submit();
  } catch (err: any) {
    const name: string = err?.name ?? '';
    if (name === 'NotAllowedError' || name === 'AbortError') {
      showError(form, 'No passkey was used. Click to try again.');
    } else {
      showError(form, 'Your device returned an unexpected response.');
    }
  }
}

function autoFireOnRoot(root: ParentNode): void {
  const form = root.querySelector<HTMLElement>('[data-webauthn-form]');
  if (!form) return;
  if (!window.PublicKeyCredential) {
    const startBtn = form.querySelector<HTMLButtonElement>('[data-webauthn-start]');
    if (startBtn) startBtn.disabled = true;
    showError(form, "Your browser doesn't support WebAuthn.");
    return;
  }
  runWebAuthnFlow(form);
}

// Auto-fire on load: if the form is present and WebAuthn is available, start
// the flow immediately without waiting for a button click.
// entry.ts runs with `defer` — the DOM is already parsed when this module
// executes, so no DOMContentLoaded listener is needed.
autoFireOnRoot(document);

// Same auto-fire after HTMX swaps in new content. Idempotent — the WebAuthn
// API itself short-circuits if the user has already produced an assertion.
document.body.addEventListener('htmx:afterSwap', (e) => {
  const target = (e as CustomEvent).detail?.target as ParentNode | undefined;
  if (target) autoFireOnRoot(target);
});

// Manual fallback: clicking [data-webauthn-start] re-runs the flow.
defineBehavior({
  selector: '[data-webauthn-start]',
  event: 'click',
  handler: (el, event) => {
    event.preventDefault();
    const form = el.closest<HTMLElement>('[data-webauthn-form]');
    if (!form) return;
    runWebAuthnFlow(form);
  },
});
