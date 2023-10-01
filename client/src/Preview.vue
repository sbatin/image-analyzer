<script>
  import * as bootstrap from 'bootstrap';

  export default {
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
        <img class="rounded" :src="`image?path=${image}`"/>
      </div>
    </div>
  </div>
</template>
<style scoped>
.modal-dialog {
  max-width: fit-content;
}
.modal-content img {
  max-height: 700px;
}
</style>
