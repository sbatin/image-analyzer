<script>
  import utils from './utils.js';

  export default {
    props: ['files'],
    emits: ['click'],

    methods: {
      formatFile(file) {
        const size = utils.formatSize(file.size);
        const date = utils.formatDate(file.date);
        return `${date} (${size})`;
      },

      getFileName(file) {
        return utils.getFileName(file.path);
      },
    }
  }
</script>
<template>
  <div class="col" v-for="file of files">
    <figure class="figure">
      <a href="javascript:void(0)" @click="$emit('click', file.path)">
        <img class="figure-img img-fluid rounded" :src="`image?path=${file.path}`" :title="file.relativePath"/>
      </a>
      <figcaption class="figure-caption img-title">{{ getFileName(file) }}</figcaption>
      <figcaption class="figure-caption">{{ formatFile(file) }}</figcaption>
    </figure>
  </div>
</template>
<style scoped>
.figure-img {
  max-height: 200px;
}
.img-title {
  max-width: 150px;
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
}
</style>
