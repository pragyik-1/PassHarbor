<script lang="ts">
	import { colors } from '../../utils';
	type DividerProps = {
		size?: 'xs' | 'sm' | 'md' | 'lg';
		variant?: 'solid' | 'dashed' | 'dotted';
		color?: 'default' | 'primary' | 'error' | 'success' | 'warning' | 'info' | 'secondary';
		vertical?: boolean;
		class?: string;
		style?: string;
	};

	let {
		size = 'md',
		variant = 'solid',
		color = 'default',
		vertical = false,
		class: _class = '',
		style = ''
	}: DividerProps = $props();

	const sizeMap = {
		xs: '1px',
		sm: '1px',
		md: '2px',
		lg: '3px'
	};

	const _color = $derived.by(() => {
		if (color === 'default') return colors.border;
		return colors[color];
	});
</script>

<div
	class={`divider ${_class}`}
	style={`
    ${vertical ? 'border-left' : 'border-top'}: ${sizeMap[size]} ${variant} ${_color};
    ${vertical ? 'height: 100%; width: 0; margin: 0 0.75rem;' : 'width: 100%; height: 0; margin: 0.75rem 0;'}
    opacity: ${variant === 'dashed' || variant === 'dotted' ? '0.7' : '1'}; ${style}
  `}
	aria-orientation={vertical ? 'vertical' : 'horizontal'}
></div>
