<script>
import * as bootstrap from 'bootstrap';
import ImageList from './ImageList.vue';
import API from './api';

export default {
  methods: {
    async analyzePoll() {
      const resp = await API.poll(this.path);
      switch (resp.type) {
        case 'Pending': {
          this.progress = resp.progress;
          await this.analyzePoll();
          break;
        }
        case 'Completed': {
          this.pending = false;
          this.similarImages = resp.data;
          return;
        }
      }
    },
    async analyze() {
      this.pending = true;
      this.progress = 0;
      await API.analyze(this.path, this.distance);
      //await this.analyzePoll();

      API.subscribe(this.path, (progress) => {
        this.progress = progress;
        if (this.progress === 100) {
          this.analyzePoll();
        }
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
      progress: 0,
      pending: false,
      distance: 10,
      images: [],
      similarImages: undefined,
      modal: undefined,
      selectedImage: '',
    };
  },
  mounted() {
    this.modal = new bootstrap.Modal('#image-popup');
    API.listDir(this.path).then((images) => {
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
      <div v-if="pending" class="progress mx-3" role="progressbar" style="width:400px">
        <div class="progress-bar" :style="`width: ${progress}%`"></div>
      </div>
      <input type="text" class="form-control me-2" style="max-width:100px" v-model="distance"/>
      <button class="btn btn-success" type="button" @click="analyze" :disabled="pending">
        <span v-if="pending" class="spinner-border spinner-border-sm" aria-hidden="true"></span>
        <span v-if="pending" role="status">Analyzing...</span>
        <span v-if="!pending">Analyze</span>
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