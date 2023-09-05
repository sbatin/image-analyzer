<script>
  export default {
    props: ['images']
  }
</script>
<template>
  <div class="col" v-for="src of images">
    <a href="javascript:void(0)" @click="$emit('click', src)">
      <img class="img-thumbnail" :src="`image?path=${src}`" :title="src"/>
    </a>
  </div>
</template>
<style scoped>
.img-thumbnail {
  max-height: 200px;
}
</style>