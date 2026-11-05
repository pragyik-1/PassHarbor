<script lang="ts">
  import { Modal, Button, Input, Switch, toast } from "@hermitk/bluenite";
  import {
    CopyIcon,
    CheckIcon,
    RefreshCwIcon,
    ChevronDownIcon,
    ChevronUpIcon,
    ShieldCheckIcon,
  } from "lucide-svelte";
  import {
    generatePassword,
    calculateEntropy,
    DEFAULT_GENERATOR_OPTIONS,
    type PasswordGeneratorOptions,
  } from "#lib/generator.ts";

  interface Props {
    open: boolean;
    onclose?: () => void;
  }

  let { open = $bindable(), onclose }: Props = $props();

  // ── Generator Configuration State ──────────────────────────

  let genOpts = $state<PasswordGeneratorOptions>({
    ...DEFAULT_GENERATOR_OPTIONS,
    customCharset: "",
  });

  let showAdvanced = $state(false);
  let copied = $state(false);
  let copiedTimeout: ReturnType<typeof setTimeout> | null = null;
  let isSpinning = $state(false);

  let currentPassword = $state("");

  // ── Password Generation ────────────────────────────────────

  function regenerate() {
    try {
      const opts: PasswordGeneratorOptions = {
        length: genOpts.length,
        lowercase: genOpts.lowercase,
        uppercase: genOpts.uppercase,
        numbers: genOpts.numbers,
        symbols: genOpts.symbols,
        excludeSimilar: genOpts.excludeSimilar,
        excludeAmbiguous: genOpts.excludeAmbiguous,
        customCharset: genOpts.customCharset?.trim() || undefined,
        minLowercase: genOpts.lowercase ? (genOpts.minLowercase ?? 0) : 0,
        minUppercase: genOpts.uppercase ? (genOpts.minUppercase ?? 0) : 0,
        minNumbers: genOpts.numbers ? (genOpts.minNumbers ?? 0) : 0,
        minSymbols: genOpts.symbols ? (genOpts.minSymbols ?? 0) : 0,
      };
      currentPassword = generatePassword(opts);
    } catch {
      // If all character sets are unchecked, display empty state
      currentPassword = "";
    }
  }

  function handleRegenerateClick() {
    isSpinning = true;
    regenerate();
    setTimeout(() => {
      isSpinning = false;
    }, 300);
  }

  // Generate fresh password on open or configuration changes
  $effect(() => {
    if (open) {
      regenerate();
    }
  });

  // ── Entropy & Strength Calculation ────────────────────────

  const entropyBits = $derived(
    currentPassword ? calculateEntropy(genOpts) : 0,
  );

  const strength = $derived.by(() => {
    if (!currentPassword || entropyBits < 40) {
      return {
        label: "Weak",
        color: "var(--danger)",
        pct: Math.min(30, Math.max(10, entropyBits)),
      };
    }
    if (entropyBits < 65) {
      return {
        label: "Moderate",
        color: "#f59e0b",
        pct: 60,
      };
    }
    if (entropyBits < 90) {
      return {
        label: "Strong",
        color: "#10b981",
        pct: 85,
      };
    }
    return {
      label: "Very strong",
      color: "var(--primary)",
      pct: 100,
    };
  });

  // ── Copy to Clipboard ──────────────────────────────────────

  async function copyPassword() {
    if (!currentPassword) return;
    try {
      await navigator.clipboard.writeText(currentPassword);
      copied = true;
      if (copiedTimeout) clearTimeout(copiedTimeout);
      copiedTimeout = setTimeout(() => {
        copied = false;
      }, 1800);
      toast.show({
        message: "Password copied to clipboard",
        variant: "success",
        duration: 2000,
      });
    } catch {
      toast.show({
        message: "Failed to copy password",
        variant: "danger",
      });
    }
  }

  function handleClose() {
    open = false;
    onclose?.();
  }

  const presets = [16, 20, 24, 32, 36, 48];
</script>

<Modal bind:open size="md" title="Password generator" onClose={handleClose}>
  <div class="modal-body">
    <!-- Prominent Password Display Box -->
    <div class="display-card">
      <div class="pw-box">
        <span class="pw-text" class:empty={!currentPassword}>
          {currentPassword || "Select at least one character type"}
        </span>

        <div class="pw-actions">
          <Button
            variant="ghost"
            color="secondary"
            size="sm"
            class="action-btn"
            aria-label="Generate new password"
            title="Generate new password"
            onclick={handleRegenerateClick}
          >
            {#snippet icon()}
              <span class="icon-wrap" class:spin={isSpinning}>
                <RefreshCwIcon size={16} />
              </span>
            {/snippet}
          </Button>

          <Button
            variant="ghost"
            color="secondary"
            size="sm"
            class="action-btn"
            aria-label="Copy password"
            title="Copy password"
            disabled={!currentPassword}
            onclick={copyPassword}
          >
            {#snippet icon()}
              {#if copied}
                <CheckIcon size={16} color="var(--primary)" />
              {:else}
                <CopyIcon size={16} />
              {/if}
            {/snippet}
          </Button>
        </div>
      </div>

      <!-- Strength meter -->
      <div class="strength-bar-wrap">
        <div
          class="strength-bar-fill"
          style="width: {strength.pct}%; background-color: {strength.color};"
        ></div>
      </div>

      <div class="strength-meta">
        <span class="meta-label">
          <ShieldCheckIcon size={13} />
          <strong style="color: {strength.color};">{strength.label}</strong>
        </span>
        <span class="meta-stats">
          {genOpts.length} characters · {entropyBits} bits entropy
        </span>
      </div>
    </div>

    <!-- Length Controls -->
    <section class="section">
      <div class="section-head">
        <label for="length-slider" class="section-title">Password length</label>
        <span class="length-badge">{genOpts.length}</span>
      </div>

      <div class="slider-row">
        <input
          id="length-slider"
          type="range"
          min="8"
          max="64"
          bind:value={genOpts.length}
          class="range-slider"
          aria-label="Password length"
        />
      </div>

      <div class="presets-row">
        {#each presets as preset (preset)}
          <button
            type="button"
            class="preset-chip"
            class:active={genOpts.length === preset}
            onclick={() => {
              genOpts.length = preset;
            }}
          >
            {preset} chars
          </button>
        {/each}
      </div>
    </section>

    <!-- Character Types -->
    <section class="section">
      <div class="section-head">
        <span class="section-title">Character types</span>
      </div>

      <div class="options-grid">
        <label class="option-card">
          <div class="option-info">
            <span class="option-name">Uppercase letters</span>
            <span class="option-sample">A, B, C, D…</span>
          </div>
          <Switch bind:checked={genOpts.uppercase} size="sm" />
        </label>

        <label class="option-card">
          <div class="option-info">
            <span class="option-name">Lowercase letters</span>
            <span class="option-sample">a, b, c, d…</span>
          </div>
          <Switch bind:checked={genOpts.lowercase} size="sm" />
        </label>

        <label class="option-card">
          <div class="option-info">
            <span class="option-name">Numbers</span>
            <span class="option-sample">0, 1, 2, 3…</span>
          </div>
          <Switch bind:checked={genOpts.numbers} size="sm" />
        </label>

        <label class="option-card">
          <div class="option-info">
            <span class="option-name">Symbols</span>
            <span class="option-sample">! @ # $ % ^ & *</span>
          </div>
          <Switch bind:checked={genOpts.symbols} size="sm" />
        </label>
      </div>
    </section>

    <!-- Exclusion Options -->
    <section class="section">
      <div class="section-head">
        <span class="section-title">Readability & exclusions</span>
      </div>

      <div class="options-list">
        <label class="option-row">
          <div class="option-info">
            <span class="option-name">Exclude visually similar characters</span>
            <span class="option-sub"
              >Avoids lookalikes like i, l, 1, I, o, 0, O</span
            >
          </div>
          <Switch bind:checked={genOpts.excludeSimilar} size="sm" />
        </label>

        <label class="option-row">
          <div class="option-info">
            <span class="option-name">Exclude ambiguous symbols</span>
            <span class="option-sub"
              >Avoids punctuation like {"{ } [ ] ( ) / \\ ' \" ` ~ , ; : . < >"}</span
            >
          </div>
          <Switch bind:checked={genOpts.excludeAmbiguous} size="sm" />
        </label>
      </div>
    </section>

    <!-- Advanced Options Toggle -->
    <div class="advanced-wrapper">
      <button
        type="button"
        class="advanced-toggle-btn"
        onclick={() => (showAdvanced = !showAdvanced)}
        aria-expanded={showAdvanced}
      >
        <span>Advanced options</span>
        {#if showAdvanced}
          <ChevronUpIcon size={15} />
        {:else}
          <ChevronDownIcon size={15} />
        {/if}
      </button>

      {#if showAdvanced}
        <div class="advanced-panel">
          <!-- Custom Charset -->
          <div class="adv-field">
            <label for="custom-charset" class="adv-label">
              Custom characters to include
            </label>
            <Input
              id="custom-charset"
              bind:value={genOpts.customCharset}
              placeholder="e.g. _-+= or additional symbols"
            />
          </div>

          <!-- Minimum Guarantees -->
          <div class="min-grid">
            <div class="min-item">
              <label for="min-lower">Min. lowercase</label>
              <input
                id="min-lower"
                type="number"
                min="0"
                max={genOpts.length}
                bind:value={genOpts.minLowercase}
                disabled={!genOpts.lowercase}
                class="num-input"
              />
            </div>

            <div class="min-item">
              <label for="min-upper">Min. uppercase</label>
              <input
                id="min-upper"
                type="number"
                min="0"
                max={genOpts.length}
                bind:value={genOpts.minUppercase}
                disabled={!genOpts.uppercase}
                class="num-input"
              />
            </div>

            <div class="min-item">
              <label for="min-num">Min. numbers</label>
              <input
                id="min-num"
                type="number"
                min="0"
                max={genOpts.length}
                bind:value={genOpts.minNumbers}
                disabled={!genOpts.numbers}
                class="num-input"
              />
            </div>

            <div class="min-item">
              <label for="min-sym">Min. symbols</label>
              <input
                id="min-sym"
                type="number"
                min="0"
                max={genOpts.length}
                bind:value={genOpts.minSymbols}
                disabled={!genOpts.symbols}
                class="num-input"
              />
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>

  {#snippet footer()}
    <div class="modal-footer">
      <Button variant="ghost" color="secondary" size="sm" onclick={handleClose}>
        Close
      </Button>

      <Button size="sm" disabled={!currentPassword} onclick={copyPassword}>
        {#snippet icon()}
          {#if copied}
            <CheckIcon size={15} />
          {:else}
            <CopyIcon size={15} />
          {/if}
        {/snippet}
        {copied ? "Copied!" : "Copy password"}
      </Button>
    </div>
  {/snippet}
</Modal>

<style>
  .modal-body {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding: 0.25rem 0;
  }

  /* ── Password Display Card ──────────────────────────────── */

  .display-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--round-lg);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .pw-box {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    min-height: 2.75rem;
  }

  .pw-text {
    font-family: ui-monospace, "Cascadia Code", "Fira Mono", monospace;
    font-size: 1.125rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    color: var(--text);
    word-break: break-all;
    user-select: all;
  }

  .pw-text.empty {
    font-size: 0.875rem;
    font-family: inherit;
    font-style: italic;
    color: var(--danger);
  }

  .pw-actions {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex-shrink: 0;
  }

  :global(.action-btn) {
    width: 2rem !important;
    height: 2rem !important;
    padding: 0 !important;
  }

  .icon-wrap {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: transform 300ms ease;
  }

  .icon-wrap.spin {
    transform: rotate(180deg);
  }

  /* Strength meter */
  .strength-bar-wrap {
    height: 4px;
    background: color-mix(in srgb, var(--border) 60%, transparent);
    border-radius: 999px;
    overflow: hidden;
  }

  .strength-bar-fill {
    height: 100%;
    transition:
      width 200ms ease,
      background-color 200ms ease;
  }

  .strength-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--muted-text);
  }

  .meta-label {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }

  .meta-stats {
    font-variant-numeric: tabular-nums;
  }

  /* ── Section Structure ───────────────────────────────────── */

  .section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-title {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text);
  }

  .length-badge {
    font-family: ui-monospace, "Cascadia Code", "Fira Mono", monospace;
    font-size: 0.8125rem;
    font-weight: 700;
    color: var(--primary);
    background: color-mix(in srgb, var(--primary) 10%, transparent);
    padding: 0.1rem 0.45rem;
    border-radius: var(--round-sm);
  }

  /* Slider */
  .slider-row {
    display: flex;
    align-items: center;
  }

  .range-slider {
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: var(--surface-hover);
    outline: none;
    cursor: pointer;
    accent-color: var(--primary);
  }

  .presets-row {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.25rem;
  }

  .preset-chip {
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--secondary-text);
    padding: 0.2rem 0.5rem;
    border-radius: var(--round-sm);
    font-size: 0.72rem;
    cursor: pointer;
    transition:
      border-color 120ms ease,
      color 120ms ease;
  }

  .preset-chip:hover,
  .preset-chip.active {
    border-color: var(--primary);
    color: var(--primary);
  }

  .preset-chip.active {
    font-weight: 600;
    background: color-mix(in srgb, var(--primary) 8%, transparent);
  }

  /* Options Grid */
  .options-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
  }

  @media (max-width: 500px) {
    .options-grid {
      grid-template-columns: 1fr;
    }
  }

  .option-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--round-md);
    padding: 0.6rem 0.75rem;
    cursor: pointer;
  }

  .option-info {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    min-width: 0;
  }

  .option-name {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text);
  }

  .option-sample,
  .option-sub {
    font-size: 0.72rem;
    color: var(--muted-text);
  }

  /* Options List */
  .options-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .option-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.4rem 0.5rem;
    border-radius: var(--round-sm);
    cursor: pointer;
  }

  .option-row:hover {
    background: var(--surface-hover);
  }

  /* ── Advanced Options Panel ──────────────────────────────── */

  .advanced-wrapper {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--border);
    padding-top: 0.75rem;
  }

  .advanced-toggle-btn {
    display: inline-flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    background: none;
    border: none;
    padding: 0.35rem 0.25rem;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--secondary-text);
    cursor: pointer;
  }

  .advanced-toggle-btn:hover {
    color: var(--text);
  }

  .advanced-panel {
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
    padding-top: 0.75rem;
  }

  .adv-field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .adv-label {
    font-size: 0.78rem;
    font-weight: 500;
    color: var(--secondary-text);
  }

  .min-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.5rem;
  }

  @media (max-width: 500px) {
    .min-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  .min-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .min-item label {
    font-size: 0.7rem;
    color: var(--muted-text);
  }

  .num-input {
    width: 100%;
    box-sizing: border-box;
    padding: 0.35rem 0.5rem;
    font-size: 0.8125rem;
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: var(--round-sm);
    outline: none;
  }

  .num-input:focus {
    border-color: var(--primary);
  }

  .num-input:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* Footer */
  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.5rem;
    width: 100%;
  }
</style>
