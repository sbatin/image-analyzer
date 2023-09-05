<script>
import * as bootstrap from 'bootstrap';
import ImageList from './ImageList.vue';

export default {
  methods: {
    analyze() {
      this.progress = true;
      fetch(`/analyze?path=${this.path}&dist=${this.distance}`, {
        method: 'POST',
      })
        .then((resp) => resp.json())
        .then((data) => {
          console.log('similar images', data);
          this.progress = false;
          this.similarImages = data;
        });
    },
    openImage(src) {
      this.selectedImage = src;
      this.modal.show();
    }
  },
  data() {
    const [, search] = window.location.hash.split('?');
    const params = new URLSearchParams(search);
    return {
      path: params.get('path'),
      progress: false,
      distance: 10,
      images: [],
      similarImages: undefined,
      modal: undefined,
      selectedImage: '',
    };
  },
  mounted() {
    this.modal = new bootstrap.Modal('#image-popup');
    fetch(`/list_folder?path=${this.path}`)
      .then((resp) => resp.json())
      .then((images) => {
        this.images = images;
      });
  },
  components: { ImageList }
}
</script>
<template>
  <nav class="navbar fixed-top bg-dark bg-body-tertiary" data-bs-theme="dark">
    <div class="container-fluid">
      <a class="navbar-brand" href="#">Image DeDup</a>
      <span class="navbar-text">{{ path }}</span>
      <ul class="navbar-nav me-auto mb-2 mb-lg-0"></ul>
      <input type="text" class="form-control me-2" style="max-width:100px" v-model="distance"/>
      <button class="btn btn-success" type="button" @click="analyze" :disabled="progress">
        <span v-if="progress" class="spinner-border spinner-border-sm" aria-hidden="true"></span>
        <span v-if="progress" role="status">Analyzing...</span>
        <span v-if="!progress">Analyze</span>
      </button>
    </div>
  </nav>
  <div class="content">
    <div class="container py-5">
      <div v-if="!similarImages" class="row row-cols-auto gy-4">
        <ImageList :images="images" @click="openImage"/>
      </div>
      <div v-if="similarImages">
        <div class="row row-cols-auto img-group gy-4" v-for="group of similarImages">
          <ImageList :images="group" @click="openImage"/>
        </div>
      </div>
    </div>
  </div>
  <div id="image-popup" class="modal" tabindex="-1">
    <div class="modal-dialog">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">{{ selectedImage.substring(path.length + 1) }}</h5>
          <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
        </div>
        <div class="modal-body">
          <img class="img-full" :src="`image?path=${selectedImage}`"/>
        </div>
        <div class="modal-footer">
          <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Close</button>
          <button type="button" class="btn btn-danger">Remove</button>
        </div>
      </div>
    </div>
  </div>
</template>
<style scoped>
.content {
  padding-top: 60px;
}
.modal-dialog {
  max-width: fit-content;
}
.modal-body {
  text-align: center;
}
.img-full {
  max-height: 600px;
}
.img-group {
  border-bottom: 1px solid var(--bs-border-color);
  padding: 20px 0;
}
</style>