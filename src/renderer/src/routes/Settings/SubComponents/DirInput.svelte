<script lang="ts">
  import { Grid, Input } from '../../../lib'

  let { value = $bindable() } = $props()
</script>

<Grid col>
  <Input type="directory" webkitdirectory bind:value />
  <p class="description-text">currently: {value}</p>
</Grid>
