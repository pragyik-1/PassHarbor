<script lang="ts">
  import { Grid, Input } from '../../../lib'

  let { value = $bindable(), id = 'dir-input' } = $props()
</script>

<Grid col>
  <Input {id} type="directory" webkitdirectory bind:value />
  <p class="description-text">currently: {value}</p>
</Grid>
