<script lang="ts">
  import '../stylesheets/Grid.css'
  type GridProps = {
    row?: boolean
    column?: boolean
    col?: boolean
    children?: any
    style?: string
    gap?: number
    class?: string
    role?: string
    ariaLabel?: string
    justify?: 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly'
    align?: 'start' | 'center' | 'end' | 'stretch' | 'baseline'
    fullWidth?: boolean
  }

  let {
    row = false,
    column = false,
    col = false,
    children,
    style = '',
    gap = 0,
    role = 'group',
    ariaLabel = '',
    class: _class = '',
    justify = 'start',
    align = 'center',
    fullWidth = false
  }: GridProps = $props()

  // Compute flex-direction from props
  let flexDirection = $state('row')
  if (column || col) {
    flexDirection = 'column'
  }
  if (row) {
    flexDirection = 'row'
  }

  // Compute justify-content and convert into a class name
  const justifyContentClasses = {
    center: 'justify-center',
    end: 'justify-end',
    between: 'justify-between',
    around: 'justify-around',
    evenly: 'justify-evenly'
  }
  let justifyContentClass = $state(justifyContentClasses[justify] ?? 'justify-start')

  // Compute align-items and convert into a class name
  const alignItemsClasses = {
    center: 'align-center',
    end: 'align-end',
    stretch: 'align-stretch',
    baseline: 'align-baseline'
  }

  let alignItemsClass = $state(alignItemsClasses[align] ?? 'align-start')

  let wrapClass = row ? 'wrap' : ''
</script>

<div
  class={`grid-container ${fullWidth && 'full-width'} ${_class} ${flexDirection} ${justifyContentClass} ${alignItemsClass} ${wrapClass}`}
  {role}
  aria-label={ariaLabel}
  style={`--grid-gap: ${gap / 4}rem; ${style}`}
>
  {@render children?.()}
</div>
