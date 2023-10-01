<script>
  export default {
    props: ['images', 'selected'],
    emits: ['click', 'dblclick'],

    data() {
      return {
        fileList: this.images.sort((a, b) => b.date - a.date),
      }
    },

    methods: {
      formatSize(size) {
        if (size < 1024) {
          return `${size}B`;
        } else if (size < 1024*1024) {
          const newSize = Math.round(10 * size / 1024) / 10;
          return `${newSize}kB`;
        } else {
          const newSize = Math.round(10 * size / 1024 / 1024) / 10;
          return `${newSize}MB`;
        }
      },

      formatFile(file) {
        const options = { year: 'numeric', month: 'short', day: 'numeric' };
        const size = this.formatSize(file.size);
        const date = new Date(file.date).toLocaleDateString(undefined, options);

        return `${date} (${size})`;
      },

      getFileName(path) {
        const parts = path.split('/');
        return parts[parts.length - 1];
      }
    }
  }
</script>
<template>
  <div class="col" v-for="file of fileList">
    <figure :class="{ figure, selected: file.path === selected}">
      <a href="javascript:void(0)" @dblclick="$emit('dblclick', file.path)" @click.stop.prevent="$emit('click', file.path)">
        <img class="figure-img img-fluid rounded" :src="`image?path=${file.path}`" :title="file.path"/>
      </a>
      <figcaption class="figure-caption img-title">{{ getFileName(file.path) }}</figcaption>
      <figcaption class="figure-caption">{{ formatFile(file) }}</figcaption>
    </figure>
  </div>
</template>
<style scoped>
.figure-img {
  max-height: 200px;
}
.selected .figure-img {
  border: 3px solid #0d6efd;
}
.img-title {
  max-width: 150px;
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
}
</style>
