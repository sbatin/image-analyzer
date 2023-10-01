<script>
  export default {
    props: ['error'],
  }
</script>
<template>
  <div v-if="error" class="alert alert-danger" role="alert">
    <h4 class="alert-heading">Error</h4>
    <p>{{ error.message }}</p>
  </div>
</template>
