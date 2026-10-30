<script lang="ts">
  interface Props {
    size?: "sm" | "md";
  }

  let { size = "md" }: Props = $props();
</script>

<span class="brand-mark" class:sm={size === "sm"} aria-hidden="true">
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path d="M12 3l7 3v5c0 4.4-2.9 7.9-7 10-4.1-2.1-7-5.6-7-10V6l7-3z" />
    <circle cx="12" cy="10.5" r="1.6" />
    <path d="M12 12.1V15" />
  </svg>
</span>

<style>
  .brand-mark {
    display: grid;
    place-items: center;
    width: 2.15rem;
    height: 2.15rem;
    border-radius: 0.65rem;
    color: #fff;
    background: linear-gradient(135deg, var(--primary), var(--secondary));
    box-shadow: 0 8px 24px -10px color-mix(in srgb, var(--primary) 80%, transparent);
    flex-shrink: 0;
  }

  .brand-mark svg {
    width: 1.25rem;
    height: 1.25rem;
  }

  .brand-mark.sm {
    width: 1.75rem;
    height: 1.75rem;
    border-radius: 0.5rem;
    box-shadow: 0 6px 18px -8px color-mix(in srgb, var(--primary) 80%, transparent);
  }

  .brand-mark.sm svg {
    width: 1rem;
    height: 1rem;
  }
</style>
