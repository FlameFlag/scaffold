<script lang="ts">
import { onDestroy } from "svelte";

let {
  value,
  label = "Copy",
}: {
  value: string;
  label?: string;
} = $props();

type CopyStatus = "idle" | "copied" | "selected" | "failed";

let status = $state<CopyStatus>("idle");
let resetTimer: number | undefined;
let buttonElement: HTMLButtonElement;

const buttonTextByStatus: Partial<Record<CopyStatus, string>> = {
  copied: "Copied",
  failed: "Failed",
  selected: "Selected",
};

const statusMessageByStatus: Partial<Record<CopyStatus, string>> = {
  copied: "Copied to clipboard.",
  failed: "Copy failed.",
  selected: "Copy unavailable; source text selected.",
};

const buttonText = $derived(buttonTextByStatus[status] ?? label);
const statusMessage = $derived(statusMessageByStatus[status] ?? "");

onDestroy(() => {
  if (resetTimer !== undefined) {
    window.clearTimeout(resetTimer);
  }
});

async function copyValue() {
  if (resetTimer !== undefined) {
    window.clearTimeout(resetTimer);
  }

  if (await writeClipboard(value)) {
    status = "copied";
  } else if (selectNearbyCode(buttonElement)) {
    status = "selected";
  } else {
    status = "failed";
  }

  resetTimer = window.setTimeout(() => {
    status = "idle";
  }, 1400);
}

async function writeClipboard(text: string) {
  if (await writeWithClipboardApi(text)) {
    return true;
  }

  return writeWithSelectionFallback(text);
}

async function writeWithClipboardApi(text: string) {
  try {
    if (!navigator.clipboard?.writeText) {
      return false;
    }

    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

function writeWithSelectionFallback(text: string) {
  const previousFocus = document.activeElement;
  const selection = window.getSelection();
  const ranges: Range[] = [];

  if (selection) {
    for (let index = 0; index < selection.rangeCount; index += 1) {
      ranges.push(selection.getRangeAt(index).cloneRange());
    }
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.readOnly = true;
  textarea.style.position = "fixed";
  textarea.style.left = "0";
  textarea.style.top = "0";
  textarea.style.width = "1px";
  textarea.style.height = "1px";
  textarea.style.opacity = "0.01";
  document.body.append(textarea);

  textarea.focus();
  textarea.select();
  textarea.setSelectionRange(0, text.length);

  try {
    return document.execCommand("copy");
  } catch {
    return false;
  } finally {
    textarea.remove();

    if (selection) {
      selection.removeAllRanges();
      for (const range of ranges) {
        selection.addRange(range);
      }
    }

    if (previousFocus instanceof HTMLElement) {
      previousFocus.focus();
    }
  }
}

function selectNearbyCode(button: HTMLButtonElement | undefined) {
  const container =
    button?.closest(".copyBlock") ?? button?.closest(".sourceDisclosure");
  const target = container?.querySelector<HTMLElement>(
    ".sourceCode code, pre code, pre",
  );
  const selection = window.getSelection();

  if (!target || !selection) {
    return false;
  }

  const range = document.createRange();
  range.selectNodeContents(target);
  selection.removeAllRanges();
  selection.addRange(range);

  return selection.toString().length > 0;
}
</script>

<button
  class="copyButton"
  type="button"
  bind:this={buttonElement}
  onclick={copyValue}
  aria-label={buttonText}
>
  {buttonText}
</button>
<span class="srOnly" aria-live="polite">{statusMessage}</span>
