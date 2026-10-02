import { parseFragment, decryptPayload } from './crypto.js';

const $ = (id) => document.getElementById(id);
let generation = 0;
let feedbackTimer;

function showState(state) {
  for (const name of ['loading', 'success', 'error', 'no-payload', 'cleared']) {
    $(`state-${name}`).classList.toggle('hidden', name !== state);
  }
}

function clearDisplay() {
  $('secret-output').value = '';
  $('copy-btn').disabled = true;
  clearTimeout(feedbackTimer);
  $('copy-feedback').classList.add('hidden');
  $('copy-feedback').textContent = '';
}

async function main() {
  const current = ++generation;
  clearDisplay();
  const fragment = window.location.hash.slice(1);
  // Remove the current entry's fragment before validating/decrypting. This does
  // not erase prior browser sync, the original message, or another copy of the URL.
  try {
    history.replaceState(null, '', window.location.pathname + window.location.search);
  } catch {
    $('error-message').textContent = 'URLから鍵を取り除けません。このブラウザでは開けません。';
    showState('error');
    return;
  }
  if (!fragment) {
    showState('no-payload');
    return;
  }
  showState('loading');
  let data;
  try {
    if (!window.isSecureContext || !crypto.subtle) throw new Error('HTTPSで開いてください。');
    data = parseFragment(fragment);
    const plaintext = await decryptPayload(data.payload, data.key);
    if (current !== generation) return;
    $('secret-output').value = plaintext;
    $('copy-btn').disabled = false;
    showState('success');
  } catch {
    if (current !== generation) return;
    $('error-message').textContent = 'URLが欠けているか、形式・鍵・暗号文が正しくありません。元の共有URLを確認してください。';
    showState('error');
  } finally {
    data?.key.fill(0);
    data?.payload.fill(0);
  }
}

$('copy-btn').addEventListener('click', async () => {
  const current = generation;
  const text = $('secret-output').value;
  if (!text || $('copy-btn').disabled) return;
  $('copy-btn').disabled = true;
  try {
    await navigator.clipboard.writeText(text);
    if (current !== generation) return;
    $('copy-feedback').textContent = 'コピーしました。クリップボードの履歴にもご注意ください。';
  } catch {
    if (current !== generation) return;
    $('secret-output').focus();
    $('secret-output').select();
    $('copy-feedback').textContent = 'コピーできませんでした。選択されたテキストを手動でコピーしてください。';
  } finally {
    if (current === generation) {
      $('copy-btn').disabled = false;
      $('copy-feedback').classList.remove('hidden');
      clearTimeout(feedbackTimer);
      feedbackTimer = setTimeout(() => $('copy-feedback').classList.add('hidden'), 4000);
    }
  }
});

$('clear-btn').addEventListener('click', () => {
  ++generation;
  clearDisplay();
  showState('cleared');
});
window.addEventListener('hashchange', main);
window.addEventListener('pagehide', () => { ++generation; clearDisplay(); });
window.addEventListener('pageshow', (event) => { if (event.persisted) main(); });
main();
