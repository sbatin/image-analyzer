<script>
  import * as bootstrap from 'bootstrap';
  import API from './api.js';

  export default {
    props: ['path'],
    emits: ['deleted'],

    data() {
      return {
        image: '',
        modal: undefined,
      }
    },

    methods: {
      open(src) {
        this.image = src;
        this.modal.show();
      },

      async deleteFile() {
        await API.deleteFile(this.image);
        this.$emit('deleted', this.image);
        this.modal.hide();
      }
    },

    mounted() {
      this.modal = new bootstrap.Modal('#image-popup');
    },
  }
</script>
<template>
  <div id="image-popup" class="modal" tabindex="-1">
    <div class="modal-dialog">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">{{ image.substring(path.length + 1) }}</h5>
          <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
        </div>
        <div class="modal-body">
          <img class="img-full" :src="`image?path=${image}`"/>
        </div>
        <div class="modal-footer">
          <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Close</button>
          <button type="button" class="btn btn-danger" @click="deleteFile">Delete</button>
        </div>
      </div>
    </div>
  </div>
</template>
<style scoped>
.modal-dialog {
  max-width: fit-content;
}
.modal-body {
  text-align: center;
}
.img-full {
  max-height: 600px;
}
</style>
