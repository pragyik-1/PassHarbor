<script lang="ts">
  import { Grid } from '../../../lib'

  let { title, children } = $props()
</script>

<Grid row justify="between" align="center" fullWidth>
  <p>{title}</p>
  {@render children?.()}
</Grid>
