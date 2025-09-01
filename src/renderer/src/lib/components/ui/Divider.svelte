<script lang="ts">
  type DividerProps = {
    size?: 'xs' | 'sm' | 'md' | 'lg'
    variant?: 'solid' | 'dashed' | 'dotted'
    color?: 'default' | 'primary' | 'error' | 'success' | 'warning' | 'info' | 'secondary'
    vertical?: boolean
    class?: string
    style?: string
  }

  let {
    size = 'md',
    variant = 'solid',
    color = 'default',
    vertical = false,
    class: className = '',
    style = ''
  }: DividerProps = $props()

  const sizeMap = {
    xs: '1px',
    sm: '1px',
    md: '2px',
    lg: '3px'
  }

  const colorMap = {
    default: 'var(--border)',
    primary: 'var(--primary)',
    error: 'var(--error)',
    success: 'var(--success)',
    warning: 'var(--warning)',
    info: 'var(--info)',
    secondary: 'var(--secondary)'
  }
</script>

<div
  class={`divider ${className}`}
  style={`
    ${vertical ? 'border-left' : 'border-top'}: ${sizeMap[size]} ${variant} ${colorMap[color]};
    ${vertical ? 'height: 100%; width: 0; margin: 0 0.75rem;' : 'width: 100%; height: 0; margin: 0.75rem 0;'}
    opacity: ${variant === 'dashed' || variant === 'dotted' ? '0.7' : '1'}; ${style}
  `}
  aria-orientation={vertical ? 'vertical' : 'horizontal'}
></div>
